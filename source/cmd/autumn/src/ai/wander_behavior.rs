use super::ActorBehavior;
use super::pathfinding;
use crate::world::Map;
use bevy::prelude::*;
use rand::Rng;

const MOVEMENT_SPEED: f32 = 10.0; // Units per second
const DESTINATION_THRESHOLD: f32 = 0.5; // How close to destination before considered "arrived"
const ACTOR_RADIUS: f32 = 1.2; // 3/4 of player radius (1.6)

/// State machine for wander behavior
enum WanderState {
    /// Waiting at a destination
    Waiting { timer: f32, duration: f32 },
    /// Planning a new route
    Planning,
    /// Moving along a path
    Moving {
        path: Vec<(f32, f32)>,
        current_index: usize,
        destinations: Vec<(f32, f32)>,
    },
}

/// Wander behavior - actor moves between random destinations
pub struct WanderBehavior {
    state: WanderState,
}

impl WanderBehavior {
    pub fn new() -> Self {
        Self {
            state: WanderState::Planning,
        }
    }
}

impl ActorBehavior for WanderBehavior {
    fn update(
        &mut self,
        transform: &mut Transform,
        map: &Map,
        delta_time: f32,
        speed_multiplier: f32,
        _player_position: Option<Vec2>,
        _actor: &crate::ai::ActorData,
    ) -> bool {
        let mut is_moving = false;

        match &mut self.state {
            WanderState::Waiting { timer, duration } => {
                *timer += delta_time;
                if *timer >= *duration {
                    // Waiting complete, plan new route
                    self.state = WanderState::Planning;
                }
            }

            WanderState::Planning => {
                // Generate 2-3 random destinations
                let mut rng = rand::rng();
                let num_destinations = rng.random_range(2..=3);
                let mut destinations = Vec::new();

                // Try to find valid destinations
                for _ in 0..num_destinations {
                    for _ in 0..20 {
                        // Max 20 attempts per destination
                        let dest_x = rng.random_range(0.0..map.width as f32 * 8.0);
                        let dest_y = rng.random_range(0.0..map.height as f32 * 8.0);

                        if map.can_move_to(dest_x, dest_y, ACTOR_RADIUS) {
                            destinations.push((dest_x, dest_y));
                            break;
                        }
                    }
                }

                if destinations.is_empty() {
                    // Couldn't find any valid destinations, just wait
                    self.state = WanderState::Waiting {
                        timer: 0.0,
                        duration: rng.random_range(1.0..3.0),
                    };
                } else {
                    // Find path to first destination
                    let current_x = transform.translation.x;
                    let current_y = transform.translation.y;

                    if let Some(path) = pathfinding::find_path(
                        map,
                        current_x,
                        current_y,
                        destinations[0].0,
                        destinations[0].1,
                    ) {
                        self.state = WanderState::Moving {
                            path,
                            current_index: 0,
                            destinations,
                        };
                    } else {
                        // Pathfinding failed, wait instead
                        self.state = WanderState::Waiting {
                            timer: 0.0,
                            duration: rng.random_range(1.0..3.0),
                        };
                    }
                }
            }

            WanderState::Moving {
                path,
                current_index,
                destinations,
            } => {
                is_moving = true;

                if *current_index >= path.len() {
                    // Reached end of current path
                    if destinations.len() > 1 {
                        // Remove completed destination and path to next one
                        destinations.remove(0);
                        let current_x = transform.translation.x;
                        let current_y = transform.translation.y;

                        if let Some(new_path) = pathfinding::find_path(
                            map,
                            current_x,
                            current_y,
                            destinations[0].0,
                            destinations[0].1,
                        ) {
                            *path = new_path;
                            *current_index = 0;
                        } else {
                            // Pathfinding failed, go to waiting/planning
                            let mut rng = rand::rng();
                            if rng.random_bool(0.7) {
                                self.state = WanderState::Planning;
                            } else {
                                self.state = WanderState::Waiting {
                                    timer: 0.0,
                                    duration: rng.random_range(1.0..3.0),
                                };
                            }
                        }
                    } else {
                        // All destinations reached, decide what to do next
                        let mut rng = rand::rng();
                        if rng.random_bool(0.7) {
                            // 70% chance to wander again
                            self.state = WanderState::Planning;
                        } else {
                            // 30% chance to wait
                            self.state = WanderState::Waiting {
                                timer: 0.0,
                                duration: rng.random_range(1.0..3.0),
                            };
                        }
                    }
                } else {
                    // Move towards current waypoint
                    let target = path[*current_index];
                    let current_x = transform.translation.x;
                    let current_y = transform.translation.y;

                    let dx = target.0 - current_x;
                    let dy = target.1 - current_y;
                    let distance = (dx * dx + dy * dy).sqrt();

                    if distance <= DESTINATION_THRESHOLD {
                        // Reached this waypoint, move to next
                        *current_index += 1;
                    } else {
                        // Move towards waypoint
                        let move_distance = MOVEMENT_SPEED * speed_multiplier * delta_time;
                        let move_distance = move_distance.min(distance);

                        let new_x = current_x + (dx / distance) * move_distance;
                        let new_y = current_y + (dy / distance) * move_distance;

                        // Check if new position is valid
                        if map.can_move_to(new_x, new_y, ACTOR_RADIUS) {
                            transform.translation.x = new_x;
                            transform.translation.y = new_y;
                        } else {
                            // Hit an obstacle, replan
                            self.state = WanderState::Planning;
                        }
                    }
                }
            }
        }

        is_moving
    }

    fn get_label(&self) -> &str {
        "wander"
    }
}
