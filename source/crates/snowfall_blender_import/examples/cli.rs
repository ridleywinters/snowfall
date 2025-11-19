use anyhow::Result;
use snowfall_blender_import::load_from_file_with_links;
use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone)]
struct FlatInstance {
    name: String,
    mesh_name: String,
    position: glam::Vec3,
    rotation: glam::Vec3,
    scale: glam::Vec3,
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <blender-file.blend>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    println!("Loading: {}", filename);

    let blend_file = load_from_file_with_links(filename)?;

    println!("\n=== File Information ===");
    println!("Blender Version: {}", blend_file.version_string());
    println!("Pointer Size: {:?}", blend_file.pointer_size);
    println!("Endianness: {:?}", blend_file.endianness);
    println!("Meshes Found: {}", blend_file.meshes.len());
    println!("Collections Found: {}", blend_file.collections.len());
    println!("Instances Found: {}", blend_file.instances.len());

    // Build mesh lookup
    let mesh_map: HashMap<String, &snowfall_blender_import::Mesh> = blend_file
        .meshes
        .iter()
        .map(|m| (m.name.clone(), m))
        .collect();

    // Build collection to meshes mapping by looking at instances in the library file
    // For now, we'll just flatten instances directly to meshes
    let flat_instances = flatten_instances(&blend_file);

    println!("\n=== Flattened Hierarchy ===");
    println!("Total mesh instances: {}", flat_instances.len());

    for (i, instance) in flat_instances.iter().enumerate() {
        println!("\nInstance #{}: \"{}\"", i + 1, instance.name);
        println!("  Mesh: \"{}\"", instance.mesh_name);

        if let Some(mesh) = mesh_map.get(&instance.mesh_name) {
            println!("    Vertices: {}", mesh.vertex_count());
            println!("    BBox: {:?} - {:?}", mesh.bbox.min, mesh.bbox.max);
            println!("    Size: {:?}", mesh.bbox.size());
        } else {
            println!("    (Mesh not found - may be in unloaded library)");
        }

        println!(
            "  Position: [{:.3}, {:.3}, {:.3}]",
            instance.position.x, instance.position.y, instance.position.z
        );
        println!(
            "  Rotation: [{:.3}, {:.3}, {:.3}]",
            instance.rotation.x, instance.rotation.y, instance.rotation.z
        );
        println!(
            "  Scale: [{:.3}, {:.3}, {:.3}]",
            instance.scale.x, instance.scale.y, instance.scale.z
        );
    }

    Ok(())
}

fn flatten_instances(blend_file: &snowfall_blender_import::BlendFile) -> Vec<FlatInstance> {
    let mut flat = Vec::new();

    // Build a map of library paths to mesh instances
    // These are the "template" instances that collections reference
    let mut library_instances: HashMap<String, Vec<&snowfall_blender_import::MeshInstance>> =
        HashMap::new();

    for instance in &blend_file.instances {
        if let Some(source) = &instance.source_file {
            library_instances
                .entry(source.clone())
                .or_insert_with(Vec::new)
                .push(instance);
        }
    }

    // Expand instances
    for instance in &blend_file.instances {
        // Skip library instances - they're templates, not scene placements
        if instance.source_file.is_some() {
            continue;
        }

        match &instance.target {
            snowfall_blender_import::ObjectRef::Mesh(mesh_name) => {
                // Direct mesh instance
                flat.push(FlatInstance {
                    name: instance.name.clone(),
                    mesh_name: mesh_name.clone(),
                    position: instance.transform.translation,
                    rotation: instance.transform.rotation,
                    scale: instance.transform.scale,
                });
            }
            snowfall_blender_import::ObjectRef::Collection(coll_name) => {
                // Find the collection to get its library path
                if let Some(collection) =
                    blend_file.collections.iter().find(|c| &c.name == coll_name)
                {
                    if let Some(lib_path) = &collection.library_path {
                        // Get all mesh instances from this library
                        if let Some(lib_insts) = library_instances.get(lib_path) {
                            // Create a flat instance for each mesh in the library
                            // applying this instance's transform
                            for lib_inst in lib_insts {
                                if let snowfall_blender_import::ObjectRef::Mesh(mesh_name) =
                                    &lib_inst.target
                                {
                                    // Combine transforms: apply library instance offset + main instance transform
                                    let combined_pos = instance.transform.translation
                                        + lib_inst.transform.translation;
                                    let combined_rot =
                                        instance.transform.rotation + lib_inst.transform.rotation;
                                    let combined_scale =
                                        instance.transform.scale * lib_inst.transform.scale;

                                    flat.push(FlatInstance {
                                        name: format!("{}:{}", instance.name, lib_inst.name),
                                        mesh_name: mesh_name.clone(),
                                        position: combined_pos,
                                        rotation: combined_rot,
                                        scale: combined_scale,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            snowfall_blender_import::ObjectRef::Empty => {
                // Skip
            }
        }
    }

    flat
}
