# snowfall_blender_import

A Rust crate for loading mesh geometry from Blender `.blend` files.

## Features

- Load mesh data from `.blend` files
- Extract vertex positions, normals, UV coordinates, and triangle indices
- Support for both file and in-memory data loading
- Bevy-compatible mesh structure (uses `glam` for math types)
- No Bevy dependency required
- Error handling with `anyhow::Result`
- **Supports both Blender 4.x (modern CustomData format) and Blender 2.x/3.x (legacy format)**
- **Includes file metadata: version, pointer size, endianness**

## Usage

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
snowfall_blender_import = "0.1.0"
```

### Loading from a file

```rust
use snowfall_blender_import::load_from_file;

fn main() -> anyhow::Result<()> {
    let blend_file = load_from_file("my_model.blend")?;

    println!("Blender Version: {}", blend_file.version_string());
    println!("Format: {}", if blend_file.is_modern_format() {
        "Modern (4.x+)"
    } else {
        "Legacy (2.x/3.x)"
    });

    for mesh in &blend_file.meshes {
        println!("Mesh: {}", mesh.name);
        println!("  Vertices: {}", mesh.vertex_count());
        println!("  Triangles: {}", mesh.triangle_count());
    }

    Ok(())
}
```

### Loading from memory

```rust
use snowfall_blender_import::load_from_memory;

fn main() -> anyhow::Result<()> {
    let data = std::fs::read("my_model.blend")?;
    let blend_file = load_from_memory(&data)?;

    // Process meshes...
    for mesh in &blend_file.meshes {
        println!("{}: {} vertices", mesh.name, mesh.vertex_count());
    }

    Ok(())
}
```

### Example CLI Tool

Run the included example to inspect a `.blend` file:

```bash
cargo run --example cli -- path/to/your/file.blend
```

This will print detailed information about the file and all meshes, including:

- Blender version, format type, pointer size, and endianness
- Mesh names
- Vertex and triangle counts
- Sample vertex positions
- Sample normals
- Sample UV coordinates (if present)
- Sample triangle indices

## Data Structures

### BlendFile

The `BlendFile` struct contains:

- `meshes: Vec<Mesh>` - All meshes found in the file
- `version: [u8; 3]` - Blender version as ASCII bytes (e.g., ['4', '0', '5'])
- `pointer_size: PointerSize` - 32-bit or 64-bit pointers
- `endianness: Endianness` - Little or big endian

Helper methods:

- `version_string()` - Get version as a string (e.g., "4.0.5")
- `is_modern_format()` - Returns true for Blender 4.x+

### Mesh

The `Mesh` struct contains:

- `name: String` - Name of the mesh in Blender
- `positions: Vec<Vec3>` - Vertex positions (xyz)
- `normals: Vec<Vec3>` - Vertex normals (normalized xyz)
- `uvs: Vec<Vec2>` - UV texture coordinates (optional)
- `indices: Vec<u32>` - Triangle indices (groups of 3)

All vector types (`Vec2`, `Vec3`) are from the `glam` crate, making the mesh data compatible with Bevy and other game engines.

## Limitations

- Only extracts mesh geometry (no materials, textures, animations, etc.)
- Polygons are triangulated using simple fan triangulation
- Does not support compressed `.blend` files

## Testing

To test the crate with a real `.blend` file:

1. Create a simple model in Blender (or use an existing `.blend` file)
2. Run the example CLI tool:
   ```bash
   cargo run --example cli -- path/to/your/file.blend
   ```

The example will print detailed information about the file format and all meshes found.
