use super::ConsoleState;
use crate::internal::*;
use crate::scripting::CVarRegistry;
use crate::scripting::process_script;
use crate::ui::PlayerStats;
use crate::ui_styles::EntityCommandsUIExt;

//=============================================================================
// Console UI Components
//=============================================================================

#[derive(Component)]
pub(super) struct ConsoleContainer;

#[derive(Component)]
pub(super) struct ConsoleHistoryText;

#[derive(Component)]
pub(super) struct ConsoleHistoryScroll;

#[derive(Component)]
pub(super) struct ConsoleInputText;

//=============================================================================
// Console UI Systems
//=============================================================================

pub(super) fn startup_console(mut commands: Commands) {
    // Initialize console state
    commands.insert_resource(ConsoleState::default());

    // Console overlay (initially hidden)
    commands
        .spawn((GamePlayEntity, ConsoleContainer))
        .styles(&vec![
            "display-none",
            "width-100% height-50% absolute top-0 left-0 flex-col p8",
            "z1000",
            "bg-rgba(0.0,0.0,0.0,0.925)",
        ])
        .with_children(|parent| {
            parent
                .spawn(ConsoleHistoryScroll)
                .styles(&vec![
                    "flex-col grow1 scroll-y", //
                    "bg-rgba(0.1,0.1,0.1,0.3)",
                ])
                .with_children(|parent| {
                    parent // History text
                        .spawn(ConsoleHistoryText)
                        .text("")
                        .styles(&vec![
                            "p8 font-size-16", //
                            "fg-rgba(0.8,0.8,0.6,1.0)",
                        ]);
                });
            parent // Input prompt
                .spawn_empty()
                .styles(&vec!["flex-row-center gap8 mt8"])
                .with_children(|parent| {
                    parent
                        .spawn_empty()
                        .text("> ")
                        .styles(&vec!["fg-white font-size-16"]);
                    parent
                        .spawn(ConsoleInputText)
                        .text("")
                        .styles(&vec!["fg-white font-size-16"]);
                });
        });
}

pub(super) fn update_console_toggle(
    input: Res<ButtonInput<KeyCode>>,
    mut console_state: ResMut<ConsoleState>,
    mut console_query: Query<&mut Node, With<ConsoleContainer>>,
) {
    // Toggle console with ` or ~
    if input.just_pressed(KeyCode::Backquote) {
        console_state.visible = !console_state.visible;

        // Update display
        if let Ok(mut node) = console_query.single_mut() {
            node.display = if console_state.visible {
                Display::Flex
            } else {
                Display::None
            };
        } else {
            error!("Console container not found");
        }
    }
}

