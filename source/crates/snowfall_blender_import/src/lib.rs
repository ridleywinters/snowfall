use anyhow::{Context, Result};
use blend::{Blend, Instance};
use glam::Vec3;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;

mod bbox;
pub use bbox::BBox;
mod mesh;
pub use mesh::*;
// Blender object type constants
const OBJ_TYPE_EMPTY: i32 = 0;
const OBJ_TYPE_MESH: i32 = 1;
// Blender uses a directly serialized format where the pointers are the
// size used on the host system that wrote the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerSize {
    Bits32,
    Bits64,
}

/// Byte order in the blend file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    Little,
    Big,
}

// Internal helper structures for building the scene graph
#[derive(Debug, Clone)]
struct CollectionData {
    name: String,
    mesh_children: Vec<String>,
    collection_children: Vec<String>,
}

#[derive(Debug, Clone)]
struct InstanceData {
    mesh_ref: Option<String>,
    collection_ref: Option<String>,
    collection_library_path: Option<String>,
    transform: MTransform,
}

/// A Blender file containing mesh data and metadata
#[derive(Debug, Clone)]
pub struct BlendFile {
    /// Blender version that created this file as ASCII bytes (e.g., ['4', '0', '5'] for Blender 4.0.5)
    pub version: [u8; 3],
    pub pointer_size: PointerSize,
    pub endianness: Endianness,

    pub linked_libraries: Vec<String>,
    pub collections: Vec<MGroup>,
    pub scene: MScene,
}

impl BlendFile {
    /// Get the Blender version as a string (e.g., "4.0.5")
    pub fn version_string(&self) -> String {
        format!(
            "{}.{}.{}",
            self.version[0] as char, self.version[1] as char, self.version[2] as char
        )
    }
}

/// Load mesh data from a .blend file
pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<BlendFile> {
    let path = path.as_ref();

    // First, scan for linked library files
    let data =
        std::fs::read(path).with_context(|| format!("Failed to read file: {}", path.display()))?;

    let blend_file = Blend::new(Cursor::new(&data))
        .map_err(|e| anyhow::anyhow!("Failed to parse .blend file: {:?}", e))?;

    // Extract linked library paths
    let mut linked_libraries = Vec::new();
    for instance in blend_file.instances_with_code(*b"LI") {
        if instance.is_valid("name") {
            let filepath = instance.get_string("name");
            linked_libraries.push(filepath);
        }
    }

    // Load all linked libraries as complete scenes with meshes
    let mut linked_scenes = Vec::new();
    for lib_path in linked_libraries {
        // Resolve relative path
        let resolved_path = if lib_path.starts_with("//") {
            // Blender relative path - relative to the blend file
            let parent_dir = path.parent().unwrap_or(Path::new("."));
            parent_dir.join(&lib_path[2..])
        } else {
            Path::new(&lib_path).to_path_buf()
        };

        println!(
            "Loading linked library: {} -> {}",
            lib_path,
            resolved_path.display()
        );

        if resolved_path.exists() {
            match load_linked_scene(&resolved_path, &lib_path) {
                Ok(scene) => {
                    linked_scenes.push((lib_path.clone(), scene));
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load linked library {}: {}", lib_path, e);
                }
            }
        } else {
            eprintln!(
                "Warning: Linked library not found: {}",
                resolved_path.display()
            );
        }
    }

    // Extract library paths from linked_scenes for storage
    let linked_library_paths: Vec<String> =
        linked_scenes.iter().map(|(path, _)| path.clone()).collect();

    load_from_memory_with_linked_scenes(&data, None, &linked_scenes, linked_library_paths)
}

