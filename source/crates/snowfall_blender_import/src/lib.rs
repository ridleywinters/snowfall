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
    let data =
        std::fs::read(path).with_context(|| format!("Failed to read file: {}", path.display()))?;
    load_from_memory(&data)
}

/// Load mesh data from a .blend file and resolve all linked collections
pub fn load_from_file_with_links<P: AsRef<Path>>(path: P) -> Result<BlendFile> {
    let path = path.as_ref();
    load_from_file_with_links_internal(path, None)
}

fn load_from_file_with_links_internal<P: AsRef<Path>>(
    path: P,
    mesh_id_prefix: Option<&str>,
) -> Result<BlendFile> {
    let path = path.as_ref();
    let data =
        std::fs::read(path).with_context(|| format!("Failed to read file: {}", path.display()))?;
    let base_path = path.parent().unwrap_or_else(|| Path::new("."));

    // First, scan the file for linked library references
    let library_paths = extract_library_paths(&data)?;

    // Load all linked libraries first
    let mut linked_scenes = Vec::new();
    for lib_path in library_paths {
        let resolved_path = if lib_path.starts_with("//") {
            base_path.join(&lib_path[2..])
        } else {
            Path::new(&lib_path).to_path_buf()
        };

        // Extract filename for mesh ID prefix
        let filename = resolved_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let prefix = format!("{}::", filename);

        // Load the library file with prefixed mesh IDs
        let lib_blend = load_from_file_with_links_internal(&resolved_path, Some(&prefix))
            .with_context(|| format!("Failed to load linked file: {}", resolved_path.display()))?;

        linked_scenes.push((lib_path, lib_blend.scene));
    }

    // Now parse the main file with knowledge of linked content
    let blend_file = load_from_memory_with_linked_content(&data, mesh_id_prefix, &linked_scenes)?;

    Ok(blend_file)
}

/// Extract library paths from OB blocks that reference linked collections
fn extract_library_paths(data: &[u8]) -> Result<Vec<String>> {
    let blend_file = Blend::new(Cursor::new(data))
        .map_err(|e| anyhow::anyhow!("Failed to parse .blend file: {:?}", e))?;

    let mut lib_paths = std::collections::HashSet::new();

    // Check OB blocks for dup_group references to linked collections
    for instance in blend_file.instances_with_code(*b"OB") {
        if instance.is_valid("dup_group") {
            let dup = instance.get("dup_group");

            // Check if dup_group has a library reference
            if dup.is_valid("lib") {
                let lib = dup.get("lib");

                // Try different field names for the library path
                let lib_path = if lib.is_valid("filepath") {
                    lib.get_string("filepath")
                } else if lib.is_valid("name") {
                    lib.get_string("name")
                } else {
                    continue;
                };

                if !lib_path.is_empty() {
                    lib_paths.insert(lib_path);
                }
            }
        }

        // Also check instance_collection for modern Blender files
        if instance.is_valid("instance_collection") {
            let coll = instance.get("instance_collection");
            if coll.is_valid("id") && coll.get("id").is_valid("lib") {
                let lib = coll.get("id").get("lib");
                let lib_path = if lib.is_valid("filepath") {
                    lib.get_string("filepath")
                } else {
                    continue;
                };

                if !lib_path.is_empty() {
                    lib_paths.insert(lib_path);
                }
            }
        }
    }

    // Also check CO and GR blocks for linked collections
    for instance in blend_file.instances_with_code(*b"CO") {
        if instance.get("id").is_valid("lib") {
            let lib = instance.get("id").get("lib");
            if lib.is_valid("filepath") {
                let lib_path = lib.get_string("filepath");
                if !lib_path.is_empty() {
                    lib_paths.insert(lib_path);
                }
            }
        }
    }

    for instance in blend_file.instances_with_code(*b"GR") {
        if instance.is_valid("id") && instance.get("id").is_valid("lib") {
            let lib = instance.get("id").get("lib");
            if lib.is_valid("filepath") {
                let lib_path = lib.get_string("filepath");
                if !lib_path.is_empty() {
                    lib_paths.insert(lib_path);
                }
            }
        }
    }

    Ok(lib_paths.into_iter().collect())
}

/// Load mesh data from in-memory .blend file data
pub fn load_from_memory(data: &[u8]) -> Result<BlendFile> {
    load_from_memory_with_linked_content(data, None, &[])
}