pub(super) fn update_console_input(
    time: Res<Time>,
    mut char_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    input: Res<ButtonInput<KeyCode>>,
    mut console_state: ResMut<ConsoleState>,
    mut stats: ResMut<PlayerStats>,
    mut cvars: ResMut<CVarRegistry>,
    mut input_text_query: Query<&mut Text, With<ConsoleInputText>>,
    mut history_text_query: Query<&mut Text, (With<ConsoleHistoryText>, Without<ConsoleInputText>)>,
) {
    if !console_state.visible {
        return;
    }

    let dt = time.delta_secs();
    let mut should_handle_backspace = false;
    let mut should_handle_delete = false;
    let mut should_handle_arrow_left = false;
    let mut should_handle_arrow_right = false;

    // Check if any repeatable key is currently pressed
    let any_repeat_key_pressed = input.pressed(KeyCode::Backspace)
        || input.pressed(KeyCode::Delete)
        || input.pressed(KeyCode::ArrowLeft)
        || input.pressed(KeyCode::ArrowRight);

    if any_repeat_key_pressed {
        console_state.key_repeat_timer += dt;
        if console_state.key_repeat_timer >= console_state.key_repeat_initial_delay {
            let time_since_initial =
                console_state.key_repeat_timer - console_state.key_repeat_initial_delay;
            let repeat_count = (time_since_initial / console_state.key_repeat_rate).floor() as i32;
            if repeat_count > 0 {
                // Reset timer to account for the repeat we're about to process
                console_state.key_repeat_timer = console_state.key_repeat_initial_delay
                    + (repeat_count as f32 * console_state.key_repeat_rate);
                if input.pressed(KeyCode::Backspace) {
                    should_handle_backspace = true;
                }
                if input.pressed(KeyCode::Delete) {
                    should_handle_delete = true;
                }
                if input.pressed(KeyCode::ArrowLeft) {
                    should_handle_arrow_left = true;
                }
                if input.pressed(KeyCode::ArrowRight) {
                    should_handle_arrow_right = true;
                }
            }
        }
    } else {
        // Reset timer when no repeatable key is pressed
        console_state.key_repeat_timer = 0.0;
    }

    // Handle Up arrow - navigate to previous command in history
    if input.just_pressed(KeyCode::ArrowUp) && !console_state.command_history.is_empty() {
        let new_index = match console_state.history_index {
            None => Some(console_state.command_history.len() - 1),
            Some(idx) if idx > 0 => Some(idx - 1),
            Some(idx) => Some(idx),
        };

        if let Some(idx) = new_index {
            console_state.history_index = new_index;
            console_state.input_text = console_state.command_history[idx].clone();
            console_state.cursor_position = console_state.input_text.chars().count();
        }
    }

    // Handle Down arrow - navigate to next command in history
    if input.just_pressed(KeyCode::ArrowDown)
        && let Some(idx) = console_state.history_index
    {
        if idx < console_state.command_history.len() - 1 {
            console_state.history_index = Some(idx + 1);
            console_state.input_text = console_state.command_history[idx + 1].clone();
            console_state.cursor_position = console_state.input_text.chars().count();
        } else {
            // At the end of history, clear input
            console_state.history_index = None;
            console_state.input_text.clear();
            console_state.cursor_position = 0;
        }
    }

    // Handle Left arrow - move cursor left
    if (input.just_pressed(KeyCode::ArrowLeft) || should_handle_arrow_left)
        && console_state.cursor_position > 0
    {
        console_state.cursor_position -= 1;
    }

    // Handle Right arrow - move cursor right
    if input.just_pressed(KeyCode::ArrowRight) || should_handle_arrow_right {
        let text_len = console_state.input_text.chars().count();
        if console_state.cursor_position < text_len {
            console_state.cursor_position += 1;
        }
    }

    // Handle Home - move cursor to start
    if input.just_pressed(KeyCode::Home) {
        console_state.cursor_position = 0;
    }

    // Handle End - move cursor to end
    if input.just_pressed(KeyCode::End) {
        console_state.cursor_position = console_state.input_text.chars().count();
    }

    // Handle Tab - autocomplete cvar names for setvar/getvar commands
    if input.just_pressed(KeyCode::Tab) {
        handle_autocomplete(&mut console_state, &cvars);
    }

    // Handle Enter key - submit command
    if input.just_pressed(KeyCode::Enter) && !console_state.input_text.is_empty() {
        // Echo the command to history
        let command = console_state.input_text.clone();
        console_state.log.push(format!(": {}", command));

        // Add to command history (for up/down arrow navigation)
        console_state.command_history.push(command.clone());
        console_state.history_index = None; // Reset history navigation

        // Process the command and get output
        let output = process_script(&command, &mut stats, &mut cvars);
        for line in output {
            console_state.log.push(format!("  {}", line));
        }

        let history_len = console_state.log.len();
        if history_len > MAX_HISTORY_LINES {
            console_state.log.drain(0..history_len - MAX_HISTORY_LINES);
        }

        // Update history display
        if let Ok(mut text) = history_text_query.single_mut() {
            **text = console_state.log.join("\n");
        }

        // Clear input and reset cursor
        console_state.input_text.clear();
        console_state.cursor_position = 0;
    }

    // Handle Backspace - delete character before cursor
    if (input.just_pressed(KeyCode::Backspace) || should_handle_backspace)
        && console_state.cursor_position > 0
    {
        let char_indices: Vec<_> = console_state.input_text.char_indices().collect();
        if console_state.cursor_position <= char_indices.len() {
            let byte_pos = char_indices[console_state.cursor_position - 1].0;
            console_state.input_text.remove(byte_pos);
            console_state.cursor_position -= 1;
            console_state.history_index = None;
        }
    }

    // Handle Delete - delete character at cursor
    if input.just_pressed(KeyCode::Delete) || should_handle_delete {
        let char_indices: Vec<_> = console_state.input_text.char_indices().collect();
        if console_state.cursor_position < char_indices.len() {
            let byte_pos = char_indices[console_state.cursor_position].0;
            console_state.input_text.remove(byte_pos);
            console_state.history_index = None;
        }
    }

    // Handle Space key explicitly
    if input.just_pressed(KeyCode::Space) {
        let char_indices: Vec<_> = console_state.input_text.char_indices().collect();
        let byte_pos = if console_state.cursor_position < char_indices.len() {
            char_indices[console_state.cursor_position].0
        } else {
            console_state.input_text.len()
        };
        console_state.input_text.insert(byte_pos, ' ');
        console_state.cursor_position += 1;
        console_state.history_index = None;
    }

    // Handle character input
    for event in char_events.read() {
        if event.state.is_pressed()
            && let bevy::input::keyboard::Key::Character(ref s) = event.logical_key
        {
            // Ignore backtick to prevent it being added when opening console
            // Also ignore space since we handle it explicitly above
            if s.as_str() != "`" && s.as_str() != "~" && s.as_str() != " " {
                let char_indices: Vec<_> = console_state.input_text.char_indices().collect();
                let byte_pos = if console_state.cursor_position < char_indices.len() {
                    char_indices[console_state.cursor_position].0
                } else {
                    console_state.input_text.len()
                };
                console_state.input_text.insert_str(byte_pos, s.as_str());
                console_state.cursor_position += s.chars().count();
                console_state.history_index = None;
            }
        }
    }

    // Update input text display with cursor
    if let Ok(mut text) = input_text_query.single_mut() {
        let char_indices: Vec<_> = console_state.input_text.char_indices().collect();

        // Insert cursor character (█) at cursor position
        let cursor = "|".to_string();
        let display_text = if char_indices.is_empty() {
            cursor.clone()
        } else if console_state.cursor_position >= char_indices.len() {
            format!("{}{}", console_state.input_text, cursor)
        } else {
            let byte_pos = char_indices[console_state.cursor_position].0;
            let before = &console_state.input_text[..byte_pos];
            let after = &console_state.input_text[byte_pos..];
            format!("{}{}{}", before, cursor, after)
        };

        **text = display_text;
    }
}

