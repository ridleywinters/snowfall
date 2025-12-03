use super::*;

#[test]
fn test_load_map_file_succeeds() {
    let result = Map::load_map_file("data/map.yaml");
    
    assert!(
        result.is_ok(),
        "Should successfully load data/map.yaml: {:?}",
        result.err()
    );
    
    let map_file = result.unwrap();
    
    // Verify grid is non-empty
    assert!(
        !map_file.grid.is_empty(),
        "Map grid should not be empty"
    );
    
    // Verify grid is rectangular
    let expected_width = map_file.grid[0].len();
    for (idx, row) in map_file.grid.iter().enumerate() {
        assert_eq!(
            row.len(),
            expected_width,
            "Grid row {} has inconsistent width: expected {}, got {}",
            idx,
            expected_width,
            row.len()
        );
    }
}

#[test]
fn test_from_map_file_creates_valid_map() {
    // Load the actual map file
    let map_file = Map::load_map_file("data/map.yaml")
        .expect("Should load map.yaml for testing");
    
    // Create Map from MapFile
    let map = Map::from_map_file(&map_file);
    
    // Verify dimensions match
    let expected_height = map_file.grid.len() as i32;
    let expected_width = if expected_height > 0 {
        map_file.grid[0].len() as i32
    } else {
        0
    };
    
    assert_eq!(map.width, expected_width, "Map width should match grid width");
    assert_eq!(map.height, expected_height, "Map height should match grid height");
    
    // Verify collision grid was populated
    assert!(
        !map.collision_grid.is_empty(),
        "Collision grid should be populated from map data"
    );
    
    // Verify at least some walls exist
    let wall_count = map.collision_grid.values()
        .filter(|tile| matches!(tile, TileType::Wall { .. }))
        .count();
    
    assert!(
        wall_count > 0,
        "Map should contain at least some walls"
    );
}

#[test]
fn test_wall_height_parsing() {
    // Create a simple test map
    let test_map_file = MapFile {
        grid: vec![
            "XXx ".to_string(),
            "X x.".to_string(),
        ],
        items: vec![],
        actors: vec![],
    };
    
    let map = Map::from_map_file(&test_map_file);
    
    // Check that 'X' creates tall walls (height 16.0)
    assert_eq!(
        map.collision_grid.get(&(0, 0)),
        Some(&TileType::Wall { height: 16.0 }),
        "Uppercase 'X' should create wall with height 16.0"
    );
    
    // Check that 'x' creates short walls (height 8.0)
    assert_eq!(
        map.collision_grid.get(&(2, 0)),
        Some(&TileType::Wall { height: 8.0 }),
        "Lowercase 'x' should create wall with height 8.0"
    );
    
    // Check that space creates empty tile
    assert_eq!(
        map.collision_grid.get(&(3, 0)),
        Some(&TileType::Empty),
        "Space should create empty tile"
    );
    
    // Check that '.' creates empty tile
    assert_eq!(
        map.collision_grid.get(&(3, 1)),
        Some(&TileType::Empty),
        "Period should create empty tile"
    );
}

#[test]
fn test_is_solid() {
    let test_map_file = MapFile {
        grid: vec![
            "X ".to_string(),
        ],
        items: vec![],
        actors: vec![],
    };
    
    let map = Map::from_map_file(&test_map_file);
    
    assert!(map.is_solid(0, 0), "Wall tile should be solid");
    assert!(!map.is_solid(1, 0), "Empty tile should not be solid");
}
