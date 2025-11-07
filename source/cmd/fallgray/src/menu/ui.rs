use bevy::prelude::*;

use crate::game_state::GameState;
use crate::ui_styles::EntityCommandsUIExt;

#[derive(Component)]
pub(super) struct MainMenuUI;

#[derive(Component)]
pub(super) struct GameOverUI;

#[derive(Component)]
pub(super) struct NewGameButton;

#[derive(Component)]
pub(super) struct RestartButton;

#[derive(Component)]
pub(super) struct QuitButton;

/// Spawn main menu UI
pub(super) fn spawn_main_menu(mut commands: Commands) {
    info!("Spawning main menu UI");
    commands
        .spawn(MainMenuUI)
        .styles(&vec![
            "absolute width-100% height-100%",
            "flex-col-center",
            "bg-rgba(0.0,0.0,0.0,0.8)",
        ])
        .with_children(|parent| {
            // Title
            parent.spawn_empty().text("FALLGRAY").styles(&vec![
                "font-size-64",
                "fg-white",
                "mb-40",
            ]);

            // New Game Button
            parent
                .spawn((NewGameButton, Interaction::default()))
                .styles(&vec![
                    "px-40 py-12",
                    "bg-rgba(0.3,0.3,0.3,0.9)",
                    "outline-width-2 outline-rgb(0.5,0.5,0.5)",
                    "mb-12",
                ])
                .with_children(|parent| {
                    parent
                        .spawn_empty()
                        .text("New Game")
                        .style("font-size-24 fg-white");
                });

            // Quit Button
            parent
                .spawn((QuitButton, Interaction::default()))
                .styles(&vec![
                    "px-40 py-12",
                    "bg-rgba(0.3,0.3,0.3,0.9)",
                    "outline-width-2 outline-rgb(0.5,0.5,0.5)",
                ])
                .with_children(|parent| {
                    parent
                        .spawn_empty()
                        .text("Quit")
                        .style("font-size-24 fg-white");
                });
        });
}

/// Spawn game over UI
pub(super) fn spawn_game_over(mut commands: Commands) {
    info!("Spawning game over UI");
    commands
        .spawn(GameOverUI)
        .styles(&vec![
            "absolute width-100% height-100%",
            "flex-col-center",
            "bg-rgba(0.0,0.0,0.0,0.8)",
        ])
        .with_children(|parent| {
            // Game Over Title
            parent.spawn_empty().text("GAME OVER").styles(&vec![
                "font-size-64",
                "fg-rgb(1.0,0.0,0.3)",
                "mb-40",
            ]);

            // Restart Button
            parent
                .spawn((RestartButton, Interaction::default()))
                .styles(&vec![
                    "px-40 py-12",
                    "bg-rgba(0.3,0.3,0.3,0.9)",
                    "outline-width-2 outline-rgb(0.5,0.5,0.5)",
                    "mb-12",
                ])
                .with_children(|parent| {
                    parent
                        .spawn_empty()
                        .text("Restart")
                        .style("font-size-24 fg-white");
                });

            // Main Menu Button
            parent
                .spawn((QuitButton, Interaction::default()))
                .styles(&vec![
                    "px-40 py-12",
                    "bg-rgba(0.3,0.3,0.3,0.9)",
                    "outline-width-2 outline-rgb(0.5,0.5,0.5)",
                ])
                .with_children(|parent| {
                    parent
                        .spawn_empty()
                        .text("Main Menu")
                        .style("font-size-24 fg-white");
                });
        });
}

/// Cleanup main menu UI
pub(super) fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUI>>) {
    info!(
        "Cleaning up main menu UI (found {} entities)",
        query.iter().count()
    );
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Cleanup game over UI
pub(super) fn cleanup_game_over(mut commands: Commands, query: Query<Entity, With<GameOverUI>>) {
    info!(
        "Cleaning up game over UI (found {} entities)",
        query.iter().count()
    );
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Handle button interactions
pub(super) fn handle_menu_buttons(
    new_game_query: Query<&Interaction, (With<NewGameButton>, Changed<Interaction>)>,
    restart_query: Query<&Interaction, (With<RestartButton>, Changed<Interaction>)>,
    quit_query: Query<&Interaction, (With<QuitButton>, Changed<Interaction>)>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<bevy::app::AppExit>,
) {
    // Check New Game button
    if let Ok(interaction) = new_game_query.single() {
        if *interaction == Interaction::Pressed {
            info!("New Game button pressed! Transitioning to Playing state");
            next_state.set(GameState::Playing);
            return;
        }
    }

    // Check Restart button
    if let Ok(interaction) = restart_query.single() {
        if *interaction == Interaction::Pressed {
            info!("Restart button pressed! Transitioning to Playing state");
            next_state.set(GameState::Playing);
            return;
        }
    }

    // Check Quit button
    if let Ok(interaction) = quit_query.single() {
        if *interaction == Interaction::Pressed {
            // In main menu, quit the game
            // In game over, go to main menu
            if *current_state.get() == GameState::MainMenu {
                info!("Quit button pressed in MainMenu! Exiting game");
                exit.write(bevy::app::AppExit::Success);
            } else {
                info!("Quit button pressed in GameOver! Transitioning to MainMenu state");
                next_state.set(GameState::MainMenu);
            }
        }
    }
}

/// Add hover effects to buttons
pub(super) fn update_button_visuals(
    mut query: Query<(&Interaction, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, mut color) in query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::srgba(0.2, 0.2, 0.2, 0.9).into();
            }
            Interaction::Hovered => {
                *color = Color::srgba(0.4, 0.4, 0.4, 0.9).into();
            }
            Interaction::None => {
                *color = Color::srgba(0.3, 0.3, 0.3, 0.9).into();
            }
        }
    }
}
