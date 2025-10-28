use bevy::prelude::*;

#[derive(Resource)]
pub struct PlayerStats {
    pub health: f32,  // 0.0 to 100.0
    pub fatigue: f32, // 0.0 to 100.0
    pub gold: u32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            health: 100.0,
            fatigue: 50.0,
            gold: 0,
        }
    }
}

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct FatigueBar;

#[derive(Component)]
pub struct GoldText;

pub fn setup_ui(mut commands: Commands) {
    // Initialize player stats
    commands.insert_resource(PlayerStats::default());

    // PICO-8 colors
    let pico8_red = Color::srgb(1.0, 0.0, 0.3);
    let pico8_green = Color::srgb(0.0, 0.89, 0.21);

    // Root UI node
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        })
        .with_children(|parent| {
            // Container for status bars
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|parent| {
                    // Health bar
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(10.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            // Bar background
                            parent
                                .spawn((
                                    Node {
                                        width: Val::Px(200.0),
                                        height: Val::Px(20.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                    Outline {
                                        width: Val::Px(1.0),
                                        offset: Val::Px(0.0),
                                        color: Color::srgb(0.25, 0.25, 0.25),
                                    },
                                ))
                                .with_children(|parent| {
                                    // Bar fill
                                    parent.spawn((
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        BackgroundColor(pico8_red),
                                        HealthBar,
                                    ));
                                });
                        });

                    // Fatigue bar
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(10.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            // Bar background
                            parent
                                .spawn((
                                    Node {
                                        width: Val::Px(200.0),
                                        height: Val::Px(20.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                    Outline {
                                        width: Val::Px(1.0),
                                        offset: Val::Px(0.0),
                                        color: Color::srgb(0.25, 0.25, 0.25),
                                    },
                                ))
                                .with_children(|parent| {
                                    // Bar fill
                                    parent.spawn((
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        BackgroundColor(pico8_green),
                                        FatigueBar,
                                    ));
                                });
                        });

                    // Gold text
                    parent.spawn((
                        Text::new("Gold: 0"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        GoldText,
                    ));
                });
        });
}

pub fn update_ui(
    stats: Res<PlayerStats>,
    mut health_query: Query<&mut Node, (With<HealthBar>, Without<FatigueBar>)>,
    mut fatigue_query: Query<&mut Node, (With<FatigueBar>, Without<HealthBar>)>,
    mut gold_query: Query<&mut Text, With<GoldText>>,
) {
    // Update health bar width
    if let Ok(mut node) = health_query.single_mut() {
        node.width = Val::Percent(stats.health);
    }

    // Update fatigue bar width
    if let Ok(mut node) = fatigue_query.single_mut() {
        node.width = Val::Percent(stats.fatigue);
    }

    // Update gold text
    if let Ok(mut text) = gold_query.single_mut() {
        **text = format!("Gold: {}", stats.gold);
    }
}

// Test system to modify stats with number keys (for demonstration)
pub fn test_stats_input(input: Res<ButtonInput<KeyCode>>, mut stats: ResMut<PlayerStats>) {
    if input.just_pressed(KeyCode::Digit1) {
        stats.health = (stats.health - 10.0).max(0.0);
    }
    if input.just_pressed(KeyCode::Digit2) {
        stats.health = (stats.health + 10.0).min(100.0);
    }
    if input.just_pressed(KeyCode::Digit3) {
        stats.fatigue = (stats.fatigue - 10.0).max(0.0);
    }
    if input.just_pressed(KeyCode::Digit4) {
        stats.fatigue = (stats.fatigue + 10.0).min(100.0);
    }
    if input.just_pressed(KeyCode::Digit5) {
        stats.gold = stats.gold.saturating_sub(10);
    }
    if input.just_pressed(KeyCode::Digit6) {
        stats.gold = stats.gold.saturating_add(10);
    }
}
