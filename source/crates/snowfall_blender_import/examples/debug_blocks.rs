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

    // Show some details about ME blocks
    println!("\n=== ME (Mesh) blocks ===");
    let mut me_count = 0;
    let mut linked_meshes = 0;
    for instance in blend_file.instances_with_code(*b"ME") {
        me_count += 1;
        let name = instance.get("id").get_string("name");

        let is_linked = instance.get("id").is_valid("lib");
        if is_linked {
            linked_meshes += 1;
            println!("\nLinked Mesh: \"{}\"", name);
            let lib = instance.get("id").get("lib");
            if lib.is_valid("filepath") {
                println!("  From: {}", lib.get_string("filepath"));
            }
        }
    }
    println!("\nTotal ME blocks: {}", me_count);
    println!("Linked meshes: {}", linked_meshes);

    // Show some details about CO blocks
    println!("\n=== CO (Collection) blocks ===");
    let mut co_count = 0;
    for instance in blend_file.instances_with_code(*b"CO") {
        co_count += 1;
        if instance.get("id").is_valid("name") {
            let name = instance.get("id").get_string("name");
            println!("Collection: \"{}\"", name);

            if instance.get("id").is_valid("lib") {
                println!("  - Linked!");
                let lib = instance.get("id").get("lib");
                if lib.is_valid("filepath") {
                    println!("  - From: {}", lib.get_string("filepath"));
                }
            }
        }
    }
    println!("Total CO blocks: {}", co_count);

    // Show some details about OB blocks
    println!("\n=== OB (Object) blocks ===");
    let mut ob_count = 0;
    let mut empty_with_collection = 0;
    for instance in blend_file.instances_with_code(*b"OB") {
        ob_count += 1;

        let name = instance.get("id").get_string("name");
        let obj_type = if instance.is_valid("type") {
            instance.get_i16("type")
        } else {
            -1
        };

        if obj_type == 0 && ob_count <= 5 {
            // Empty object - check all possible linking mechanisms
            println!("\nEmpty Object: \"{}\"", name);

            // Check for linked object itself
            if instance.get("id").is_valid("lib") {
                println!("  - Object itself is LINKED!");
                let lib = instance.get("id").get("lib");
                if lib.is_valid("filepath") {
                    println!("  - From: {}", lib.get_string("filepath"));
                }
            }

            if instance.is_valid("instance_collection") {
                empty_with_collection += 1;
                println!("  - HAS instance_collection!");
                let coll = instance.get("instance_collection");
                if coll.is_valid("id") && coll.get("id").is_valid("name") {
                    let coll_name = coll.get("id").get_string("name");
                    println!("  - Collection: \"{}\"", coll_name);

                    if coll.get("id").is_valid("lib") {
                        println!("  - Collection is LINKED!");
                        let lib = coll.get("id").get("lib");
                        if lib.is_valid("filepath") {
                            println!("  - From: {}", lib.get_string("filepath"));
                        }
                    }
                }
            }

            // Check for proxy
            if instance.is_valid("proxy") {
                println!("  - HAS proxy!");
            }

            // Check for proxy_group (old linking method)
            if instance.is_valid("proxy_group") {
                println!("  - HAS proxy_group!");
            }

            // Check for duplicator
            if instance.is_valid("duplicator") {
                println!("  - HAS duplicator!");
            }

            // Check for data field
            if instance.is_valid("data") {
                println!("  - HAS data field!");
                let data = instance.get("data");
                if data.is_valid("id") {
                    let data_name = data.get("id").get_string("name");
                    println!("  - Data: \"{}\"", data_name);
                }
            }

            // Check for parent
            if instance.is_valid("parent") {
                println!("  - HAS parent!");
                let parent = instance.get("parent");
                if parent.is_valid("id") {
                    let parent_name = parent.get("id").get_string("name");
                    println!("  - Parent: \"{}\"", parent_name);
                }
            }

            // Check if it has children by checking if any object has this as parent
            println!("  - Checking structure...");
        }
    }
    println!("\nTotal OB blocks: {}", ob_count);
    println!(
        "Empty objects with instance_collection: {}",
        empty_with_collection
    );

    Ok(())
}
