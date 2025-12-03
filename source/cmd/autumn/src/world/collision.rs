use bevy::prelude::*;
use std::collections::HashMap;

pub const PLAYER_RADIUS: f32 = 8.0 * 0.2;

#[derive(Resource)]
pub struct CollisionMap {
    grid: HashMap<(i32, i32), bool>,
    width: usize,
    height: usize,
}

impl CollisionMap {
    pub fn new(grid: HashMap<(i32, i32), bool>, width: usize, height: usize) -> Self {
        Self {
            grid,
            width,
            height,
        }
    }

    pub fn is_solid(&self, grid_x: i32, grid_y: i32) -> bool {
        // Check bounds first
        if grid_x < 0 || grid_y < 0 || grid_x >= self.width as i32 || grid_y >= self.height as i32 {
            return true; // Treat out of bounds as solid
        }
        // If not in map, treat as empty (false)
        *self.grid.get(&(grid_x, grid_y)).unwrap_or(&false)
    }

    // Checks if the world position is occupied for a given aligned 2d
    // bounding box.
    pub fn can_move_to(&self, world_x: f32, world_y: f32, half_size: f32) -> bool {
        let min_x = world_x - half_size;
        let max_x = world_x + half_size;
        let min_y = world_y - half_size;
        let max_y = world_y + half_size;

        // Calculate grid cell range that the bounding box overlaps
        // Check all cells that any part of the box could touch
        let min_grid_x = (min_x / 8.0).floor() as i32;
        let max_grid_x = (max_x / 8.0).floor() as i32;
        let min_grid_y = (min_y / 8.0).floor() as i32;
        let max_grid_y = (max_y / 8.0).floor() as i32;

        // Check if any of the cells the bounding box overlaps is solid
        for grid_y in min_grid_y..=max_grid_y {
            for grid_x in min_grid_x..=max_grid_x {
                if self.is_solid(grid_x, grid_y) {
                    return false;
                }
            }
        }
        true
    }
}

/// Check if two positions in the XY plane are within a given radius of each other
pub fn check_circle_collision(pos1: Vec3, pos2: Vec3, radius: f32) -> bool {
    let dx = pos1.x - pos2.x;
    let dy = pos1.y - pos2.y;
    let distance_squared = dx * dx + dy * dy;
    distance_squared <= radius * radius
}
