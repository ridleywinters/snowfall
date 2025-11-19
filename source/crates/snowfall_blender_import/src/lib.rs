use anyhow::{Context, Result};
use blend::{Blend, Instance};
use glam::Vec3;
use std::io::Cursor;
use std::path::Path;

mod bbox;
pub use bbox::BBox;

mod transform_trs;
pub use transform_trs::TransformTRS;

mod instance;
pub use instance::MeshInstance;

mod mesh;
pub use mesh::Mesh;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectRef {
    Mesh(String),
    Collection(String),
    Empty,
}

/// A Blender file containing mesh data and metadata
#[derive(Debug, Clone)]
pub struct BlendFile {
    pub meshes: Vec<Mesh>,
    pub collections: Vec<Collection>,
    pub instances: Vec<MeshInstance>,
    /// Blender version that created this file as ASCII bytes (e.g., ['4', '0', '5'] for Blender 4.0.5)
    pub version: [u8; 3],
    pub pointer_size: PointerSize,
    pub endianness: Endianness,
}

impl BlendFile {
    /// Get the Blender version as a string (e.g., "4.0.5")
    pub fn version_string(&self) -> String {
        format!(
            "{}.{}.{}",
            self.version[0] as char, self.version[1] as char, self.version[2] as char
        )
    }

    /// Get a mapping of collection names to the mesh names they contain
    /// This is built by examining instances in the file
    pub fn collection_mesh_map(&self) -> std::collections::HashMap<String, Vec<String>> {
        let map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

        // Look at each instance and if it's in a collection (has a parent or is referenced),
        // we'd track it. For now, we'll use a simpler approach: assume top-level mesh instances
        // in a library file are what should be instantiated when the collection is referenced.

        // Since Blender doesn't explicitly store "this mesh is in this collection" for groups,
        // we'll return an empty map and rely on a different strategy
        map
    }
}

#[derive(Debug, Clone)]
pub struct Collection {
    pub name: String,
    pub is_linked: bool,
    pub library_path: Option<String>,
    pub children: Vec<ObjectRef>,
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
    let mut blend_file = load_from_file(path)?;

    let base_path = path.parent().unwrap_or_else(|| Path::new("."));

    // Collect all unique library paths
    let mut library_paths: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for collection in &blend_file.collections {
        if collection.is_linked {
            if let Some(lib_path) = &collection.library_path {
                library_paths
                    .entry(lib_path.clone())
                    .or_insert_with(Vec::new)
                    .push(collection.name.clone());
            }
        }
    }

    // Load each linked library and extract the referenced collections
    for (lib_path, collection_names) in library_paths {
        // Resolve relative path (Blender uses // prefix for relative paths)
        let resolved_path = if lib_path.starts_with("//") {
            base_path.join(&lib_path[2..])
        } else {
            Path::new(&lib_path).to_path_buf()
        };

        // Load the library file
        let lib_blend = match load_from_file(&resolved_path) {
            Ok(blend) => blend,
            Err(e) => {
                eprintln!(
                    "Warning: Failed to load linked library {:?}: {}",
                    resolved_path, e
                );
                continue;
            }
        };

        // Merge meshes from library (they may be referenced by collections)
        blend_file.meshes.extend(lib_blend.meshes.clone());

        // Merge instances from library - these define what's in each collection
        for lib_instance in lib_blend.instances.clone() {
            blend_file.instances.push(MeshInstance {
                source_file: Some(lib_path.clone()),
                ..lib_instance
            });
        }

        // Find and update the collections with their children
        for collection_name in collection_names {
            // Find the collection in the library
            if let Some(lib_collection) = lib_blend
                .collections
                .iter()
                .find(|c| c.name == collection_name)
            {
                // Update the collection in our main file
                if let Some(collection) = blend_file.collections.iter_mut().find(|c| {
                    c.name == collection_name
                        && c.is_linked
                        && c.library_path.as_ref() == Some(&lib_path)
                }) {
                    collection.children = lib_collection.children.clone();
                }
            }
        }
    }

    Ok(blend_file)
}

