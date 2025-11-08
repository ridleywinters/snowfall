use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub mod editor;

use crate::actor::{ActorDefinitions, ActorPosition};
use crate::game_state::GamePlayEntity;
use crate::item::{Item, ItemDefinitions, ItemPosition};
use crate::rendering::Billboard;
use crate::texture_loader::{load_image_texture, load_weapon_texture};

/// Grid size for walls (8×8 grid)
const GRID_SIZE: f32 = 8.0;

/// Wrapper for YAML file format (has "map:" prefix)
#[derive(Deserialize)]
struct MapFileWrapper {
    map: MapFile,
}

/// Represents the type of tile at a grid position
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TileType {
    Empty,
    Wall { height: f32 },
}

/// Unified map resource that tracks walls, items, and actors
#[derive(Resource)]
pub struct Map {
    /// Grid dimensions
    pub width: i32,
    pub height: i32,

    /// Collision grid (8×8 grid aligned)
    pub collision_grid: HashMap<(i32, i32), TileType>,

    /// Entity tracking for walls (8×8 grid aligned)
    pub walls: HashMap<(i32, i32), Entity>,

    /// Entity tracking for items (2×2 grid aligned - intentionally different)
    pub items: HashMap<(i32, i32), Entity>,

    /// Item world positions with types for collision checks and saving
    pub item_world_positions: Vec<(Vec3, String)>,

    /// Entity tracking for actors
    pub actors: HashMap<Entity, ActorPosition>,
}

