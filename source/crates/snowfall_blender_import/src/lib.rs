use anyhow::{Context, Result};
use blend::{Blend, Instance};
use glam::{Vec2, Vec3};
use std::io::Cursor;
use std::path::Path;

/// A Blender file containing mesh data and metadata
#[derive(Debug, Clone)]
pub struct BlendFile {
    /// All meshes found in the file
    pub meshes: Vec<Mesh>,
    /// Blender version that created this file as ASCII bytes (e.g., ['4', '0', '5'] for Blender 4.0.5)
    pub version: [u8; 3],
    /// Pointer size used in the file (32-bit or 64-bit)
    pub pointer_size: PointerSize,
    /// Endianness of the file
    pub endianness: Endianness,
}

/// Pointer size in the blend file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerSize {
    /// 32-bit pointers
    Bits32,
    /// 64-bit pointers
    Bits64,
}

/// Byte order in the blend file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    /// Little-endian (most modern systems)
    Little,
    /// Big-endian
    Big,
}

impl BlendFile {
    /// Get the Blender version as a string (e.g., "4.0.5")
    pub fn version_string(&self) -> String {
        format!(
            "{}.{}.{}",
            self.version[0] as char, self.version[1] as char, self.version[2] as char
        )
    }

    /// Check if this file was created with Blender 4.x or newer
    pub fn is_modern_format(&self) -> bool {
        self.version[0] >= b'4'
    }
}

/// A mesh extracted from a Blender file.
/// This structure is compatible with Bevy's mesh format but doesn't depend on Bevy.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// The name of the mesh (as defined in Blender)
    pub name: String,
    /// Vertex positions
    pub positions: Vec<Vec3>,
    /// Vertex normals
    pub normals: Vec<Vec3>,
    /// Texture coordinates (UVs)
    pub uvs: Vec<Vec2>,
    /// Triangle indices (groups of 3)
    pub indices: Vec<u32>,
}

impl Mesh {
    /// Create a new empty mesh with the given name
    pub fn new(name: String) -> Self {
        Self {
            name,
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Get the number of vertices in this mesh
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Get the number of triangles in this mesh
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// Load mesh data from a .blend file
pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<BlendFile> {
    let path = path.as_ref();
    let data =
        std::fs::read(path).with_context(|| format!("Failed to read file: {}", path.display()))?;
    load_from_memory(&data)
}

/// Load mesh data from in-memory .blend file data
pub fn load_from_memory(data: &[u8]) -> Result<BlendFile> {
    let blend_file = Blend::new(Cursor::new(data))
        .map_err(|e| anyhow::anyhow!("Failed to parse .blend file: {:?}", e))?;

    let header = &blend_file.blend.header;
    let version = header.version;

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

    Ok(BlendFile {
        meshes,
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

    if instance.is_valid("vdata") {
        return extract_mesh_data_modern(instance, mesh);
    }

    if !instance.is_valid("mvert") {
        return Ok(mesh);
    }

    extract_mesh_data_legacy(instance, mesh)
}

/// Extract mesh data using Blender 4.x CustomData format
fn extract_mesh_data_modern(instance: &Instance, mut mesh: Mesh) -> Result<Mesh> {
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

    Ok(mesh)
}

/// Extract mesh data using legacy Blender 2.x/3.x format  
fn extract_mesh_data_legacy(instance: &Instance, mut mesh: Mesh) -> Result<Mesh> {
    let verts: Vec<_> = instance.get_iter("mvert").collect();

    for (_i, vert) in verts.iter().enumerate() {
        let co = vert.get_f32_vec("co");
        if co.len() >= 3 {
            mesh.positions.push(Vec3::new(co[0], co[1], co[2]));
        }

        let no = vert.get_i16_vec("no");
        if no.len() >= 3 {
            let nx = no[0] as f32 / 32767.0;
            let ny = no[1] as f32 / 32767.0;
            let nz = no[2] as f32 / 32767.0;
            mesh.normals.push(Vec3::new(nx, ny, nz));
        }
    }

    if !instance.is_valid("mpoly") || !instance.is_valid("mloop") {
        return Ok(mesh);
    }

    let polys: Vec<_> = instance.get_iter("mpoly").collect();
    let loops: Vec<_> = instance.get_iter("mloop").collect();

    let uvs_exist = instance.is_valid("mloopuv");
    let uv_loops: Vec<_> = if uvs_exist {
        instance.get_iter("mloopuv").collect()
    } else {
        Vec::new()
    };

    for (_poly_idx, poly) in polys.iter().enumerate() {
        let loopstart = poly.get_i32("loopstart") as usize;
        let totloop = poly.get_i32("totloop") as usize;

        let mut poly_indices = Vec::new();
        for i in 0..totloop {
            let loop_index = loopstart + i;
            if loop_index < loops.len() {
                let v = loops[loop_index].get_i32("v") as u32;
                poly_indices.push(v);

                if uvs_exist && loop_index < uv_loops.len() {
                    let uv = uv_loops[loop_index].get_f32_vec("uv");
                    if uv.len() >= 2 {
                        mesh.uvs.push(Vec2::new(uv[0], uv[1]));
                    }
                }
            }
        }

        for i in 1..poly_indices.len().saturating_sub(1) {
            mesh.indices.push(poly_indices[0]);
            mesh.indices.push(poly_indices[i]);
            mesh.indices.push(poly_indices[i + 1]);
        }
    }

    Ok(mesh)
}
