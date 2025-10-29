use bevy::prelude::*;

#[derive(Resource)]
pub struct PlayerStats {
    pub health: f32,  // 0.0 to 100.0
    pub stamina: f32, // 0.0 to 100.0
    pub gold: i32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            health: 100.0,
            stamina: 50.0,
            gold: 0,
        }
    }
}

#[derive(Resource)]
pub struct Toolbar {
    pub active_slot: usize, // 0-9
}

impl Default for Toolbar {
    fn default() -> Self {
        Self { active_slot: 0 }
    }
}

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct FatigueBar;

#[derive(Component)]
pub struct GoldText;

#[derive(Component)]
pub struct ToolbarSlot {
    pub slot_index: usize,
}

pub fn startup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Initialize player stats
    commands.insert_resource(PlayerStats::default());
    commands.insert_resource(Toolbar::default());

    // PICO-8 colors
    let pico8_red = Color::srgb(1.0, 0.0, 0.3);
    let pico8_green = Color::srgb(0.0, 0.89, 0.21);

    // Status bars at bottom left
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexEnd,
            padding: UiRect::all(Val::Px(8.0)),
            position_type: PositionType::Absolute,
            ..default()
        })
        .with_children(|parent| {
            // Container for status bars
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                    Interaction::default(),
                ))
                .with_children(|parent| {
                    // Health bar with its own background
                    parent
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(10.0),
                                padding: UiRect::all(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                        ))
                        .with_children(|parent| {
                            // Heart icon
                            parent.spawn((
                                ImageNode::new(asset_server.load("base/icons/heart.png")),
                                Node {
                                    width: Val::Px(20.0),
                                    height: Val::Px(20.0),
                                    ..default()
                                },
                            ));

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

                    // Fatigue bar with its own background
                    parent
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(10.0),
                                padding: UiRect::all(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                        ))
                        .with_children(|parent| {
                            // Foot icon
                            parent.spawn((
                                ImageNode::new(asset_server.load("base/icons/foot.png")),
                                Node {
                                    width: Val::Px(20.0),
                                    height: Val::Px(20.0),
                                    ..default()
                                },
                            ));

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
                });
        });

    // Gold text (keeping at top for now)
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            padding: UiRect::all(Val::Px(20.0)),
            position_type: PositionType::Absolute,
            ..default()
        })
        .with_children(|parent| {
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

    // Toolbar icons
    let toolbar_icons = [
        "torch",
        "sword",
        "bow",
        "chest",
        "key",
        "map",
        "book",
        "diamond",
        "camp",
        "question",
        "flag_green",
        "bowl",
        "feather",
        "shovel",
        "axe",
        "glove",
        "letter",
        "foot",
        "heart",
    ];

    // Toolbar at the bottom center
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
            padding: UiRect::all(Val::Px(8.0)),
            position_type: PositionType::Absolute,
            ..default()
        })
        .with_children(|parent| {
            // Toolbar container with interaction area and margin
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(4.0),
                        padding: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    Interaction::default(),
                ))
                .with_children(|parent| {
                    // Create 10 toolbar slots (0-9)
                    for i in 0..10 {
                        let outline_color = if i == 0 {
                            Color::WHITE
                        } else {
                            Color::srgb(0.4, 0.4, 0.4)
                        };

                        // Get icon for this slot (wrap if index exceeds array length)
                        let icon_name = toolbar_icons[i % toolbar_icons.len()];
                        let icon_path = format!("base/icons/{}.png", icon_name);
                        let icon_image = asset_server.load(icon_path);

                        parent
                            .spawn((
                                Node {
                                    width: Val::Px(64.0),
                                    height: Val::Px(64.0),
                                    padding: UiRect::all(Val::Px(4.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    position_type: PositionType::Relative,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                                Outline {
                                    width: Val::Px(2.0),
                                    offset: Val::Px(0.0),
                                    color: outline_color,
                                },
                                ToolbarSlot { slot_index: i },
                                Interaction::default(),
                            ))
                            .with_children(|parent| {
                                // Icon for this slot
                                parent.spawn((
                                    ImageNode::new(icon_image),
                                    Node {
                                        width: Val::Px(48.0),
                                        height: Val::Px(48.0),
                                        ..default()
                                    },
                                ));

                                // Numeric label (1-9, 0 for slot 9)
                                let label_text = if i == 9 { "0" } else { &(i + 1).to_string() };
                                parent.spawn((
                                    Text::new(label_text),
                                    TextFont {
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                    Node {
                                        position_type: PositionType::Absolute,
                                        top: Val::Px(2.0),
                                        left: Val::Px(2.0),
                                        ..default()
                                    },
                                ));
                            });
                    }
                });
        });
}

pub fn update_ui(
    stats: Res<PlayerStats>,
    toolbar: Res<Toolbar>,
    mut health_query: Query<&mut Node, (With<HealthBar>, Without<FatigueBar>)>,
    mut fatigue_query: Query<&mut Node, (With<FatigueBar>, Without<HealthBar>)>,
    mut gold_query: Query<&mut Text, With<GoldText>>,
    mut toolbar_slots: Query<(&ToolbarSlot, &mut Outline)>,
) {
    // Update health bar width
    if let Ok(mut node) = health_query.single_mut() {
        node.width = Val::Percent(stats.health);
    }

    // Update fatigue bar width
    if let Ok(mut node) = fatigue_query.single_mut() {
        node.width = Val::Percent(stats.stamina);
    }

    // Update gold text
    if let Ok(mut text) = gold_query.single_mut() {
        **text = format!("Gold: {}", stats.gold);
    }

    // Update toolbar slot outlines
    for (slot, mut outline) in toolbar_slots.iter_mut() {
        outline.color = if slot.slot_index == toolbar.active_slot {
            Color::WHITE
        } else {
            Color::srgb(0.4, 0.4, 0.4)
        };
    }
}

// Test system to modify stats with number keys (for demonstration)
// Also handles toolbar slot selection (keys 1-9 and 0)
pub fn update_toolbar_input(
    input: Res<ButtonInput<KeyCode>>,
    mut stats: ResMut<PlayerStats>,
    mut toolbar: ResMut<Toolbar>,
) {
    // Toolbar slot selection (1-9, 0 for slot 10)
    if input.just_pressed(KeyCode::Digit1) {
        toolbar.active_slot = 0;
    }
    if input.just_pressed(KeyCode::Digit2) {
        toolbar.active_slot = 1;
    }
    if input.just_pressed(KeyCode::Digit3) {
        toolbar.active_slot = 2;
    }
    if input.just_pressed(KeyCode::Digit4) {
        toolbar.active_slot = 3;
    }
    if input.just_pressed(KeyCode::Digit5) {
        toolbar.active_slot = 4;
    }
    if input.just_pressed(KeyCode::Digit6) {
        toolbar.active_slot = 5;
    }
    if input.just_pressed(KeyCode::Digit7) {
        toolbar.active_slot = 6;
    }
    if input.just_pressed(KeyCode::Digit8) {
        toolbar.active_slot = 7;
    }
    if input.just_pressed(KeyCode::Digit9) {
        toolbar.active_slot = 8;
    }
    if input.just_pressed(KeyCode::Digit0) {
        toolbar.active_slot = 9;
    }
}

pub fn update_toolbar_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut toolbar: ResMut<Toolbar>,
    slot_query: Query<(&Interaction, &ToolbarSlot)>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    for (interaction, slot) in slot_query.iter() {
        if *interaction == Interaction::Pressed {
            toolbar.active_slot = slot.slot_index;
        }
    }
}
