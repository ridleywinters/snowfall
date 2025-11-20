use snowfall_blender_import::load_from_file;

#[derive(serde::Deserialize)]
struct TestCase {
    name: String,
    file: String,
    expected: ExpectedData,
}

#[derive(serde::Deserialize)]
struct ExpectedData {
    version: String,
    pointer_size: String,
    endianness: String,
    meshes: Vec<ExpectedMesh>,
}

#[derive(serde::Deserialize)]
struct ExpectedMesh {
    name: String,
    vertex_count: usize,
    triangle_count: usize,
    positions: Vec<[f32; 3]>,
    first_triangle: [u32; 3],
}

#[test]
fn test_blend_files() {
    let test_data =
        std::fs::read_to_string("tests/test_cases.yaml").expect("Failed to read test_cases.yaml");
    let test_cases: Vec<TestCase> =
        serde_yaml::from_str(&test_data).expect("Failed to parse test_cases.yaml");

    for test_case in test_cases {
        let blend_file = load_from_file(&test_case.file)
            .unwrap_or_else(|_| panic!("Failed to load {}", test_case.file));

        assert_eq!(
            blend_file.version_string(),
            test_case.expected.version,
            "Version mismatch for {}",
            test_case.name
        );

        let pointer_size_str = format!("{:?}", blend_file.pointer_size);
        assert_eq!(
            pointer_size_str, test_case.expected.pointer_size,
            "Pointer size mismatch for {}",
            test_case.name
        );

        let endianness_str = format!("{:?}", blend_file.endianness);
        assert_eq!(
            endianness_str, test_case.expected.endianness,
            "Endianness mismatch for {}",
            test_case.name
        );

        assert_eq!(
            blend_file.scene.meshes.len(),
            test_case.expected.meshes.len(),
            "Mesh count mismatch for {}",
            test_case.name
        );

        for expected_mesh in &test_case.expected.meshes {
            let mesh = blend_file
                .scene
                .meshes
                .get(&expected_mesh.name)
                .unwrap_or_else(|| {
                    panic!(
                        "Mesh '{}' not found for {}",
                        expected_mesh.name, test_case.name
                    )
                });

            assert_eq!(
                mesh.id, expected_mesh.name,
                "Mesh name mismatch for {}",
                test_case.name
            );
            assert_eq!(
                mesh.vertex_count(),
                expected_mesh.vertex_count,
                "Vertex count mismatch for {} mesh '{}'",
                test_case.name,
                mesh.id
            );
            assert_eq!(
                mesh.triangle_count(),
                expected_mesh.triangle_count,
                "Triangle count mismatch for {} mesh '{}'",
                test_case.name,
                mesh.id
            );

            for (i, expected_pos) in expected_mesh.positions.iter().enumerate() {
                let pos = mesh.positions[i];
                assert!(
                    (pos.x - expected_pos[0]).abs() < 0.0001
                        && (pos.y - expected_pos[1]).abs() < 0.0001
                        && (pos.z - expected_pos[2]).abs() < 0.0001,
                    "Position mismatch at vertex {} for {} mesh '{}': expected ({}, {}, {}), got ({}, {}, {})",
                    i,
                    test_case.name,
                    mesh.id,
                    expected_pos[0],
                    expected_pos[1],
                    expected_pos[2],
                    pos.x,
                    pos.y,
                    pos.z
                );
            }

            assert_eq!(
                &mesh.indices[0..3],
                &expected_mesh.first_triangle,
                "First triangle indices mismatch for {} mesh '{}'",
                test_case.name,
                mesh.id
            );
        }
    }
}