fn load_linked_scene<P: AsRef<Path>>(path: P, _lib_path: &str) -> Result<MScene> {
    let path = path.as_ref();
    let data = std::fs::read(path)
        .with_context(|| format!("Failed to read linked file: {}", path.display()))?;

    let blend_file = Blend::new(Cursor::new(&data))
        .map_err(|e| anyhow::anyhow!("Failed to parse linked .blend file: {:?}", e))?;

    let mut scene = MScene {
        meshes: HashMap::new(),
        materials: HashMap::new(),
        root: MGroup {
            name: None,
            children: Vec::new(),
            transform: None,
        },
    };
    for instance in blend_file.instances_with_code(*b"ME") {
        let (mesh_id, mesh) = extract_mesh_data(&instance, None)?;
        scene.meshes.insert(mesh_id, mesh);
    }

    // Extract collections from linked file
    let mut collections = Vec::new();
    let mut collection_names = std::collections::HashSet::new();

    let mut add_collection = |coll: CollectionData| {
        if collection_names.insert(coll.name.clone()) {
            collections.push(coll);
        }
    };

    // Extract CO blocks
    for instance in blend_file.instances_with_code(*b"CO") {
        let collection = extract_collection_data(&instance)
            .with_context(|| "Failed to extract collection from linked file")?;
        add_collection(collection);
    }

    // Extract GR blocks
    for instance in blend_file.instances_with_code(*b"GR") {
        let collection = extract_group_data(&instance)
            .with_context(|| "Failed to extract group from linked file")?;
        add_collection(collection);
    }

    // Build collection map
    let mut collection_map: HashMap<String, CollectionData> = HashMap::new();
    for collection in collections {
        collection_map.insert(collection.name.clone(), collection);
    }

    // Build scene graph from collections (no instances needed for linked files)
    // Each collection becomes a group in the root with its name preserved
    for (collection_name, collection_data) in collection_map.iter() {
        let mut group = build_group_from_collection(collection_data, &collection_map, None, None)?;
        group.name = Some(collection_name.clone());
        scene.root.children.push(MNode::MGroup(group));
    }

    Ok(scene)
}

fn load_from_memory_with_linked_scenes(
    data: &[u8],
    mesh_id_prefix: Option<&str>,
    linked_scenes: &[(String, MScene)],
    linked_libraries: Vec<String>,
) -> Result<BlendFile> {
    let blend_file = Blend::new(Cursor::new(data))
        .map_err(|e| anyhow::anyhow!("Failed to parse .blend file: {:?}", e))?;

    let header = &blend_file.blend.header;
    let version = header.version;

    if version[0] < b'4' {
        return Err(anyhow::anyhow!(
            "Blender 4.0 or newer required, found version {}.{}.{}",
            version[0] as char,
            version[1] as char,
            version[2] as char
        ));
    }

    let pointer_size = match header.pointer_size {
        blend::parsers::PointerSize::Bits32 => PointerSize::Bits32,
        blend::parsers::PointerSize::Bits64 => PointerSize::Bits64,
    };

    let endianness = match header.endianness {
        blend::parsers::Endianness::Little => Endianness::Little,
        blend::parsers::Endianness::Big => Endianness::Big,
    };

    // Initialize MScene
    let mut scene = MScene {
        meshes: HashMap::new(),
        materials: HashMap::new(),
        root: MGroup {
            name: None,
            children: Vec::new(),
            transform: None,
        },
    };
    for instance in blend_file.instances_with_code(*b"ME") {
        let (mesh_id, mesh) = extract_mesh_data(&instance, mesh_id_prefix)?;
        scene.meshes.insert(mesh_id, mesh);
    }

    // Extract instances
    let mut instances = Vec::new();
    for instance in blend_file.instances_with_code(*b"OB") {
        if let Some(instance_data) = extract_instance_data(&instance)? {
            instances.push(instance_data);
        }
    }
    println!("Total instances: {}", instances.len());

    // Extract collections from main file
    let mut collections = Vec::new();
    let mut collection_names = std::collections::HashSet::new();

    // Helper to add collection without duplicates
    let mut add_collection = |coll: CollectionData| {
        if collection_names.insert(coll.name.clone()) {
            collections.push(coll);
        }
    };

    // Extract ALL CO blocks from main file
    for instance in blend_file.instances_with_code(*b"CO") {
        let collection =
            extract_collection_data(&instance).with_context(|| "Failed to extract collection")?;
        add_collection(collection);
    }

    // Extract from Scene's master_collection hierarchy (but skip the master collection itself)
    let mut scene_collections = Vec::new();
    for scene_instance in blend_file.instances_with_code(*b"SC") {
        if scene_instance.is_valid("master_collection") {
            let master_coll = scene_instance.get("master_collection");
            extract_collections_from_hierarchy(&master_coll, &mut scene_collections)?;
        }
    }
    for coll in scene_collections {
        // Skip "Scene Collection" which is the master collection container
        if coll.name != "Scene Collection" {
            add_collection(coll);
        }
    }

    // Extract GR (Group) blocks
    for instance in blend_file.instances_with_code(*b"GR") {
        let collection =
            extract_group_data(&instance).with_context(|| "Failed to extract group")?;
        add_collection(collection);
    }

    for collection_data in &collections {
        println!(
            "Collection:\n\tname={}\n\tmesh_children={:?}\n\tcollection_children={:?}",
            collection_data.name,
            collection_data.mesh_children,
            collection_data.collection_children
        );
    }
    for collection_data in &collections {
        println!(
            "Collection:\n\tname={}\n\tmesh_children={:?}\n\tcollection_children={:?}",
            collection_data.name,
            collection_data.mesh_children,
            collection_data.collection_children
        );
    }
    println!("Total collections: {}", collections.len());

    // Build scene graph from collections and instances
    build_scene_graph(
        &mut scene,
        collections,
        instances,
        mesh_id_prefix,
        linked_scenes,
    )?;

    Ok(BlendFile {
        version,
        pointer_size,
        endianness,
        scene,
        linked_libraries,
    })
}