/// Load mesh data from in-memory .blend file data
pub fn load_from_memory(data: &[u8]) -> Result<BlendFile> {
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

    let mut meshes = Vec::new();

    for instance in blend_file.instances_with_code(*b"ME") {
        let mesh = extract_mesh_data(&instance).with_context(|| "Failed to extract mesh data")?;
        meshes.push(mesh);
    }

    let mut collections = Vec::new();
    for instance in blend_file.instances_with_code(*b"CO") {
        let collection =
            extract_collection(&instance).with_context(|| "Failed to extract collection")?;
        collections.push(collection);
    }

    // Also extract GR (Group) blocks from older Blender versions
    for instance in blend_file.instances_with_code(*b"GR") {
        let collection = extract_group(&instance).with_context(|| "Failed to extract group")?;
        collections.push(collection);
    }

    let mut instances = Vec::new();
    for instance in blend_file.instances_with_code(*b"OB") {
        if let Some(mut mesh_instance) = extract_instance(&instance)? {
            mesh_instance.source_file = None; // Main file
            instances.push(mesh_instance);
        }
    }

    // Extract linked collections from dup_group references
    let mut seen_linked = std::collections::HashSet::new();
    for instance in blend_file.instances_with_code(*b"OB") {
        if instance.is_valid("dup_group") {
            let dup = instance.get("dup_group");
            if dup.is_valid("name") && dup.is_valid("lib") {
                let name = dup.get_string("name");
                let clean_name = if name.len() > 2 && name.starts_with("GR") {
                    name[2..].to_string()
                } else {
                    name
                };

                let lib = dup.get("lib");
                let lib_path = if lib.is_valid("name") {
                    lib.get_string("name")
                } else {
                    String::new()
                };

                let key = (clean_name.clone(), lib_path.clone());
                if seen_linked.insert(key) && !lib_path.is_empty() {
                    collections.push(Collection {
                        name: clean_name,
                        is_linked: true,
                        library_path: Some(lib_path),
                        children: Vec::new(),
                    });
                }
            }
        }
    }

    Ok(BlendFile {
        meshes,
        collections,
        instances,
        version,
        pointer_size,
        endianness,
    })
}

/// Extract mesh data from a blend file instance
fn extract_mesh_data(instance: &Instance) -> Result<Mesh> {
    let name = instance.get("id").get_string("name");

    let clean_name = if name.len() > 2 && name.starts_with("ME") {
        name[2..].to_string()
    } else {
        name
    };

    let mesh = Mesh::new(clean_name);
    extract_mesh_data_v4(instance, mesh)
}

fn extract_mesh_data_v4(instance: &Instance, mut mesh: Mesh) -> Result<Mesh> {
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

fn extract_collection(instance: &Instance) -> Result<Collection> {
    let name = instance.get("id").get_string("name");

    let clean_name = if name.len() > 2 && name.starts_with("CO") {
        name[2..].to_string()
    } else {
        name
    };

    let is_linked = instance.get("id").is_valid("lib");
    let library_path = if is_linked {
        Some(instance.get("id").get("lib").get_string("filepath"))
    } else {
        None
    };

    let mut children = Vec::new();

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
                        children.push(ObjectRef::Mesh(clean_mesh_name));
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
                children.push(ObjectRef::Collection(clean_child_name));
            }
        }
    }

    Ok(Collection {
        name: clean_name,
        is_linked,
        library_path,
        children,
    })
}

fn extract_group(instance: &Instance) -> Result<Collection> {
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

    let is_linked = instance.is_valid("id") && instance.get("id").is_valid("lib");
    let library_path = if is_linked {
        Some(instance.get("id").get("lib").get_string("filepath"))
    } else {
        None
    };

    let mut children = Vec::new();

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
                        children.push(ObjectRef::Mesh(clean_mesh_name));
                    }
                }
            }

            if !current.is_valid("next") {
                break;
            }
            current = current.get("next");
        }
    }

    Ok(Collection {
        name: clean_name,
        is_linked,
        library_path,
        children,
    })
}

fn extract_instance(instance: &Instance) -> Result<Option<MeshInstance>> {
    let obj_name = instance.get("id").get_string("name");
    let clean_name = if obj_name.len() > 2 && obj_name.starts_with("OB") {
        obj_name[2..].to_string()
    } else {
        obj_name
    };

    if !instance.is_valid("type") {
        return Ok(None);
    }

    let obj_type = instance.get_i16("type") as i32;

    let target = if obj_type == 1 && instance.is_valid("data") {
        let mesh_data = instance.get("data");
        let mesh_name = mesh_data.get("id").get_string("name");
        let clean_mesh_name = if mesh_name.len() > 2 && mesh_name.starts_with("ME") {
            mesh_name[2..].to_string()
        } else {
            mesh_name
        };
        ObjectRef::Mesh(clean_mesh_name)
    } else if instance.is_valid("instance_collection") {
        let coll = instance.get("instance_collection");
        let coll_name = coll.get("id").get_string("name");
        let clean_coll_name = if coll_name.len() > 2 && coll_name.starts_with("CO") {
            coll_name[2..].to_string()
        } else {
            coll_name
        };
        ObjectRef::Collection(clean_coll_name)
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
            ObjectRef::Collection(clean_coll_name)
        } else {
            ObjectRef::Empty
        }
    } else if obj_type == 0 {
        // Empty object - include it so application can handle asset loading
        ObjectRef::Empty
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

    Ok(Some(MeshInstance {
        name: clean_name,
        target,
        transform: TransformTRS {
            translation: position,
            rotation,
            scale,
        },
        source_file: None, // Will be set by caller if from library
    }))
}
