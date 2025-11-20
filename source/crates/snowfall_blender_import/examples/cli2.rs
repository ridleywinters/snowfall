use anyhow::{Context, Result};
use blend::{Blend, Instance};
use glam::Vec3;
use snowfall_blender_import::{MGroup, MInstance, MNode, MTransform};
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
    pub assets: HashMap<String, ()>,
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
    pub linked_libraries: Vec<BlendLibrary>,
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
        linked_libraries: Vec::new(),
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
        file.linked_libraries.push(lib);
    }

    for inst in blend.instances_with_code(*b"OB") {
        if let Some(node) = extract_object(&inst) {
            file.root.children.push(node);
        }
    }

    println!("\n=== File Information ===");
    println!(
        "Blender Version: {}.{}.{}",
        version[0] as char, version[1] as char, version[2] as char
    );
    println!("Pointer Size: {:?}", file.pointer_size);
    println!("Endianness: {:?}", file.endianness);

    println!("\n=== Linked Libraries ===");
    for lib in &file.linked_libraries {
        println!("* \"{}\"", lib.name);
        for (asset_id, _) in &lib.assets {
            println!("  Asset: {}", asset_id);
        }
    }

    println!("\n=== Scene Hierarchy ===");
    print_hierarchy(&file.root.children, 1);

    Ok(())
}

fn print_hierarchy(nodes: &[MNode], level: usize) {
    let indent_str = "    ".repeat(level);

    for node in nodes {
        match node {
            MNode::MInstance(instance) => {
                println!("{}MInstance:", indent_str);
                println!("{}  geometry: \"{}\"", indent_str, instance.geometry_id);
            }
            MNode::MGroup(group) => {
                println!("{}MGroup:", indent_str);
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
                    print_hierarchy(&group.children, level + 2);
                }
            }
        }
    }
}

fn extract_object(inst: &Instance) -> Option<MNode> {
    let obj_type = inst.get_i16("type") as i32;

    let transform = extract_transform(&inst);

    let name = match obj_type {
        0 => Some(load_empty(&inst)),
        _ => {
            return None;
        }
    };

    Some(MNode::MGroup(MGroup {
        name,
        transform: Some(transform),
        children: Vec::new(),
    }))
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

fn load_empty(inst: &Instance) -> String {
    let dup = inst.get("dup_group");
    let collection_name = dup.get_string("name");

    for (field, _ignored) in &dup.fields {
        println!("    Field: {}", field);
    }
    collection_name
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
        assets.insert(asset_id, ());
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
    })
}