/// Extract mesh data from a blend file instance
fn extract_mesh_data(
    instance: &Instance,
    mesh_id_prefix: Option<&str>,
) -> Result<(MMeshID, MMesh)> {
    let clean_name = clean_blender_id(instance, "ME");

    let mesh_id = if let Some(prefix) = mesh_id_prefix {
        format!("{}{}", prefix, clean_name)
    } else {
        clean_name.clone()
    };

    let mesh = MMesh::new(clean_name);
    let mesh = extract_mesh_data_v4(instance, mesh)?;
    Ok((mesh_id, mesh))
}

fn extract_mesh_data_v4(instance: &Instance, mut mesh: MMesh) -> Result<MMesh> {
    let totloop = instance.get_i32("totloop") as usize;
    let totpoly = instance.get_i32("totpoly") as usize;

    if instance.is_valid("vdata") {
        let vdata = instance.get("vdata");
        if vdata.is_valid("layers") {
            for layer in vdata.get_iter("layers") {
                let layer_name = layer.get_string("name");

                if layer_name == "position" && layer.is_valid("data") {
                    for vert_data in layer.get_iter("data") {
                        let x = vert_data.get_f32("x");
                        let y = vert_data.get_f32("y");
                        let z = vert_data.get_f32("z");
                        mesh.positions.push(Vec3::new(x, y, z));
                    }
                    break;
                }
            }
        }
    }

    let mut corner_verts = Vec::new();
    if instance.is_valid("ldata") {
        let ldata = instance.get("ldata");
        if ldata.is_valid("layers") {
            for layer in ldata.get_iter("layers") {
                let layer_name = layer.get_string("name");

                if layer_name == ".corner_vert" && layer.is_valid("data") {
                    for loop_data in layer.get_iter("data") {
                        let vert_idx = loop_data.get_i32("i") as u32;
                        corner_verts.push(vert_idx);
                    }
                    break;
                }
            }
        }
    }

    let corners_per_poly = if totpoly > 0 { totloop / totpoly } else { 0 };

    for poly_idx in 0..totpoly {
        let start = poly_idx * corners_per_poly;
        let end = start + corners_per_poly;

        if end <= corner_verts.len() && corners_per_poly >= 3 {
            for i in 1..(corners_per_poly - 1) {
                mesh.indices.push(corner_verts[start]);
                mesh.indices.push(corner_verts[start + i]);
                mesh.indices.push(corner_verts[start + i + 1]);
            }
        }
    }

    mesh.bbox = BBox::from_positions(&mesh.positions);
    Ok(mesh)
}

