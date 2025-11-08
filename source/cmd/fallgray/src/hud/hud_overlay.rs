use crate::camera::Player;
use crate::game_state::GamePlayEntity;
use crate::rendering::load_image_texture;
use fallgray_bevy_ui::EntityCommandsUIExt;
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

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct FatigueBar;

#[derive(Component)]
pub struct GoldText;

pub fn startup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Spawning playing state UI");
    // Initialize player stats
    commands.insert_resource(PlayerStats::default());

    let container_style = vec![
        "flex-row-center gap10 p8", //
        "bg-rgba(0.2,0.2,0.2,0.8)",
    ];
    let icon_style = vec!["width-20 height-20"];
    let bar_style = vec![
        "width-200 height-20", //
        "bg-rgba(0.2,0.2,0.2,1.0)",
        "outline-width-1 outline-rgb(0.25,0.25,0.25)",
    ];
    let pico8_red = "bg-rgb(1.0,0.0,0.3)";
    let pico8_green = "bg-rgb(0.0,0.89,0.21)";

    // Status bars at bottom left
    commands
        .spawn(GamePlayEntity)
        .styles(&vec![
            "absolute width-100% height-100% p8",
            "justify-start align-end",
        ])
        .with_children(|parent| {
            // Container for status bars
            parent
                .spawn(Interaction::default())
                .styles(&vec!["flex-col gap2"])
                .with_children(|parent| {
                    parent // Health bar
                        .spawn_empty()
                        .styles(&container_style)
                        .with_children(|parent| {
                            parent
                                .spawn(ImageNode::new(load_image_texture(
                                    &asset_server,
                                    "base/icons/heart.png",
                                )))
                                .styles(&icon_style);
                            parent
                                .spawn_empty()
                                .styles(&bar_style)
                                .with_children(|parent| {
                                    parent
                                        .spawn(HealthBar)
                                        .styles(&vec!["width-100% height-100%", pico8_red]);
                                });
                        });

                    parent // Fatigue bar
                        .spawn_empty()
                        .styles(&container_style)
                        .with_children(|parent| {
                            parent
                                .spawn((ImageNode::new(load_image_texture(
                                    &asset_server,
                                    "base/icons/foot.png",
                                )),))
                                .styles(&icon_style);
                            parent
                                .spawn_empty()
                                .styles(&bar_style)
                                .with_children(|parent| {
                                    parent
                                        .spawn(FatigueBar)
                                        .styles(&vec!["width-100% height-100%", pico8_green]);
                                });
                        });
                });
        });

    // Gold text (keeping at top for now)
    commands
        .spawn(GamePlayEntity)
        .style("width-100% height-100% justify-start align-start p20 absolute")
        .with_children(|parent| {
            parent
                .spawn(GoldText)
                .text("Gold: 0")
                .style("font-size-16 fg-white");
        });
}

pub fn update_ui(
    player_query: Query<&Player>,
    mut stats: ResMut<PlayerStats>,
    mut health_query: Query<&mut Node, (With<HealthBar>, Without<FatigueBar>)>,
    mut fatigue_query: Query<&mut Node, (With<FatigueBar>, Without<HealthBar>)>,
    mut gold_query: Query<&mut Text, With<GoldText>>,
) {
    // Sync Player health to PlayerStats
    if let Ok(player) = player_query.single() {
        stats.health = (player.current_health / player.max_health * 100.0).clamp(0.0, 100.0);
    }

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
}