fn load_from_memory_with_linked_content(
    data: &[u8],
    mesh_id_prefix: Option<&str>,
    linked_scenes: &[(String, MScene)],
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
            children: Vec::new(),
            transform: None,
        },
    };

    // Extract all meshes from this file
    for instance in blend_file.instances_with_code(*b"ME") {
        let (mesh_id, mesh) = extract_mesh_data(&instance, mesh_id_prefix)
            .with_context(|| "Failed to extract mesh data")?;
        scene.meshes.insert(mesh_id, mesh);
    }

    // Extract collections
    let mut collections = Vec::new();
    for instance in blend_file.instances_with_code(*b"CO") {
        let collection =
            extract_collection_data(&instance).with_context(|| "Failed to extract collection")?;
        collections.push(collection);
    }

    // Also extract GR (Group) blocks from older Blender versions
    for instance in blend_file.instances_with_code(*b"GR") {
        let collection =
            extract_group_data(&instance).with_context(|| "Failed to extract group")?;
        collections.push(collection);
    }

    // Extract instances
    let mut instances = Vec::new();
    for instance in blend_file.instances_with_code(*b"OB") {
        if let Some(instance_data) = extract_instance_data(&instance)? {
            instances.push(instance_data);
        }
    }

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
    })
}

/// Extract mesh data from a blend file instance
fn extract_mesh_data(
    instance: &Instance,
    mesh_id_prefix: Option<&str>,
) -> Result<(MMeshID, MMesh)> {
    let name = instance.get("id").get_string("name");

    let clean_name = if name.len() > 2 && name.starts_with("ME") {
        name[2..].to_string()
    } else {
        name
    };

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
    let name = instance.get("id").get_string("name");

    let clean_name = if name.len() > 2 && name.starts_with("CO") {
        name[2..].to_string()
    } else {
        name
    };

    let mut mesh_children = Vec::new();
    let mut collection_children = Vec::new();

    if instance.is_valid("gobject") {
        let mut current = instance.get("gobject");

        loop {
            if current.is_valid("ob") {
                let object = current.get("ob");

                if object.is_valid("type") {
                    let obj_type = object.get_i16("type") as i32;

                    if obj_type == 1 && object.is_valid("data") {
                        let mesh_data = object.get("data");
                        let mesh_name = mesh_data.get("id").get_string("name");
                        let clean_mesh_name = if mesh_name.len() > 2 && mesh_name.starts_with("ME")
                        {
                            mesh_name[2..].to_string()
                        } else {
                            mesh_name
                        };
                        mesh_children.push(clean_mesh_name);
                    }
                }
            }

            if !current.is_valid("next") {
                break;
            }
            current = current.get("next");
        }
    }

    if instance.is_valid("children") {
        for child_coll_instance in instance.get_iter("children") {
            if child_coll_instance.is_valid("collection") {
                let child_coll = child_coll_instance.get("collection");
                let child_name = child_coll.get("id").get_string("name");
                let clean_child_name = if child_name.len() > 2 && child_name.starts_with("CO") {
                    child_name[2..].to_string()
                } else {
                    child_name
                };
                collection_children.push(clean_child_name);
            }
        }
    }

    Ok(CollectionData {
        name: clean_name,
        mesh_children,
        collection_children,
    })
}

fn extract_group_data(instance: &Instance) -> Result<CollectionData> {
    let name = if instance.is_valid("id") {
        instance.get("id").get_string("name")
    } else {
        "Unknown".to_string()
    };

    let clean_name = if name.len() > 2 && name.starts_with("GR") {
        name[2..].to_string()
    } else {
        name
    };

    let mut mesh_children = Vec::new();

    if instance.is_valid("gobject") {
        let mut current = instance.get("gobject");

        loop {
            if current.is_valid("ob") {
                let object = current.get("ob");

                if object.is_valid("type") {
                    let obj_type = object.get_i16("type") as i32;

                    if obj_type == 1 && object.is_valid("data") {
                        let mesh_data = object.get("data");
                        let mesh_name = mesh_data.get("id").get_string("name");
                        let clean_mesh_name = if mesh_name.len() > 2 && mesh_name.starts_with("ME")
                        {
                            mesh_name[2..].to_string()
                        } else {
                            mesh_name
                        };
                        mesh_children.push(clean_mesh_name);
                    }
                }
            }

            if !current.is_valid("next") {
                break;
            }
            current = current.get("next");
        }
    }

    Ok(CollectionData {
        name: clean_name,
        mesh_children,
        collection_children: Vec::new(),
    })
}