/// Automatically scrolls the console history to the bottom when the console is visible.
/// This ensures that the most recent log entries are always visible to the user.
pub(super) fn update_console_scroll(
    console_state: Res<ConsoleState>,
    mut scroll_query: Query<&mut ScrollPosition, With<ConsoleHistoryScroll>>,
) {
    if !console_state.visible {
        return;
    }

    // Auto-scroll to bottom when console is visible
    if let Ok(mut scroll_position) = scroll_query.single_mut() {
        scroll_position.y = f32::MAX; // Scroll to bottom
    }
}

//=============================================================================
// Helper Functions
//=============================================================================

/// Handle Tab completion for cvar names in setvar/getvar commands
fn handle_autocomplete(console_state: &mut ConsoleState, cvars: &CVarRegistry) {
    let words: Vec<&str> = console_state.input_text.split_whitespace().collect();

    // Check if first word is setvar or getvar
    if words.is_empty() || (words[0] != "setvar" && words[0] != "getvar") {
        return;
    }

    // Check if there's a second word (partial variable name)
    if words.len() < 2 || words[1].is_empty() {
        return;
    }

    let current_word = words[1];

    // Get all the cvars; they are already in alphabetical order
    let all_cvars = cvars.list();

    let is_exact_match = all_cvars.iter().any(|(name, _)| name == current_word);
    let matching_cvar: Option<String> = if is_exact_match {
        // Current word is exact match - find next cvar in the list
        let mut found_current = false;
        let mut next_cvar: Option<String> = None;

        for (name, _) in &all_cvars {
            if found_current {
                next_cvar = Some(name.clone());
                break;
            }
            if name == current_word {
                found_current = true;
            }
        }

        // If we didn't find a next one (we were at the end), wrap to first
        if next_cvar.is_none() && !all_cvars.is_empty() {
            next_cvar = Some(all_cvars[0].0.clone());
        }

        next_cvar
    } else {
        // Not an exact match - find first cvar that starts with this prefix
        let mut first_match: Option<String> = None;

        for (name, _) in &all_cvars {
            if name.starts_with(current_word) {
                first_match = Some(name.clone());
                break;
            }
        }

        first_match
    };

    // If we found a match, replace the partial name with the full name
    if let Some(full_name) = matching_cvar {
        // Reconstruct the command with the completed variable name
        let mut new_text = format!("{} {}", words[0], full_name);

        // If there are more words (like a value for setvar), append them
        if words.len() > 2 {
            for word in &words[2..] {
                new_text.push(' ');
                new_text.push_str(word);
            }
        }

        console_state.input_text = new_text;
        console_state.cursor_position = console_state.input_text.chars().count();
        console_state.history_index = None;
    }
}

