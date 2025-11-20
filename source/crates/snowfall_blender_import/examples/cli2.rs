use anyhow::{Context, Result};
use blend::{Blend, Instance};
use glam::Vec3;
use snowfall_blender_import::{BBox, MGroup, MInstance, MLink, MMesh, MNode, MTransform};
use std::collections::HashMap;
use std::env;
use std::io::Cursor;
use std::path::Path;

fn main() -> Result<()> {
    println!("This is CLI example 2.");

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <blender-file.blend>", args[0]);
        std::process::exit(1);
    }
    let filename = &args[1];
    cmd_summary(filename)?;

    Ok(())
}

pub struct BlendLibrary {
    pub name: String,
    pub filepath: String,
    pub assets: HashMap<String, MNode>,
    pub meshes: HashMap<String, MMesh>,
}

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

pub struct BlendFile2 {
    pub version: [u8; 3],
    pub pointer_size: PointerSize,
    pub endianness: Endianness,

    pub libraries: Vec<BlendLibrary>,
    pub root: MGroup,
}

fn cmd_summary(filename: &str) -> Result<()> {
    println!("Loading: {}", filename);

    // First, scan for linked library files
    let path = Path::new(filename);
    let data = std::fs::read(&path) //
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    let blend = Blend::new(Cursor::new(&data))
        .map_err(|e| anyhow::anyhow!("Failed to parse .blend file: {:?}", e))?;

    let header = &blend.blend.header;
    let version = header.version;
    let pointer_size = match header.pointer_size {
        blend::parsers::PointerSize::Bits32 => PointerSize::Bits32,
        blend::parsers::PointerSize::Bits64 => PointerSize::Bits64,
    };

    let endianness = match header.endianness {
        blend::parsers::Endianness::Little => Endianness::Little,
        blend::parsers::Endianness::Big => Endianness::Big,
    };

    let mut file = BlendFile2 {
        version,
        pointer_size,
        endianness,
        libraries: Vec::new(),
        root: MGroup {
            name: None,
            transform: None,
            children: Vec::new(),
        },
    };

    for instance in blend.instances_with_code(*b"LI") {
        if !instance.is_valid("name") {
            panic!("LI instance missing 'name' property");
        }
        let lib = load_library(path, &instance)?;
        file.libraries.push(lib);
    }

    for inst in blend.instances_with_code(*b"OB") {
        if let Some(link) = extract_object(&inst) {
            file.root.children.push(MNode::MLink(link));
        }
    }

    print_summary(&file);

    Ok(())
}

fn print_summary(file: &BlendFile2) {
    println!("\n=== File Information ===");
    println!(
        "Blender Version: {}.{}.{}",
        file.version[0] as char, file.version[1] as char, file.version[2] as char
    );
    println!("Pointer Size: {:?}", file.pointer_size);
    println!("Endianness: {:?}", file.endianness);

    println!("\n=== Linked Libraries ===");
    for lib in &file.libraries {
        println!("* \"{}\"", lib.name);
        for (asset_id, root) in &lib.assets {
            println!("  Asset: {}", asset_id);
            print_hierarchy2(root, 2);
        }

        for (mesh_id, mesh) in &lib.meshes {
            println!(
                "  Mesh \"{}\": {} vertices, {} triangles",
                mesh_id,
                mesh.vertex_count(),
                mesh.triangle_count()
            );
        }
    }

    println!("\n=== Scene Hierarchy ===");
    print_hierarchy(&file.root.children, 1);
}

fn print_hierarchy2(node: &MNode, level: usize) {
    let indent_str = "    ".repeat(level);

    match node {
        MNode::MInstance(instance) => {
            if let Some(name) = &instance.name {
                println!("{}  name: \"{}\"", indent_str, name);
            }
            println!("{}  geometry: \"{}\"", indent_str, instance.geometry_id);
        }
        MNode::MLink(link) => {
            println!("{}  MLink: \"{}\" ({})", indent_str, link.id, link.library);
            if let Some(transform) = &link.transform {
                println!(
                    "{}    position: [{:.1}, {:.1}, {:.1}]",
                    indent_str,
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z,
                );
            }
        }
        MNode::MGroup(group) => {
            if let Some(name) = &group.name {
                println!("{}  name: \"{}\"", indent_str, name);
            }
            if let Some(transform) = &group.transform {
                println!(
                    "{}  position: [{:.1}, {:.1}, {:.1}]",
                    indent_str,
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z,
                );
            }
            if !group.children.is_empty() {
                println!("{}  children:", indent_str);
                print_hierarchy(&group.children, level + 1);
            }
        }
    }
}

