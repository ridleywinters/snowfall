use crate::console::ConsoleState;
use crate::game_state::GamePlayEntity;
use crate::game_state::GameState;
use crate::rendering::load_image_texture;
use fallgray_bevy_ui::EntityCommandsUIExt;
use bevy::prelude::*;

/// Resource tracking the currently active toolbar slot
#[derive(Resource)]
pub struct Toolbar {
    pub active_slot: usize, // 1-9, 0 for 10th slot
}

impl Default for Toolbar {
    fn default() -> Self {
        Self { active_slot: 1 }
    }
}

/// Component marking a toolbar slot with its index
#[derive(Component)]
struct ToolbarSlot {
    slot_index: usize,
}

/// Plugin that adds toolbar functionality
pub struct ToolbarPlugin;

impl Plugin for ToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Toolbar::default())
            .add_systems(OnEnter(GameState::Playing), startup_toolbar)
            .add_systems(
                Update,
                (
                    update_toolbar_ui,
                    update_toolbar_input,
                    update_toolbar_click,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn startup_toolbar(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Toolbar icons
    let toolbar_icons = [
        "torch",
        "axe",
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
        "glove",
        "letter",
        "foot",
        "heart",
        "sword",
    ];

    // Toolbar at the bottom center
    commands
        .spawn(GamePlayEntity)
        .style("width-100% height-100% justify-center align-end p8 absolute")
        .with_children(|parent| {
            // Toolbar container with interaction area and margin
            parent
                .spawn(Interaction::default())
                .style("flex-row gap4 p4")
                .with_children(|parent| {
                    // Create 10 toolbar slots (1-9, then 0 for the 10th slot)
                    for i in 0..10 {
                        // Map visual position to slot number: pos 0->slot 1, pos 1->slot 2, ..., pos 9->slot 0
                        let slot_number = if i == 9 { 0 } else { i + 1 };

                        // Get icon for this slot (wrap if index exceeds array length)
                        let icon_name = toolbar_icons[i % toolbar_icons.len()];
                        let icon_path = format!("base/icons/{}.png", icon_name);
                        let icon_image = load_image_texture(&asset_server, icon_path);

                        parent
                            .spawn((
                                ToolbarSlot {
                                    slot_index: slot_number,
                                },
                                Interaction::default(),
                            ))
                            .styles(&vec![
                                "width-64 height-64 p4 justify-center align-center relative",
                                "bg-rgba(0.2,0.2,0.2,0.8)",
                                "outline-width-2",
                                if slot_number == 1 {
                                    "outline-rgb(1.0,1.0,1.0)"
                                } else {
                                    "outline-rgb(0.4,0.4,0.4)"
                                },
                            ])
                            .with_children(|parent| {
                                parent
                                    .spawn((ImageNode::new(icon_image),))
                                    .style("width-48 height-48");
                                let label_text = if i == 9 { "0" } else { &(i + 1).to_string() };
                                parent
                                    .spawn_empty()
                                    .text(label_text)
                                    .style("font-size-14 fg-white absolute top-2 left-2");
                            });
                    }
                });
        });
}

fn update_toolbar_ui(
    toolbar: Res<Toolbar>,
    mut toolbar_slots: Query<(&ToolbarSlot, &mut Outline)>,
) {
    // Update toolbar slot outlines
    for (slot, mut outline) in toolbar_slots.iter_mut() {
        outline.color = if slot.slot_index == toolbar.active_slot {
            Color::WHITE
        } else {
            Color::srgb(0.4, 0.4, 0.4)
        };
    }
}

fn update_toolbar_input(
    input: Res<ButtonInput<KeyCode>>,
    mut toolbar: ResMut<Toolbar>,
    console_state: Res<ConsoleState>,
) {
    // Don't process toolbar input if console is open
    if console_state.visible {
        return;
    }

    // Toolbar slot selection (1-9, 0 for slot 10)
    if input.just_pressed(KeyCode::Digit1) {
        toolbar.active_slot = 1;
    }
    if input.just_pressed(KeyCode::Digit2) {
        toolbar.active_slot = 2;
    }
    if input.just_pressed(KeyCode::Digit3) {
        toolbar.active_slot = 3;
    }
    if input.just_pressed(KeyCode::Digit4) {
        toolbar.active_slot = 4;
    }
    if input.just_pressed(KeyCode::Digit5) {
        toolbar.active_slot = 5;
    }
    if input.just_pressed(KeyCode::Digit6) {
        toolbar.active_slot = 6;
    }
    if input.just_pressed(KeyCode::Digit7) {
        toolbar.active_slot = 7;
    }
    if input.just_pressed(KeyCode::Digit8) {
        toolbar.active_slot = 8;
    }
    if input.just_pressed(KeyCode::Digit9) {
        toolbar.active_slot = 9;
    }
    if input.just_pressed(KeyCode::Digit0) {
        toolbar.active_slot = 0;
    }
}

fn update_toolbar_click(
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