//=============================================================================
// Tests
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::CVarValue;

    #[test]
    fn test_autocomplete_partial_match() {
        let mut console_state = ConsoleState::default();
        let mut cvars = CVarRegistry::default();

        // Add some test cvars
        cvars.init("player_speed", CVarValue::F32(1.0)).unwrap();
        cvars.init("player_health", CVarValue::Int32(100)).unwrap();
        cvars.init("enemy_speed", CVarValue::F32(0.5)).unwrap();

        // Test partial match with "setvar"
        console_state.input_text = "setvar player".to_string();
        console_state.cursor_position = console_state.input_text.len();

        handle_autocomplete(&mut console_state, &cvars);

        // Should complete to first matching cvar (alphabetically)
        assert_eq!(console_state.input_text, "setvar player_health");
        assert_eq!(
            console_state.cursor_position,
            console_state.input_text.len()
        );
    }

    #[test]
    fn test_autocomplete_exact_match_cycles() {
        let mut console_state = ConsoleState::default();
        let mut cvars = CVarRegistry::default();

        cvars.init("player_speed", CVarValue::F32(1.0)).unwrap();
        cvars.init("player_health", CVarValue::Int32(100)).unwrap();
        cvars.init("enemy_speed", CVarValue::F32(0.5)).unwrap();

        // Start with exact match
        console_state.input_text = "getvar player_health".to_string();
        console_state.cursor_position = console_state.input_text.len();

        handle_autocomplete(&mut console_state, &cvars);

        // Should cycle to next cvar starting with "player_"
        assert_eq!(console_state.input_text, "getvar player_speed");
    }

    #[test]
    fn test_autocomplete_wraps_to_first() {
        let mut console_state = ConsoleState::default();
        let mut cvars = CVarRegistry::default();

        cvars.init("aaa", CVarValue::Int32(1)).unwrap();
        cvars.init("bbb", CVarValue::Int32(2)).unwrap();
        cvars.init("ccc", CVarValue::Int32(3)).unwrap();

        // Start at last cvar
        console_state.input_text = "setvar ccc".to_string();
        console_state.cursor_position = console_state.input_text.len();

        handle_autocomplete(&mut console_state, &cvars);

        // Should wrap to first cvar
        assert_eq!(console_state.input_text, "setvar aaa");
    }

    #[test]
    fn test_autocomplete_preserves_value() {
        let mut console_state = ConsoleState::default();
        let mut cvars = CVarRegistry::default();

        cvars.init("player_speed", CVarValue::F32(1.0)).unwrap();
        cvars.init("player_health", CVarValue::Int32(100)).unwrap();

        // Test with value after cvar name
        console_state.input_text = "setvar player 5.0".to_string();
        console_state.cursor_position = console_state.input_text.len();

        handle_autocomplete(&mut console_state, &cvars);

        // Should complete cvar name but preserve the value
        assert_eq!(console_state.input_text, "setvar player_health 5.0");
    }

    #[test]
    fn test_autocomplete_no_match() {
        let mut console_state = ConsoleState::default();
        let mut cvars = CVarRegistry::default();

        cvars.init("player_speed", CVarValue::F32(1.0)).unwrap();

        // Test with non-matching prefix
        console_state.input_text = "setvar enemy".to_string();
        let original_text = console_state.input_text.clone();
        console_state.cursor_position = console_state.input_text.len();

        handle_autocomplete(&mut console_state, &cvars);

        // Should not change anything
        assert_eq!(console_state.input_text, original_text);
    }

    #[test]
    fn test_autocomplete_ignores_non_setvar_getvar() {
        let mut console_state = ConsoleState::default();
        let mut cvars = CVarRegistry::default();

        cvars.init("test_var", CVarValue::Int32(1)).unwrap();

        // Test with different command
        console_state.input_text = "listvars test".to_string();
        let original_text = console_state.input_text.clone();
        console_state.cursor_position = console_state.input_text.len();

        handle_autocomplete(&mut console_state, &cvars);

        // Should not autocomplete for non-setvar/getvar commands
        assert_eq!(console_state.input_text, original_text);
    }

    #[test]
    fn test_autocomplete_empty_input() {
        let mut console_state = ConsoleState::default();
        let cvars = CVarRegistry::default();

        console_state.input_text = "".to_string();
        console_state.cursor_position = 0;

        handle_autocomplete(&mut console_state, &cvars);

        // Should not crash or change anything
        assert_eq!(console_state.input_text, "");
    }
}