fn extract_collection_data(instance: &Instance) -> Result<CollectionData> {
    let name = clean_blender_id(instance, "CO");
    let mesh_children = extract_mesh_children(instance);

    let mut collection_children = Vec::new();
    if instance.is_valid("children") {
        for child_coll_instance in instance.get_iter("children") {
            if child_coll_instance.is_valid("collection") {
                let child_coll = child_coll_instance.get("collection");
                let child_name = clean_blender_id(&child_coll, "CO");
                collection_children.push(child_name);
            }
        }
    }

    Ok(CollectionData {
        name,
        mesh_children,
        collection_children,
    })
}

fn extract_group_data(instance: &Instance) -> Result<CollectionData> {
    let name = if instance.is_valid("id") {
        clean_blender_id(instance, "GR")
    } else {
        "Unknown".to_string()
    };

    let mesh_children = extract_mesh_children(instance);

    Ok(CollectionData {
        name,
        mesh_children,
        collection_children: Vec::new(),
    })
}

fn extract_instance_data(instance: &Instance) -> Result<Option<InstanceData>> {
    if !instance.is_valid("type") {
        return Ok(None);
    }

    let obj_type = instance.get_i16("type") as i32;

    let (mesh_ref, collection_ref, collection_library_path) = match obj_type {
        OBJ_TYPE_MESH if instance.is_valid("data") => {
            let mesh_name =
                strip_blender_prefix(&instance.get("data").get("id").get_string("name"), "ME");
            println!("Instance: mesh type, mesh_name={}", mesh_name);
            (Some(mesh_name), None, None)
        }
        OBJ_TYPE_EMPTY => {
            println!("Instance: empty type");
            if instance.is_valid("instance_collection") {
                let coll = instance.get("instance_collection");
                println!("{:?}", coll);
                let collection_name = clean_blender_id(&coll, "CO");
                let lib_path = extract_library_path_from_id(&coll);
                println!(
                    "Instance: instance_collection, coll_name={}, lib_path={:?}",
                    collection_name, lib_path
                );
                (None, Some(collection_name), lib_path)
            } else if instance.is_valid("dup_group") {
                let dup = instance.get("dup_group");
                if dup.is_valid("name") {
                    let collection_name = strip_blender_prefix(&dup.get_string("name"), "GR");
                    println!("Collection name: {}", collection_name);
                    let lib_path = extract_library_path(&dup);
                    println!(
                        "Instance: dup_group, coll_name={}, lib_path={:?}",
                        collection_name, lib_path
                    );
                    (None, Some(collection_name), lib_path)
                } else {
                    println!("Instance: dup_group with no name");
                    (None, None, None)
                }
            } else {
                println!("Instance: empty type");
                (None, None, None)
            }
        }
        _ => {
            println!("Instance: unhandled type={}", obj_type);
            return Ok(None);
        }
    };

    let transform = extract_transform(instance);

    Ok(Some(InstanceData {
        mesh_ref,
        collection_ref,
        collection_library_path,
        transform,
    }))
}

