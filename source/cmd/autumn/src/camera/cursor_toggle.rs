/// Cursor lock toggle system
///
/// Handles toggling cursor grab/lock state with ESC key and manages
/// cursor visibility. Also responds to console state changes.
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use super::mouse_look_settings::MouseLookSettings;
use crate::console::ConsoleState;

/// System to handle cursor lock toggling with ESC key
pub fn toggle_cursor_lock(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_look: ResMut<MouseLookSettings>,
    mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    console_state: Res<ConsoleState>,
) {
    if console_state.visible {
        return;
    }

    // Toggle cursor lock with ESC key
    if keyboard.just_pressed(KeyCode::Escape) {
        mouse_look.cursor_locked = !mouse_look.cursor_locked;

        if let Ok(mut cursor) = cursor_query.single_mut() {
            if mouse_look.cursor_locked {
                cursor.grab_mode = CursorGrabMode::Locked;
                cursor.visible = false;
            } else {
                cursor.grab_mode = CursorGrabMode::None;
                cursor.visible = true;
            }
        }
    }
}

/// System to engage cursor lock when clicking outside UI
pub fn click_to_lock_cursor(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_look: ResMut<MouseLookSettings>,
    mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    ui_interaction_query: Query<&Interaction>,
    console_state: Res<ConsoleState>,
) {
    // Don't process if console is open or cursor is already locked
    if console_state.visible || mouse_look.cursor_locked {
        return;
    }

    // Check for left mouse click
    if mouse_button.just_pressed(MouseButton::Left) {
        // Check if click is not on any UI element
        let ui_clicked = ui_interaction_query.iter().any(|i| *i != Interaction::None);

        if !ui_clicked {
            // Engage cursor lock
            mouse_look.cursor_locked = true;

            if let Ok(mut cursor) = cursor_query.single_mut() {
                cursor.grab_mode = CursorGrabMode::Locked;
                cursor.visible = false;
            }
        }
    }
}

/// System to force unlock cursor when console opens
pub fn handle_console_cursor(
    console_state: Res<ConsoleState>,
    mouse_look: Res<MouseLookSettings>,
    mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    // When console opens, always unlock cursor
    if console_state.visible {
        if let Ok(mut cursor) = cursor_query.single_mut() {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        }
    } else {
        // When console closes, restore lock state if it was locked
        if mouse_look.cursor_locked {
            if let Ok(mut cursor) = cursor_query.single_mut() {
                cursor.grab_mode = CursorGrabMode::Locked;
                cursor.visible = false;
            }
        }
    }
}
