use crate::ai::pathfinding::{find_path, grid_to_world, world_to_grid};
use crate::world::{Map, TileType};
use std::collections::HashMap;

#[test]
fn test_world_to_grid_conversion() {
    assert_eq!(world_to_grid(0.0, 0.0), (0, 0));
    assert_eq!(world_to_grid(8.0, 8.0), (1, 1));
    assert_eq!(world_to_grid(7.9, 7.9), (0, 0));
    assert_eq!(world_to_grid(16.0, 24.0), (2, 3));
}

#[test]
fn test_grid_to_world_conversion() {
    assert_eq!(grid_to_world(0, 0), (4.0, 4.0)); // Center of cell
    assert_eq!(grid_to_world(1, 1), (12.0, 12.0));
    assert_eq!(grid_to_world(2, 3), (20.0, 28.0));
}

#[test]
fn test_pathfinding_simple_path() {
    // Create a simple 5x5 map with no walls
    let mut collision_grid = HashMap::new();
    for x in 0..5 {
        for y in 0..5 {
            collision_grid.insert((x, y), TileType::Empty);
        }
    }

    let map = Map {
        width: 5,
        height: 5,
        collision_grid,
        walls: HashMap::new(),
        items: HashMap::new(),
        item_world_positions: Vec::new(),
        actors: HashMap::new(),
    };

    // Find path from (4.0, 4.0) to (20.0, 20.0)
    let path = find_path(&map, 4.0, 4.0, 20.0, 20.0);
    assert!(path.is_some());

    let path = path.unwrap();
    assert!(!path.is_empty());
    // First position should be near start, last should be near goal
    assert_eq!(path[0], (4.0, 4.0));
    assert_eq!(path[path.len() - 1], (20.0, 20.0));
}

#[test]
fn test_pathfinding_blocked_destination() {
    // Create a 5x5 map with destination blocked
    let mut collision_grid = HashMap::new();
    for x in 0..5 {
        for y in 0..5 {
            collision_grid.insert((x, y), TileType::Empty);
        }
    }
    // Block the destination
    collision_grid.insert((2, 2), TileType::Wall { height: 1.0 });

    let map = Map {
        width: 5,
        height: 5,
        collision_grid,
        walls: HashMap::new(),
        items: HashMap::new(),
        item_world_positions: Vec::new(),
        actors: HashMap::new(),
    };

    // Try to find path to blocked location
    let path = find_path(&map, 4.0, 4.0, 20.0, 20.0);
    assert!(path.is_none());
}