/// Build the scene graph from collections and instances
fn build_scene_graph(
    scene: &mut MScene,
    collections: Vec<CollectionData>,
    instances: Vec<InstanceData>,
    mesh_id_prefix: Option<&str>,
    linked_scenes: &[(String, MScene)],
) -> Result<()> {
    // Build a lookup map for collections from this file
    let mut collection_map: HashMap<String, CollectionData> = HashMap::new();
    for collection in collections {
        collection_map.insert(collection.name.clone(), collection);
    }

    // Build a lookup map for linked scenes
    let mut linked_scene_map: HashMap<String, &MScene> = HashMap::new();
    for (lib_path, linked_scene) in linked_scenes {
        linked_scene_map.insert(lib_path.clone(), linked_scene);
    }

    // Process instances at the root level (not in collections)
    // For now, we'll add all instances to root and handle collection instances specially
    println!("Instance count: {}", instances.len());
    for instance_data in instances {
        match (&instance_data.mesh_ref, &instance_data.collection_ref) {
            (Some(mesh_name), None) => {
                // Direct mesh instance
                let mesh_id = if let Some(prefix) = mesh_id_prefix {
                    format!("{}{}", prefix, mesh_name)
                } else {
                    mesh_name.clone()
                };

                scene.root.children.push(MNode::MInstance(MInstance {
                    geometry_id: mesh_id,
                    material_id: None,
                    transform: Some(instance_data.transform),
                }));
            }
            (None, Some(collection_name)) => {
                // Collection instance - check if it's from a linked file
                match &instance_data.collection_library_path {
                    Some(lib_path) => {
                        // This is a linked collection - find it in the linked scene by name
                        let Some(linked_scene) = linked_scene_map.get(lib_path) else {
                            panic!(
                                "Linked library '{}' not found for collection '{}'",
                                lib_path, collection_name
                            );
                        };

                        let Some(matching_group) = linked_scene.root.children.iter().find_map(|node| {
                            if let MNode::MGroup(group) = node {
                                if group.name.as_ref() == Some(collection_name) {
                                    return Some(group);
                                } else {
                                    panic!(
                                        "Warning: Collection '{}' not found in linked library '{}'",
                                        collection_name, lib_path
                                    );
                                }
                            }
                            None
                        }) else {
                            panic!(
                                "Warning: Collection '{}' not found in linked library '{}'",
                                collection_name, lib_path
                            );
                        };

                        let mut instance_group = matching_group.clone();
                        instance_group.transform = Some(instance_data.transform);

                        merge_meshes_from_nodes(
                            &instance_group.children,
                            &linked_scene.meshes,
                            &mut scene.meshes,
                        );

                        scene.root.children.push(MNode::MGroup(instance_group));
                    }
                    None => {
                        // Local collection - build from local collection data
                        if let Some(collection_data) = collection_map.get(collection_name) {
                            let group = build_group_from_collection(
                                collection_data,
                                &collection_map,
                                Some(instance_data.transform),
                                mesh_id_prefix,
                            )?;
                            scene.root.children.push(MNode::MGroup(group));
                        } else {
                            eprintln!("Collection ref: {}", collection_name);
                            eprintln!("Collection map: {:?}", collection_map.keys());
                            panic!(
                                "Collection '{}' not found in main file for instance",
                                collection_name
                            );
                        }
                    }
                }
            }
            (None, None) => {
                panic!("Instance has neither mesh nor collection reference");
            }
            (Some(_), Some(_)) => {
                panic!("Instance has both mesh and collection reference");
            }
        }
    }

    Ok(())
}

/// Recursively collect mesh IDs from nodes and merge them from source if not already present
fn merge_meshes_from_nodes(
    nodes: &[MNode],
    source_meshes: &HashMap<MMeshID, MMesh>,
    target_meshes: &mut HashMap<MMeshID, MMesh>,
) {
    for node in nodes {
        match node {
            MNode::MInstance(instance) => {
                // If this mesh isn't already in the target, copy it from source
                if !target_meshes.contains_key(&instance.geometry_id) {
                    if let Some(mesh) = source_meshes.get(&instance.geometry_id) {
                        target_meshes.insert(instance.geometry_id.clone(), mesh.clone());
                    }
                }
            }
            MNode::MGroup(group) => {
                // Recursively process children
                merge_meshes_from_nodes(&group.children, source_meshes, target_meshes);
            }
        }
    }
}

/// Build a MGroup from a CollectionData, recursively
fn build_group_from_collection(
    collection: &CollectionData,
    collection_map: &HashMap<String, CollectionData>,
    transform: Option<MTransform>,
    mesh_id_prefix: Option<&str>,
) -> Result<MGroup> {
    let mut children = Vec::new();

    // Add mesh instances
    for mesh_name in &collection.mesh_children {
        let mesh_id = if let Some(prefix) = mesh_id_prefix {
            format!("{}{}", prefix, mesh_name)
        } else {
            mesh_name.clone()
        };

        children.push(MNode::MInstance(MInstance {
            geometry_id: mesh_id,
            material_id: None,
            transform: None,
        }));
    }

    // Add child collections
    for child_collection_name in &collection.collection_children {
        if let Some(child_collection) = collection_map.get(child_collection_name) {
            let child_group = build_group_from_collection(
                child_collection,
                collection_map,
                None,
                mesh_id_prefix,
            )?;
            children.push(MNode::MGroup(child_group));
        }
    }

    Ok(MGroup {
        name: None,
        children,
        transform,
    })
}

// Helper functions for extracting data from blend file instances

