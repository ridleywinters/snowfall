use super::ActorBehavior;
use super::pathfinding;
use crate::world::Map;
use bevy::prelude::*;
use rand::Rng;

const MOVEMENT_SPEED: f32 = 10.0;
const DESTINATION_THRESHOLD: f32 = 0.5;
const ACTOR_RADIUS: f32 = 1.2;

// Detection and combat ranges
const DETECTION_RANGE: f32 = 25.0;
const CHASE_RANGE: f32 = 40.0;
const ATTACK_RANGE: f32 = 4.0;
// Add buffer to attack range to prevent oscillation when player moves slightly
const ATTACK_RANGE_BUFFER: f32 = 1.5;
// When this close to player, stop using pathfinding and move directly toward them
const DIRECT_MOVEMENT_RANGE: f32 = 12.0;

// Timing constants
const PATH_REPLAN_INTERVAL: f32 = 0.5;

/// Wander sub-state for when no player is detected
enum WanderSubState {
    Waiting {
        timer: f32,
        duration: f32,
    },
    Planning,
    Moving {
        path: Vec<(f32, f32)>,
        current_index: usize,
        destinations: Vec<(f32, f32)>,
    },
}

/// State machine for aggressive behavior
enum AggressiveState {
    /// Wandering when no player nearby
    Wandering { wander_state: WanderSubState },
    /// Chasing the player
    Chasing {
        path: Vec<(f32, f32)>,
        current_index: usize,
        replan_timer: f32,
    },
    /// Currently attacking
    Attacking { timer: f32, has_dealt_damage: bool },
    /// Cooldown after attack
    Cooldown { timer: f32 },
}

/// Aggressive behavior - wanders until player detected, then chases and attacks
pub struct AggressiveBehavior {
    state: AggressiveState,
}

impl AggressiveBehavior {
    pub fn new() -> Self {
        Self {
            state: AggressiveState::Wandering {
                wander_state: WanderSubState::Planning,
            },
        }
    }

    /// Check if player is within detection range
    fn can_detect_player(actor_pos: Vec2, player_pos: Vec2) -> bool {
        actor_pos.distance(player_pos) <= DETECTION_RANGE
    }

    /// Check if player is within chase range
    fn in_chase_range(actor_pos: Vec2, player_pos: Vec2) -> bool {
        actor_pos.distance(player_pos) <= CHASE_RANGE
    }

    /// Check if player is within attack range
    fn in_attack_range(actor_pos: Vec2, player_pos: Vec2, attack_range: f32) -> bool {
        actor_pos.distance(player_pos) <= attack_range
    }

    /// Update wander sub-state (reused from WanderBehavior logic)
    fn update_wander(
        wander_state: &mut WanderSubState,
        transform: &mut Transform,
        map: &Map,
        delta_time: f32,
        speed_multiplier: f32,
    ) -> bool {
        let mut is_moving = false;

        match wander_state {
            WanderSubState::Waiting { timer, duration } => {
                *timer += delta_time;
                if *timer >= *duration {
                    *wander_state = WanderSubState::Planning;
                }
            }

            WanderSubState::Planning => {
                let mut rng = rand::rng();
                let num_destinations = rng.random_range(2..=3);
                let mut destinations = Vec::new();

                for _ in 0..num_destinations {
                    for _ in 0..20 {
                        let dest_x = rng.random_range(0.0..map.width as f32 * 8.0);
                        let dest_y = rng.random_range(0.0..map.height as f32 * 8.0);

                        if map.can_move_to(dest_x, dest_y, ACTOR_RADIUS) {
                            destinations.push((dest_x, dest_y));
                            break;
                        }
                    }
                }

                if destinations.is_empty() {
                    *wander_state = WanderSubState::Waiting {
                        timer: 0.0,
                        duration: rng.random_range(1.0..3.0),
                    };
                } else {
                    let current_x = transform.translation.x;
                    let current_y = transform.translation.y;

                    if let Some(path) = pathfinding::find_path(
                        map,
                        current_x,
                        current_y,
                        destinations[0].0,
                        destinations[0].1,
                    ) {
                        *wander_state = WanderSubState::Moving {
                            path,
                            current_index: 0,
                            destinations,
                        };
                    } else {
                        *wander_state = WanderSubState::Waiting {
                            timer: 0.0,
                            duration: rng.random_range(1.0..3.0),
                        };
                    }
                }
            }

            WanderSubState::Moving {
                path,
                current_index,
                destinations,
            } => {
                is_moving = true;

                if *current_index >= path.len() {
                    if destinations.len() > 1 {
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
                            *wander_state = WanderSubState::Planning;
                        }
                    } else {
                        let mut rng = rand::rng();
                        if rng.random_bool(0.7) {
                            *wander_state = WanderSubState::Planning;
                        } else {
                            *wander_state = WanderSubState::Waiting {
                                timer: 0.0,
                                duration: rng.random_range(1.0..3.0),
                            };
                        }
                    }
                } else {
                    let target = path[*current_index];
                    let current_x = transform.translation.x;
                    let current_y = transform.translation.y;

                    let dx = target.0 - current_x;
                    let dy = target.1 - current_y;
                    let distance = (dx * dx + dy * dy).sqrt();

                    if distance <= DESTINATION_THRESHOLD {
                        *current_index += 1;
                    } else {
                        let move_distance = MOVEMENT_SPEED * speed_multiplier * delta_time;
                        let move_distance = move_distance.min(distance);

                        let new_x = current_x + (dx / distance) * move_distance;
                        let new_y = current_y + (dy / distance) * move_distance;

                        if map.can_move_to(new_x, new_y, ACTOR_RADIUS) {
                            transform.translation.x = new_x;
                            transform.translation.y = new_y;
                        } else {
                            *wander_state = WanderSubState::Planning;
                        }
                    }
                }
            }
        }

