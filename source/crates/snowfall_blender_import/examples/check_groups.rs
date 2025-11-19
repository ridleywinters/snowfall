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

    println!("\n=== GR (Group) blocks ===");
    let mut gr_count = 0;
    for instance in blend_file.instances_with_code(*b"GR") {
        gr_count += 1;
        
        println!("\nGroup #{}:", gr_count);
        println!("  has 'id': {}", instance.is_valid("id"));
        println!("  has 'name': {}", instance.is_valid("name"));
        
        if instance.is_valid("id") {
            let name = instance.get("id").get_string("name");
            println!("  Name from id: \"{}\"", name);
            
            if instance.get("id").is_valid("lib") {
                println!("  IS LINKED!");
                let lib = instance.get("id").get("lib");
                if lib.is_valid("filepath") {
                    println!("  From: {}", lib.get_string("filepath"));
                }
            }
        }
        
        if instance.is_valid("gobject") {
            println!("  Has gobject field");
        }
    }
    println!("\nTotal GR blocks: {}", gr_count);

    Ok(())
}