/// Strip Blender ID prefix from a name (e.g., "MECube" -> "Cube")
fn strip_blender_prefix(name: &str, prefix: &str) -> String {
    if name.len() > 2 && name.starts_with(prefix) {
        name[2..].to_string()
    } else {
        name.to_string()
    }
}

/// Extract clean object name from Blender ID field
fn clean_blender_id(instance: &Instance, expected_prefix: &str) -> String {
    let name = instance.get("id").get_string("name");
    strip_blender_prefix(&name, expected_prefix)
}

/// Extract library filepath from an Instance that may have a lib reference
fn extract_library_path(instance: &Instance) -> Option<String> {
    if !instance.is_valid("lib") {
        return None;
    }
    let lib = instance.get("lib");
    let path = if lib.is_valid("filepath") {
        lib.get_string("filepath")
    } else if lib.is_valid("name") {
        lib.get_string("name")
    } else {
        return None;
    };

    if !path.is_empty() {
        return Some(path);
    }
    None
}

/// Extract library path from an ID field
fn extract_library_path_from_id(instance: &Instance) -> Option<String> {
    if instance.is_valid("id") && instance.get("id").is_valid("lib") {
        extract_library_path(&instance.get("id"))
    } else {
        None
    }
}

/// Extract mesh name from an object instance
fn extract_mesh_from_object(object: &Instance) -> Option<String> {
    if !object.is_valid("type") {
        return None;
    }

    let obj_type = object.get_i16("type") as i32;
    if obj_type != OBJ_TYPE_MESH || !object.is_valid("data") {
        return None;
    }

    let mesh_data = object.get("data");
    let mesh_name = mesh_data.get("id").get_string("name");
    Some(strip_blender_prefix(&mesh_name, "ME"))
}

/// Iterator over mesh children in a collection's gobject list
fn extract_mesh_children(instance: &Instance) -> Vec<String> {
    let mut mesh_children = Vec::new();

    if !instance.is_valid("gobject") {
        return mesh_children;
    }

    let mut current = instance.get("gobject");
    loop {
        if current.is_valid("ob") {
            if let Some(mesh_name) = extract_mesh_from_object(&current.get("ob")) {
                mesh_children.push(mesh_name);
            }
        }

        if !current.is_valid("next") {
            break;
        }
        current = current.get("next");
    }

    mesh_children
}

/// Extract transform data from an instance
fn extract_transform(instance: &Instance) -> MTransform {
    let translation = extract_vec3(instance, "loc", Vec3::ZERO);
    let rotation = extract_vec3(instance, "rot", Vec3::ZERO);
    let scale =
        extract_vec3(instance, "scale", Vec3::ONE).max(extract_vec3(instance, "size", Vec3::ONE));

    MTransform {
        translation,
        rotation,
        scale,
    }
}

/// Extract a Vec3 from an instance field
fn extract_vec3(instance: &Instance, field: &str, default: Vec3) -> Vec3 {
    if !instance.is_valid(field) {
        return default;
    }

    let vec = instance.get_f32_vec(field);
    if vec.len() >= 3 {
        Vec3::new(vec[0], vec[1], vec[2])
    } else {
        default
    }
}

/// Recursively extract collections from a Scene's master_collection hierarchy
fn extract_collections_from_hierarchy(
    collection_instance: &Instance,
    collections: &mut Vec<CollectionData>,
) -> Result<()> {
    // Extract the collection itself
    let collection_data = if collection_instance.is_valid("id") {
        let id_name = collection_instance.get("id").get_string("name");
        // Determine if it's a GR or CO block by the prefix
        if id_name.starts_with("GR") {
            extract_group_data(collection_instance)?
        } else if id_name.starts_with("CO") {
            extract_collection_data(collection_instance)?
        } else {
            // Fallback: treat as collection
            extract_collection_data(collection_instance)?
        }
    } else {
        return Ok(());
    };

    collections.push(collection_data);

    // Recursively process children
    if collection_instance.is_valid("children") {
        for child in collection_instance.get_iter("children") {
            if child.is_valid("collection") {
                let child_coll = child.get("collection");
                extract_collections_from_hierarchy(&child_coll, collections)?;
            }
        }
    }

    Ok(())
}
