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

    // Find first Empty object and dump all its fields
    println!("\n=== Finding Empty objects and dumping fields ===");
    for instance in blend_file.instances_with_code(*b"OB") {
        let name = instance.get("id").get_string("name");
        
        if !instance.is_valid("type") {
            continue;
        }
        
        let obj_type = instance.get_i16("type");
        
        if obj_type == 0 && (name.contains("Floor") || name.contains("Character")) {
            println!("\n\nObject: \"{}\"", name);
            println!("Type: {}", obj_type);
            
            // Try to access the underlying Instance to see field info
            println!("\nTrying to read instance_collection with different approaches:");
            
            // Try exact field name
            if instance.is_valid("instance_collection") {
                println!("  ✓ instance_collection field EXISTS");
                let coll = instance.get("instance_collection");
                if coll.is_valid("id") {
                    let coll_name = coll.get("id").get_string("name");
                    println!("  Collection name: \"{}\"", coll_name);
                    
                    if coll.get("id").is_valid("lib") {
                        println!("  Collection IS LINKED!");
                        let lib = coll.get("id").get("lib");
                        if lib.is_valid("filepath") {
                            println!("  From: {}", lib.get_string("filepath"));
                        }
                    }
                }
            } else {
                println!("  ✗ instance_collection field does NOT exist");
            }
            
            // Check dup_group
            if instance.is_valid("dup_group") {
                println!("\n  ✓ dup_group field EXISTS");
                let dup = instance.get("dup_group");
                
                println!("    Checking dup_group structure:");
                println!("      has 'id': {}", dup.is_valid("id"));
                println!("      has 'name': {}", dup.is_valid("name"));
                
                // dup_group IS a Collection pointer, so it should have id
                if dup.is_valid("id") {
                    let coll_name = dup.get("id").get_string("name");
                    println!("      Collection name: \"{}\"", coll_name);
                    
                    if dup.get("id").is_valid("lib") {
                        println!("      Collection IS LINKED!");
                        let lib = dup.get("id").get("lib");
                        if lib.is_valid("filepath") {
                            println!("      From: {}", lib.get_string("filepath"));
                        }
                    }
                } else {
                    println!("      (dup_group doesn't have id field - might be direct pointer)");
                    
                    // Maybe it's a direct name?
                    if dup.is_valid("name") {
                        let name = dup.get_string("name");
                        println!("      Direct name: \"{}\"", name);
                    }
                }
            }
            
            // Try variations
            let variations = vec![
                "instance_collection",
                "instancecollection", 
                "dup_group",
                "dupli_group",
                "group",
            ];
            
            println!("\nTrying field name variations:");
            for variant in variations {
                if instance.is_valid(variant) {
                    println!("  ✓ Found field: \"{}\"", variant);
                } else {
                    println!("  ✗ No field: \"{}\"", variant);
                }
            }
            
            // Only check first matching object
            break;
        }
    }

    Ok(())
}
