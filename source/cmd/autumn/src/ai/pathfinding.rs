use crate::world::Map;
use pathfinding::prelude::astar;

const GRID_SIZE: f32 = 8.0;

/// Convert world coordinates to grid coordinates
pub fn world_to_grid(world_x: f32, world_y: f32) -> (i32, i32) {
    (
        (world_x / GRID_SIZE).floor() as i32,
        (world_y / GRID_SIZE).floor() as i32,
    )
}

/// Convert grid coordinates to world coordinates (center of grid cell)
pub fn grid_to_world(grid_x: i32, grid_y: i32) -> (f32, f32) {
    (
        grid_x as f32 * GRID_SIZE + GRID_SIZE / 2.0,
        grid_y as f32 * GRID_SIZE + GRID_SIZE / 2.0,
    )
}

/// Find a path from start to goal using A* pathfinding
/// Returns a list of world positions to follow
pub fn find_path(
    map: &Map,
    start_x: f32,
    start_y: f32,
    goal_x: f32,
    goal_y: f32,
) -> Option<Vec<(f32, f32)>> {
    let start_grid = world_to_grid(start_x, start_y);
    let goal_grid = world_to_grid(goal_x, goal_y);

    // Check if goal is walkable
    if map.is_solid(goal_grid.0, goal_grid.1) {
        return None;
    }

    let result = astar(
        &start_grid,
        |&(x, y)| {
            // Generate neighbors (4-directional movement)
            let mut neighbors = Vec::new();
            for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let nx = x + dx;
                let ny = y + dy;
                if !map.is_solid(nx, ny) {
                    neighbors.push(((nx, ny), 1));
                }
            }
            neighbors
        },
        |&(x, y)| {
            // Manhattan distance heuristic
            ((x - goal_grid.0).abs() + (y - goal_grid.1).abs()) as u32
        },
        |&pos| pos == goal_grid,
    );

    result.map(|(path, _cost)| {
        // Convert grid path to world positions
        path.into_iter()
            .map(|(gx, gy)| grid_to_world(gx, gy))
            .collect()
    })
}