        is_moving
    }
}

impl ActorBehavior for AggressiveBehavior {
    fn update(
        &mut self,
        transform: &mut Transform,
        map: &Map,
        delta_time: f32,
        speed_multiplier: f32,
        player_position: Option<Vec2>,
        actor: &crate::ai::ActorData,
    ) -> bool {
        let actor_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let mut is_moving = false;

        // If no player position available, just wander
        let Some(player_pos) = player_position else {
            if let AggressiveState::Wandering {
                ref mut wander_state,
            } = self.state
            {
                return Self::update_wander(
                    wander_state,
                    transform,
                    map,
                    delta_time,
                    speed_multiplier,
                );
            }
            self.state = AggressiveState::Wandering {
                wander_state: WanderSubState::Planning,
            };
            return false;
        };

        match &mut self.state {
            AggressiveState::Wandering { wander_state } => {
                // Check if player entered detection range
                if Self::can_detect_player(actor_pos, player_pos) {
                    // Transition to chasing
                    if let Some(path) = pathfinding::find_path(
                        map,
                        actor_pos.x,
                        actor_pos.y,
                        player_pos.x,
                        player_pos.y,
                    ) {
                        self.state = AggressiveState::Chasing {
                            path,
                            current_index: 0,
                            replan_timer: 0.0,
                        };
                    }
                } else {
                    // Continue wandering
                    is_moving = Self::update_wander(
                        wander_state,
                        transform,
                        map,
                        delta_time,
                        speed_multiplier,
                    );
                }
            }

            AggressiveState::Chasing {
                path,
                current_index,
                replan_timer,
            } => {
                is_moving = true;

                // Check if player escaped
                if !Self::in_chase_range(actor_pos, player_pos) {
                    self.state = AggressiveState::Wandering {
                        wander_state: WanderSubState::Planning,
                    };
                    return false;
                }

                // Check if we should enter attack range
                if Self::in_attack_range(actor_pos, player_pos, actor.attack_range) {
                    self.state = AggressiveState::Attacking {
                        timer: 0.0,
                        has_dealt_damage: false,
                    };
                    return false;
                }

                // Determine movement strategy based on distance
                let distance_to_player = actor_pos.distance(player_pos);

                if distance_to_player <= DIRECT_MOVEMENT_RANGE {
                    // Close enough - move directly toward player without pathfinding
                    let dx = player_pos.x - actor_pos.x;
                    let dy = player_pos.y - actor_pos.y;
                    let distance = (dx * dx + dy * dy).sqrt();

                    if distance > 0.1 {
                        let move_distance = MOVEMENT_SPEED * speed_multiplier * delta_time;
                        let move_distance = move_distance.min(distance);

                        let new_x = actor_pos.x + (dx / distance) * move_distance;
                        let new_y = actor_pos.y + (dy / distance) * move_distance;

                        if map.can_move_to(new_x, new_y, ACTOR_RADIUS) {
                            transform.translation.x = new_x;
                            transform.translation.y = new_y;
                        } else {
                            // Hit a wall during direct movement, switch back to pathfinding
                            if let Some(new_path) = pathfinding::find_path(
                                map,
                                actor_pos.x,
                                actor_pos.y,
                                player_pos.x,
                                player_pos.y,
                            ) {
                                *path = new_path;
                                *current_index = 0;
                                *replan_timer = 0.0;
                            }
                        }
                    }
                } else {
                    // Too far - use pathfinding

                    // Replan path periodically to track moving player
                    *replan_timer += delta_time;
                    if *replan_timer >= PATH_REPLAN_INTERVAL {
                        *replan_timer = 0.0;
                        if let Some(new_path) = pathfinding::find_path(
                            map,
                            actor_pos.x,
                            actor_pos.y,
                            player_pos.x,
                            player_pos.y,
                        ) {
                            *path = new_path;
                            *current_index = 0;
                        }
                    }

                    // Move along path
                    if *current_index >= path.len() {
                        // Path exhausted, replan immediately
                        if let Some(new_path) = pathfinding::find_path(
                            map,
                            actor_pos.x,
                            actor_pos.y,
                            player_pos.x,
                            player_pos.y,
                        ) {
                            *path = new_path;
                            *current_index = 0;
                        } else {
                            // Can't find path, go back to wandering
                            self.state = AggressiveState::Wandering {
                                wander_state: WanderSubState::Planning,
                            };
                        }
                    } else {
                        let target = path[*current_index];
                        let dx = target.0 - actor_pos.x;
                        let dy = target.1 - actor_pos.y;
                        let distance = (dx * dx + dy * dy).sqrt();

                        if distance <= DESTINATION_THRESHOLD {
                            *current_index += 1;
                        } else {
                            let move_distance = MOVEMENT_SPEED * speed_multiplier * delta_time;
                            let move_distance = move_distance.min(distance);

                            let new_x = actor_pos.x + (dx / distance) * move_distance;
                            let new_y = actor_pos.y + (dy / distance) * move_distance;

                            if map.can_move_to(new_x, new_y, ACTOR_RADIUS) {
                                transform.translation.x = new_x;
                                transform.translation.y = new_y;
                            }
                        }
                    }
                }
            }

            AggressiveState::Attacking { timer, .. } => {
                // Attack animation is handled by separate system
                // Check if attack is complete (attack state is Idle)
                *timer += delta_time;

                // Only transition out of attacking if:
                // 1. The attack animation is complete (Idle state)
                // 2. AND player has moved beyond the attack range buffer
                if actor.attack_state == crate::actor::ActorAttackState::Idle {
                    // Use buffered range to prevent oscillation - actor stays committed to attacking
                    // as long as player is within attack_range + buffer
                    let buffered_range = actor.attack_range + ATTACK_RANGE_BUFFER;

                    if !Self::in_attack_range(actor_pos, player_pos, buffered_range) {
                        // Player is beyond the buffered range
                        if Self::in_chase_range(actor_pos, player_pos) {
                            // Player moved out of attack range but still in chase range
                            // Resume chasing
                            if let Some(path) = pathfinding::find_path(
                                map,
                                actor_pos.x,
                                actor_pos.y,
                                player_pos.x,
                                player_pos.y,
                            ) {
                                self.state = AggressiveState::Chasing {
                                    path,
                                    current_index: 0,
                                    replan_timer: 0.0,
                                };
                            }
                        } else {
                            // Player escaped, return to wandering
                            self.state = AggressiveState::Wandering {
                                wander_state: WanderSubState::Planning,
                            };
                        }
                    }
                    // If player is still within buffered range, stay in Attacking state
                    // The attack system will handle cooldown and initiate next attack
                }
            }

            AggressiveState::Cooldown { timer } => {
                // Cooldown is handled by separate system
                *timer += delta_time;
            }
        }

        is_moving
    }

    fn get_label(&self) -> &str {
        "aggressive"
    }
}
