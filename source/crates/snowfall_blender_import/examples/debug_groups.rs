use anyhow::Result;
use blend::Blend;
use std::env;
use std::io::Cursor;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <blender-file.blend>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    println!("Loading: {}", filename);

    let data = std::fs::read(filename)?;
    let blend_file = Blend::new(Cursor::new(data))
        .map_err(|e| anyhow::anyhow!("Failed to parse blend file: {:?}", e))?;

    println!("\n=== GR (Group) blocks with gobject traversal ===");
    for instance in blend_file.instances_with_code(*b"GR") {
        let name = instance.get("id").get_string("name");
        println!("\nGroup: {}", name);
        
        if instance.is_valid("gobject") {
            println!("  Has gobject field");
            let mut current = instance.get("gobject");
            let mut count = 0;
            
            loop {
                count += 1;
                if count > 100 {
                    println!("  ERROR: Too many iterations, breaking");
                    break;
                }
                
                println!("  Node #{}", count);
                println!("    has 'ob': {}", current.is_valid("ob"));
                println!("    has 'next': {}", current.is_valid("next"));
                
                if current.is_valid("ob") {
                    let object = current.get("ob");
                    println!("    ob has 'type': {}", object.is_valid("type"));
                    
                    if object.is_valid("type") {
                        let obj_type = object.get_i16("type") as i32;
                        println!("    obj_type: {}", obj_type);
                        
                        if obj_type == 1 && object.is_valid("data") {
                            let mesh_data = object.get("data");
                            println!("    mesh_data has 'id': {}", mesh_data.is_valid("id"));
                            if mesh_data.is_valid("id") {
                                let mesh_name = mesh_data.get("id").get_string("name");
                                println!("    Mesh: {}", mesh_name);
                            }
                        }
                    }
                }

                if !current.is_valid("next") {
                    println!("  End of list (no 'next')");
                    break;
                }
                current = current.get("next");
            }
        } else {
            println!("  No gobject field");
        }
    }

    Ok(())
}
