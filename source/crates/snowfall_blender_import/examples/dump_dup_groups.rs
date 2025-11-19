use anyhow::Result;
use blend::Blend;
use std::collections::HashSet;
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

    println!("\n=== dup_group references ===");
    let mut seen_names = HashSet::new();
    
    for instance in blend_file.instances_with_code(*b"OB") {
        if instance.is_valid("dup_group") {
            let dup = instance.get("dup_group");
            if dup.is_valid("name") {
                let name = dup.get_string("name");
                if seen_names.insert(name.clone()) {
                    println!("  dup_group name: \"{}\"", name);
                    println!("    has 'id': {}", dup.is_valid("id"));
                    println!("    has 'lib': {}", dup.is_valid("lib"));
                    
                    // Check if it's linked
                    if dup.is_valid("lib") {
                        println!("    IS LINKED (direct lib)!");
                        let lib = dup.get("lib");
                        println!("    lib has 'filepath': {}", lib.is_valid("filepath"));
                        println!("    lib has 'filepath_abs': {}", lib.is_valid("filepath_abs"));
                        println!("    lib has 'name': {}", lib.is_valid("name"));
                        
                        if lib.is_valid("filepath") {
                            println!("    filepath: {}", lib.get_string("filepath"));
                        }
                        if lib.is_valid("filepath_abs") {
                            println!("    filepath_abs: {}", lib.get_string("filepath_abs"));
                        }
                        if lib.is_valid("name") {
                            println!("    lib name: {}", lib.get_string("name"));
                        }
                    }
                    
                    if dup.is_valid("id") && dup.get("id").is_valid("lib") {
                        println!("    IS LINKED (via id)!");
                        let lib = dup.get("id").get("lib");
                        if lib.is_valid("filepath") {
                            println!("    From: {}", lib.get_string("filepath"));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