fn print_hierarchy(nodes: &[MNode], level: usize) {
    for node in nodes {
        print_hierarchy2(node, level);
    }
}

fn extract_object(inst: &Instance) -> Option<MLink> {
    let obj_type = inst.get_i16("type") as i32;

    let transform = extract_transform(&inst);
    let (name, lib_name) = match obj_type {
        0 => load_empty(&inst),
        _ => {
            return None;
        }
    };

    Some(MLink {
        id: name,
        library: lib_name,
        transform: Some(transform),
    })
}

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

fn load_empty(inst: &Instance) -> (String, String) {
    let dup = inst.get("dup_group");
    let collection_name = dup.get_string("name");

    // Check if the dup_group (collection) is from a linked library

    let mut library_name = String::new();
    if dup.is_valid("lib") {
        let lib = dup.get("lib");
        if lib.is_valid("filepath") {
            let lib_path = lib.get_string("filepath");
            library_name = lib_path.clone();
        } else if lib.is_valid("name") {
            let lib_name = lib.get_string("name");
            library_name = lib_name.clone();
        }
    }
    (collection_name, library_name)
}

fn load_library(basepath: &Path, instance: &Instance) -> Result<BlendLibrary> {
    if !instance.is_valid("name") {
        panic!("LI instance missing 'name' property");
    }
    let filepath = instance.get_string("name");

    let path = if filepath.starts_with("//") {
        basepath.parent().unwrap().join(&filepath[2..])
    } else {
        Path::new(&filepath).to_path_buf()
    };

    let data = std::fs::read(&path) //
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    let blend = Blend::new(Cursor::new(&data))
        .map_err(|e| anyhow::anyhow!("Failed to parse .blend file: {:?}", e))?;

    let mut assets = HashMap::new();
    for inst in blend.instances_with_code(*b"GR") {
        let asset_id = inst.get("id").get_string("name");

        let gobj = load_group_object(&inst);
        if gobj.is_none() {
            continue;
        }
        assets.insert(asset_id, gobj.unwrap());
    }

    let mut meshes = HashMap::new();
    for inst in blend.instances_with_code(*b"ME") {
        let mesh = extract_mesh_data_v4(&inst);
        if let Ok(mesh) = mesh {
            meshes.insert(mesh.id.clone(), mesh);
        }
    }

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(BlendLibrary {
        name,
        filepath: path.to_string_lossy().to_string(),
        assets,
        meshes,
    })
}

fn load_group_object(inst: &Instance) -> Option<MNode> {
    if !inst.is_valid("gobject") {
        return None;
    }

    let mut meshes = Vec::new();
    for obj_it in inst.get_iter("gobject") {
        let obj = obj_it.get("ob");
        let obj_type = obj.get_i16("type") as i32;
        if obj_type != 1 {
            continue;
        }
        let mesh = obj.get("data");
        let mesh_name = mesh.get("id").get_string("name");

        let mmesh = MInstance {
            name: None,
            geometry_id: mesh_name.clone(),
            material_id: None,
            transform: None,
        };
        meshes.push(mmesh);
    }

    let node = match meshes.len() {
        0 => return None,
        1 => MNode::MInstance(meshes[0].clone()),
        _ => MNode::MGroup(MGroup {
            name: None,
            transform: None,
            children: meshes.into_iter().map(MNode::MInstance).collect(),
        }),
    };
    Some(node)
}

fn extract_mesh_data_v4(instance: &Instance) -> Result<MMesh> {
    let mut mesh = MMesh {
        id: instance.get("id").get_string("name"),
        positions: Vec::new(),
        normals: Vec::new(),
        uvs: Vec::new(),
        indices: Vec::new(),
        bbox: BBox::empty(),
    };

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