impl Map {
    /// Create a new empty map
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            collision_grid: HashMap::new(),
            walls: HashMap::new(),
            items: HashMap::new(),
            item_world_positions: Vec::new(),
            actors: HashMap::new(),
        }
    }

    /// Load and parse map YAML file (Bevy-independent)
    pub fn load_map_file(path: &str) -> Result<MapFile, String> {
        let map_path = PathBuf::from(path);

        let file_contents =
            fs::read_to_string(&map_path).map_err(|e| format!("Failed to read map file: {}", e))?;

        let wrapper: MapFileWrapper = serde_yaml::from_str(&file_contents)
            .map_err(|e| format!("Failed to parse map YAML: {}", e))?;

        Ok(wrapper.map)
    }

    /// Create Map structure from parsed MapFile (Bevy-independent)
    pub fn from_map_file(map_file: &MapFile) -> Self {
        let height = map_file.grid.len() as i32;
        let width = if height > 0 {
            map_file.grid[0].len() as i32
        } else {
            0
        };

        let mut map = Self::new(width, height);

        // Parse grid and populate collision data
        for (row_idx, row) in map_file.grid.iter().enumerate() {
            for (col_idx, ch) in row.chars().enumerate() {
                let grid_x = col_idx as i32;
                let grid_y = row_idx as i32;

                match ch {
                    '#' | 'X' => {
                        map.collision_grid
                            .insert((grid_x, grid_y), TileType::Wall { height: 16.0 });
                    }
                    'x' => {
                        map.collision_grid
                            .insert((grid_x, grid_y), TileType::Wall { height: 8.0 });
                    }
                    _ => {
                        map.collision_grid.insert((grid_x, grid_y), TileType::Empty);
                    }
                }
            }
        }

        map
    }

    /// Load map from YAML file and spawn all entities
    pub fn load_from_file(
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        item_defs: &ItemDefinitions,
        actor_defs: &ActorDefinitions,
    ) -> Result<Self, String> {
        let map_file = Self::load_map_file("data/map.yaml")?;
        let mut map = Self::from_map_file(&map_file);

        // Spawn wall entities
        for ((grid_x, grid_y), tile_type) in &map.collision_grid.clone() {
            if let TileType::Wall { height } = tile_type {
                map.spawn_wall(
                    commands,
                    asset_server,
                    meshes,
                    materials,
                    *grid_x,
                    *grid_y,
                    *height,
                );
            }
        }

        // Spawn items
        for item_pos in &map_file.items {
            map.spawn_item(
                commands,
                asset_server,
                meshes,
                materials,
                item_defs,
                item_pos.x,
                item_pos.y,
                &item_pos.item_type,
            );
        }

        // Spawn actors
        for actor_pos in &map_file.actors {
            map.spawn_actor(
                commands,
                asset_server,
                meshes,
                materials,
                actor_defs,
                actor_pos.x as f32,
                actor_pos.y as f32,
                &actor_pos.actor_type,
            );
        }

        Ok(map)
    }

    /// Create a billboard mesh oriented in the YZ plane (normal along X-axis)
    /// for use with the billboard rotation system
    fn create_billboard_mesh(scale: f32) -> Mesh {
        let mut billboard_mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        let positions = vec![
            [0.0, -scale, -scale], // bottom-left
            [0.0, scale, -scale],  // top-left
            [0.0, scale, scale],   // top-right
            [0.0, -scale, scale],  // bottom-right
        ];
        billboard_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

        billboard_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[1.0, 0.0, 0.0]; 4]);

        let uvs = vec![
            [0.0, 1.0], // top-left -> bottom-left in texture
            [1.0, 1.0], // top-right -> bottom-right in texture
            [1.0, 0.0], // bottom-right -> top-right in texture
            [0.0, 0.0], // bottom-left -> top-left in texture
        ];
        billboard_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        billboard_mesh.insert_indices(Indices::U32(vec![
            0, 1, 2, // first triangle
            0, 2, 3, // second triangle
        ]));

        billboard_mesh
    }

    /// Spawn a wall at the given grid position (8×8 grid)
    pub fn spawn_wall(
        &mut self,
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        grid_x: i32,
        grid_y: i32,
        wall_height: f32,
    ) {
        // Register in collision grid
        self.collision_grid.insert(
            (grid_x, grid_y),
            TileType::Wall {
                height: wall_height,
            },
        );

        // Spawn wall entity
        // Position at grid corner, then offset by half grid size to center the cuboid in the cell
        let world_x = grid_x as f32 * GRID_SIZE + GRID_SIZE / 2.0;
        let world_y = grid_y as f32 * GRID_SIZE + GRID_SIZE / 2.0;

        let texture_handle = load_image_texture(asset_server, "base/textures/stone_2.png");

        let entity = commands
            .spawn((
                GamePlayEntity,
                Mesh3d(meshes.add(Cuboid::new(GRID_SIZE, GRID_SIZE, wall_height))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(texture_handle),
                    ..default()
                })),
                Transform::from_xyz(world_x, world_y, wall_height / 2.0),
            ))
            .id();

        // Track entity
        self.walls.insert((grid_x, grid_y), entity);
    }

    /// Spawn an item at the given world position (uses 2×2 grid internally for tracking)
    pub fn spawn_item(
        &mut self,
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        item_defs: &ItemDefinitions,
        world_x: f32,
        world_y: f32,
        item_type: &str,
    ) {
        let Some(item_def) = item_defs.items.get(item_type) else {
            warn!("Unknown item type: {}", item_type);
            return;
        };

        // Items use 2×2 grid for tracking
        let grid_x = (world_x / 2.0).floor() as i32;
        let grid_y = (world_y / 2.0).floor() as i32;
        let world_pos = Vec3::new(world_x, world_y, item_def.scale);

        let texture_handle = load_weapon_texture(asset_server, &item_def.image);

        let entity = commands
            .spawn((
                GamePlayEntity,
                Billboard,
                Item {
                    interaction_radius: 2.0,
                },
                Mesh3d(meshes.add(Self::create_billboard_mesh(item_def.scale))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(texture_handle),
                    alpha_mode: AlphaMode::Blend,
                    unlit: false,
                    ..default()
                })),
                Transform::from_translation(world_pos),
            ))
            .id();

        // Track entity and world position
        self.items.insert((grid_x, grid_y), entity);
        self.item_world_positions
            .push((world_pos, item_type.to_string()));
    }

    /// Spawn an actor at the given world position
    pub fn spawn_actor(
        &mut self,
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        actor_defs: &ActorDefinitions,
        world_x: f32,
        world_y: f32,
        actor_type: &str,
    ) {
        let Some(actor_def) = actor_defs.actors.get(actor_type) else {
            warn!("Unknown actor type: {}", actor_type);
            return;
        };

        let world_pos = Vec3::new(world_x, world_y, actor_def.scale);
        let texture_handle = load_weapon_texture(asset_server, &actor_def.sprite);

        // Create behavior based on definition
        let behavior: Option<Box<dyn crate::ai::ActorBehavior>> = match actor_def.behavior.as_str()
        {
            "wander" => Some(Box::new(crate::ai::wander_behavior::WanderBehavior::new())),
            "stand" => Some(Box::new(crate::ai::stand_behavior::StandBehavior)),
            "aggressive" => Some(Box::new(
                crate::ai::aggressive_behavior::AggressiveBehavior::new(),
            )),
            _ => {
                warn!(
                    "Unknown behavior type: {}, defaulting to wander",
                    actor_def.behavior
                );
                Some(Box::new(crate::ai::wander_behavior::WanderBehavior::new()))
            }
        };

        let entity = commands
            .spawn((
                GamePlayEntity,
                Billboard,
                crate::actor::Actor {
                    actor_type: actor_type.to_string(),
                    health: actor_def.max_health,
                    max_health: actor_def.max_health,
                    scale: actor_def.scale,
                    armor: 0,
                    physical_resistance: 0.0,
                    actor_radius: 1.2, // 3/4 of player radius (1.6)
                    speed_multiplier: actor_def.speed,
                    behavior,
                    is_moving: false,
                    base_z: actor_def.scale,
                    attack_damage: actor_def.attack_damage,
                    attack_range: actor_def.attack_range,
                    attack_cooldown: actor_def.attack_cooldown,
                    attack_timer: 0.0,
                    stun_timer: 0.0,
                    attack_state: crate::actor::ActorAttackState::Idle,
                },
                Mesh3d(meshes.add(Self::create_billboard_mesh(actor_def.scale))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(texture_handle),
                    alpha_mode: AlphaMode::Blend,
                    unlit: false,
                    ..default()
                })),
                Transform::from_translation(world_pos),
            ))
            .id();

        // Track entity
        self.actors.insert(
            entity,
            ActorPosition {
                x: world_x,
                y: world_y,
                actor_type: actor_type.to_string(),
            },
        );
    }

    /// Remove a wall at the given grid position
    pub fn remove_wall(&mut self, commands: &mut Commands, grid_x: i32, grid_y: i32) -> bool {
        // Update collision grid immediately
        self.collision_grid
            .insert((grid_x, grid_y), TileType::Empty);

        // Despawn entity if it exists
        if let Some(entity) = self.walls.remove(&(grid_x, grid_y)) {
            commands.entity(entity).despawn();
            true
        } else {
            false
        }
    }

    /// Unregister an item entity (when picked up or removed)
    pub fn unregister_item(&mut self, grid_x: i32, grid_y: i32) {
        self.items.remove(&(grid_x, grid_y));

        // Remove from world positions
        let world_x = grid_x as f32 * 2.0;
        let world_y = grid_y as f32 * 2.0;

        self.item_world_positions.retain(|(pos, _)| {
            let dx = pos.x - world_x;
            let dy = pos.y - world_y;
            dx * dx + dy * dy > 0.01
        });
    }

    /// Unregister an actor entity (when it dies or is removed)
    pub fn unregister_actor(&mut self, entity: Entity) {
        self.actors.remove(&entity);
    }

    /// Get item at the given grid position (2×2 grid)
    pub fn get_item_at(&self, grid_x: i32, grid_y: i32) -> Option<Entity> {
        self.items.get(&(grid_x, grid_y)).copied()
    }

    /// Check if a grid position is solid (8×8 grid)
    pub fn is_solid(&self, grid_x: i32, grid_y: i32) -> bool {
        matches!(
            self.collision_grid.get(&(grid_x, grid_y)),
            Some(TileType::Wall { .. })
        )
    }

    /// Check if player can move to a world position with given bounding box half-size
    pub fn can_move_to(&self, world_x: f32, world_y: f32, half_size: f32) -> bool {
        let min_x = world_x - half_size;
        let max_x = world_x + half_size;
        let min_y = world_y - half_size;
        let max_y = world_y + half_size;

        // Calculate grid cell range that the bounding box overlaps
        let min_grid_x = (min_x / GRID_SIZE).floor() as i32;
        let max_grid_x = (max_x / GRID_SIZE).floor() as i32;
        let min_grid_y = (min_y / GRID_SIZE).floor() as i32;
        let max_grid_y = (max_y / GRID_SIZE).floor() as i32;

        // Check if any of the cells the bounding box overlaps is solid
        for grid_y in min_grid_y..=max_grid_y {
            for grid_x in min_grid_x..=max_grid_x {
                if grid_x < 0 || grid_x >= self.width || grid_y < 0 || grid_y >= self.height {
                    return false; // Out of bounds
                }
                if self.is_solid(grid_x, grid_y) {
                    return false;
                }
            }
        }
        true
    }

    /// Convert Map to MapFile for saving
    pub fn to_map_file(&self) -> MapFile {
        // Reconstruct grid
        let mut grid = vec![vec![' '; self.width as usize]; self.height as usize];

        for ((grid_x, grid_y), tile_type) in &self.collision_grid {
            if *grid_x >= 0 && *grid_x < self.width && *grid_y >= 0 && *grid_y < self.height {
                let ch = match tile_type {
                    TileType::Wall { height } if *height > 10.0 => 'X',
                    TileType::Wall { .. } => 'x',
                    TileType::Empty => '.',
                };
                grid[*grid_y as usize][*grid_x as usize] = ch;
            }
        }

        let grid_strings: Vec<String> = grid.iter().map(|row| row.iter().collect()).collect();

        // Items and actors will be filled in by save_to_yaml
        MapFile {
            grid: grid_strings,
            items: Vec::new(),
            actors: Vec::new(),
        }
    }

    /// Save map to YAML file
    pub fn save_to_yaml(&self) -> Result<(), String> {
        let mut map_file = self.to_map_file();

        // Build items from world positions with types
        map_file.items = self
            .item_world_positions
            .iter()
            .map(|(pos, item_type)| ItemPosition {
                x: pos.x,
                y: pos.y,
                item_type: item_type.clone(),
            })
            .collect();

        let yaml_string = serde_yaml::to_string(&map_file)
            .map_err(|e| format!("Failed to serialize map: {}", e))?;

        let map_path = PathBuf::from("data/map.yaml");

        fs::write(&map_path, yaml_string)
            .map_err(|e| format!("Failed to write map file: {}", e))?;

        println!("Map saved to {:?}", map_path);
        Ok(())
    }
}

/// Map file format for YAML serialization
#[derive(Debug, Deserialize, Serialize)]
pub struct MapFile {
    pub grid: Vec<String>,
    pub items: Vec<ItemPosition>,
    pub actors: Vec<ActorPosition>,
}
