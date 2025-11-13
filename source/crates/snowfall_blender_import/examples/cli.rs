use anyhow::Result;
use snowfall_blender_import::load_from_file;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <blender-file.blend>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    println!("Loading: {}", filename);

    let blend_file = load_from_file(filename)?;

    println!("\n=== File Information ===");
    println!("Blender Version: {}", blend_file.version_string());
    println!(
        "Format: {}",
        if blend_file.is_modern_format() {
            "Modern (4.x+)"
        } else {
            "Legacy (2.x/3.x)"
        }
    );
    println!("Pointer Size: {:?}", blend_file.pointer_size);
    println!("Endianness: {:?}", blend_file.endianness);
    println!("Meshes Found: {}", blend_file.meshes.len());

    for (i, mesh) in blend_file.meshes.iter().enumerate() {
        println!("\n=== Mesh #{} ===", i + 1);
        print_mesh_info(mesh);
    }

    Ok(())
}

fn print_mesh_info(mesh: &snowfall_blender_import::Mesh) {
    println!("  Name: \"{}\"", mesh.name);
    println!("  Vertices: {}", mesh.vertex_count());
    println!("  Triangles: {}", mesh.triangle_count());
    println!("  Normals: {}", mesh.normals.len());
    println!("  UVs: {}", mesh.uvs.len());
    println!("  Indices: {}", mesh.indices.len());

    if !mesh.positions.is_empty() {
        println!("  Sample vertex positions:");
        for (i, pos) in mesh.positions.iter().take(3).enumerate() {
            println!("    [{}] ({:.6}, {:.6}, {:.6})", i, pos.x, pos.y, pos.z);
        }
        if mesh.positions.len() > 3 {
            println!("    ... and {} more", mesh.positions.len() - 3);
        }
    }

    if !mesh.normals.is_empty() {
        println!("  Sample normals:");
        for (i, normal) in mesh.normals.iter().take(3).enumerate() {
            println!(
                "    [{}] ({:.6}, {:.6}, {:.6})",
                i, normal.x, normal.y, normal.z
            );
        }
        if mesh.normals.len() > 3 {
            println!("    ... and {} more", mesh.normals.len() - 3);
        }
    }

    if !mesh.uvs.is_empty() {
        println!("  Sample UVs:");
        for (i, uv) in mesh.uvs.iter().take(3).enumerate() {
            println!("    [{}] ({:.6}, {:.6})", i, uv.x, uv.y);
        }
        if mesh.uvs.len() > 3 {
            println!("    ... and {} more", mesh.uvs.len() - 3);
        }
    }

    if mesh.indices.len() >= 3 {
        println!("  Sample triangle indices:");
        println!(
            "    Triangle 0: [{}, {}, {}]",
            mesh.indices[0], mesh.indices[1], mesh.indices[2]
        );
        if mesh.triangle_count() > 1 {
            println!("    ... and {} more triangles", mesh.triangle_count() - 1);
        }
    }
}
