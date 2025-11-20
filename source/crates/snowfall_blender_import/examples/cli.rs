use anyhow::Result;
use snowfall_blender_import::{MNode, load_from_file_with_links};
use std::env;

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
    println!("Meshes Found: {}", blend_file.scene.meshes.len());

    println!("\n=== Meshes ===");
    for (mesh_id, mesh) in &blend_file.scene.meshes {
        println!(
            "  \"{}\": {} vertices, {} triangles",
            mesh_id,
            mesh.vertex_count(),
            mesh.triangle_count()
        );
    }

    println!("\n=== Scene Hierarchy ===");
    print_hierarchy(&blend_file.scene.root.children, 0, &blend_file.scene.meshes);

    Ok(())
}

fn print_hierarchy(
    nodes: &[MNode],
    indent: usize,
    meshes: &std::collections::HashMap<String, snowfall_blender_import::MMesh>,
) {
    let indent_str = "  ".repeat(indent);

    for node in nodes {
        match node {
            MNode::MInstance(instance) => {
                println!("{}MInstance:", indent_str);
                println!("{}  geometry: \"{}\"", indent_str, instance.geometry_id);

                if let Some(mesh) = meshes.get(&instance.geometry_id) {
                    println!(
                        "{}    ({} vertices, {} triangles)",
                        indent_str,
                        mesh.vertex_count(),
                        mesh.triangle_count()
                    );
                }

                if let Some(transform) = &instance.transform {
                    println!("{}  transform:", indent_str);
                    println!(
                        "{}    position: [{:.3}, {:.3}, {:.3}]",
                        indent_str,
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z
                    );
                    println!(
                        "{}    rotation: [{:.3}, {:.3}, {:.3}]",
                        indent_str,
                        transform.rotation.x,
                        transform.rotation.y,
                        transform.rotation.z
                    );
                    println!(
                        "{}    scale: [{:.3}, {:.3}, {:.3}]",
                        indent_str, transform.scale.x, transform.scale.y, transform.scale.z
                    );
                }
            }
            MNode::MGroup(group) => {
                println!("{}MGroup: ({} children)", indent_str, group.children.len());

                if let Some(transform) = &group.transform {
                    println!("{}  transform:", indent_str);
                    println!(
                        "{}    position: [{:.3}, {:.3}, {:.3}]",
                        indent_str,
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z
                    );
                    println!(
                        "{}    rotation: [{:.3}, {:.3}, {:.3}]",
                        indent_str,
                        transform.rotation.x,
                        transform.rotation.y,
                        transform.rotation.z
                    );
                    println!(
                        "{}    scale: [{:.3}, {:.3}, {:.3}]",
                        indent_str, transform.scale.x, transform.scale.y, transform.scale.z
                    );
                }

                print_hierarchy(&group.children, indent + 1, meshes);
            }
        }
    }
}