fn extract_instance_data(instance: &Instance) -> Result<Option<InstanceData>> {
    if !instance.is_valid("type") {
        return Ok(None);
    }

    let obj_type = instance.get_i16("type") as i32;

    let (mesh_ref, collection_ref, collection_library_path) =
        if obj_type == 1 && instance.is_valid("data") {
            let mesh_data = instance.get("data");
            let mesh_name = mesh_data.get("id").get_string("name");
            let clean_mesh_name = if mesh_name.len() > 2 && mesh_name.starts_with("ME") {
                mesh_name[2..].to_string()
            } else {
                mesh_name
            };
            (Some(clean_mesh_name), None, None)
        } else if instance.is_valid("instance_collection") {
            let coll = instance.get("instance_collection");
            let coll_name = coll.get("id").get_string("name");
            let clean_coll_name = if coll_name.len() > 2 && coll_name.starts_with("CO") {
                coll_name[2..].to_string()
            } else {
                coll_name
            };

            // Check if collection is linked
            let lib_path = if coll.is_valid("id") && coll.get("id").is_valid("lib") {
                let lib = coll.get("id").get("lib");
                if lib.is_valid("filepath") {
                    Some(lib.get_string("filepath"))
                } else {
                    None
                }
            } else {
                None
            };

            (None, Some(clean_coll_name), lib_path)
        } else if instance.is_valid("dup_group") {
            // Older Blender versions use dup_group (Group/GR blocks) instead of instance_collection
            let dup = instance.get("dup_group");
            if dup.is_valid("name") {
                let coll_name = dup.get_string("name");
                let clean_coll_name = if coll_name.len() > 2 && coll_name.starts_with("GR") {
                    coll_name[2..].to_string()
                } else {
                    coll_name
                };

                // Check if dup_group is linked
                let lib_path = if dup.is_valid("lib") {
                    let lib = dup.get("lib");
                    if lib.is_valid("filepath") {
                        Some(lib.get_string("filepath"))
                    } else if lib.is_valid("name") {
                        Some(lib.get_string("name"))
                    } else {
                        None
                    }
                } else {
                    None
                };

                (None, Some(clean_coll_name), lib_path)
            } else {
                (None, None, None)
            }
        } else if obj_type == 0 {
            // Empty object
            (None, None, None)
        } else {
            return Ok(None);
        };

    let position = if instance.is_valid("loc") {
        let loc = instance.get_f32_vec("loc");
        if loc.len() >= 3 {
            Vec3::new(loc[0], loc[1], loc[2])
        } else {
            Vec3::ZERO
        }
    } else {
        Vec3::ZERO
    };

    let rotation = if instance.is_valid("rot") {
        let rot = instance.get_f32_vec("rot");
        if rot.len() >= 3 {
            Vec3::new(rot[0], rot[1], rot[2])
        } else {
            Vec3::ZERO
        }
    } else {
        Vec3::ZERO
    };

    let scale = if instance.is_valid("scale") {
        let s = instance.get_f32_vec("scale");
        if s.len() >= 3 {
            Vec3::new(s[0], s[1], s[2])
        } else {
            Vec3::ONE
        }
    } else if instance.is_valid("size") {
        let s = instance.get_f32_vec("size");
        if s.len() >= 3 {
            Vec3::new(s[0], s[1], s[2])
        } else {
            Vec3::ONE
        }
    } else {
        Vec3::ONE
    };

    Ok(Some(InstanceData {
        mesh_ref,
        collection_ref,
        collection_library_path,
        transform: MTransform {
            translation: position,
            rotation,
            scale,
        },
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
    for instance_data in instances {
        if let Some(mesh_name) = instance_data.mesh_ref {
            // Direct mesh instance
            let mesh_id = if let Some(prefix) = mesh_id_prefix {
                format!("{}{}", prefix, mesh_name)
            } else {
                mesh_name
            };

            scene.root.children.push(MNode::MInstance(MInstance {
                geometry_id: mesh_id,
                material_id: None,
                transform: Some(instance_data.transform),
            }));
        } else if let Some(collection_name) = instance_data.collection_ref {
            // Collection instance - check if it's from a linked file
            if let Some(lib_path) = &instance_data.collection_library_path {
                // This is a linked collection - get it from the linked scene
                if let Some(linked_scene) = linked_scene_map.get(lib_path) {
                    // Clone the entire linked scene's root as a group with this instance's transform
                    // Then merge only the meshes that are actually used
                    let mut group = (*linked_scene).root.clone();
                    group.transform = Some(instance_data.transform);

                    // Merge only meshes referenced in this group
                    merge_meshes_from_nodes(
                        &group.children,
                        &linked_scene.meshes,
                        &mut scene.meshes,
                    );

                    scene.root.children.push(MNode::MGroup(group));
                }
            } else {
                // Local collection - build from local collection data
                if let Some(collection_data) = collection_map.get(&collection_name) {
                    let group = build_group_from_collection(
                        collection_data,
                        &collection_map,
                        Some(instance_data.transform),
                        mesh_id_prefix,
                    )?;
                    scene.root.children.push(MNode::MGroup(group));
                }
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
        children,
        transform,
    })
}
