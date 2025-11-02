use crate::console_variables::ConsoleVariableRegistry;
use crate::script::process_script;
use crate::ui::PlayerStats;
use bevy::{
    ecs::relationship::{RelatedSpawnerCommands, Relationship},
    log,
    prelude::*,
};
use regex::Regex;
use std::sync::LazyLock;

#[derive(Resource)]
pub struct ConsoleState {
    pub visible: bool,
    pub input_text: String,
    pub cursor_position: usize, // Cursor position in the input text (in chars, not bytes)
    pub log: Vec<String>,
    pub command_history: Vec<String>, // Stores only commands (not output)
    pub history_index: Option<usize>, // Current position in command history
    pub key_repeat_timer: f32,        // Timer for key repeat
    pub key_repeat_initial_delay: f32, // Initial delay before repeat starts
    pub key_repeat_rate: f32,         // Time between repeats once started
}

impl Default for ConsoleState {
    fn default() -> Self {
        Self {
            visible: false,
            input_text: String::new(),
            cursor_position: 0,
            log: Vec::new(),
            command_history: Vec::new(),
            history_index: None,
            key_repeat_timer: 0.0,
            key_repeat_initial_delay: 0.3, // initial delay (in seconds)
            key_repeat_rate: 0.015,        // time between repeats (in seconds)
        }
    }
}

#[derive(Component)]
pub struct ConsoleContainer;

#[derive(Component)]
pub struct ConsoleHistoryText;

#[derive(Component)]
pub struct ConsoleHistoryScroll;

#[derive(Component)]
pub struct ConsoleInputText;

fn spawn_with<'a>(e: &'a mut EntityCommands<'a>, styles: Vec<&str>) -> &'a mut EntityCommands<'a> {
    node_style(e, styles.join(" ").as_str());
    e
}

fn styled_root_with<F>(spawner: &mut Commands, styles: Vec<&str>, callback: F)
where
    F: FnOnce(&mut EntityCommands),
{
    let mut entity_commands = spawner.spawn_empty();
    node_style(&mut entity_commands, styles.join(" ").as_str());
    callback(&mut entity_commands);
}

fn styled_child_with<'w, R, F>(
    spawner: &mut RelatedSpawnerCommands<'w, R>,
    styles: Vec<&str>,
    callback: F,
) where
    R: Relationship + 'static,
    F: FnOnce(&mut EntityCommands),
{
    let mut entity_commands = spawner.spawn_empty();
    node_style(&mut entity_commands, styles.join(" ").as_str());
    callback(&mut entity_commands);
}

pub fn startup_console(mut commands: Commands) {
    // Initialize console state
    commands.insert_resource(ConsoleState::default());

    // Console overlay (initially hidden)
    styled_root_with(
        &mut commands,
        vec![
            "width-100% height-50% absolute top0 left0 flex-col p8 display-none",
            "z1000",
            "bg-srgba-(0.0,0.0,0.0,0.925)",
        ],
        |root| {
            root.insert(ConsoleContainer).with_children(|parent| {
                // Scrollable history area container
                styled_child_with(
                    parent,
                    vec!["flex-col grow1 scroll-y", "bg-rgba(0.1,0.1,0.1,0.3)"],
                    |parent| {
                        parent.insert(ConsoleHistoryScroll).with_children(|parent| {
                            // History text
                            parent.spawn((
                                Text::new(""),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.8, 0.8, 0.6)),
                                Node {
                                    padding: UiRect::all(Val::Px(8.0)),
                                    ..default()
                                },
                                ConsoleHistoryText,
                            ));
                        });
                    },
                );

                // Input prompt
                spawn_with(
                    &mut parent.spawn_empty(), //
                    vec!["flex-row-center gap8 mt8"],
                )
                .with_children(|parent| {
                    // Prompt symbol
                    parent.spawn((
                        Text::new("> "),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Input text
                    parent.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        ConsoleInputText,
                    ));
                });
            });
        },
    );
}

struct StyledBundle {
    node: Node,
    z_index: Option<ZIndex>,
    background_color: Option<BackgroundColor>,
}

enum StyleHandler {
    Void(fn(&mut StyledBundle)),
    I32(fn(&mut StyledBundle, i32)),
    F32F32F32F32(fn(&mut StyledBundle, f32, f32, f32, f32)),
}

static COMPILED_PATTERNS: LazyLock<Vec<(Regex, StyleHandler)>> = LazyLock::new(|| {
    use StyleHandler::*;
    let patterns: Vec<(&str, StyleHandler)> = vec![
        //
        // Positioning
        //
        (
            "absolute",
            Void(|b| {
                b.node.position_type = PositionType::Absolute;
            }),
        ),
        (
            r"top(\d+)",
            I32(|b, v| {
                b.node.top = Val::Px(v as f32);
            }),
        ),
        (
            r"left(\d+)",
            I32(|b, v| {
                b.node.left = Val::Px(v as f32);
            }),
        ),
        (
            r"bottom(\d+)",
            I32(|b, v| {
                b.node.bottom = Val::Px(v as f32);
            }),
        ),
        (
            r"right(\d+)",
            I32(|b, v| {
                b.node.right = Val::Px(v as f32);
            }),
        ),
        (
            r"width-(\d+)",
            I32(|b, v| {
                b.node.width = Val::Px(v as f32);
            }),
        ),
        (
            r"width-(\d+)%",
            I32(|b, v| {
                b.node.width = Val::Percent(v as f32);
            }),
        ),
        (
            r"height-(\d+)",
            I32(|b, v| {
                b.node.height = Val::Px(v as f32);
            }),
        ),
        (
            r"height-(\d+)%",
            I32(|b, v| {
                b.node.height = Val::Percent(v as f32);
            }),
        ),
        (
            r"z(\d)+",
            I32(|b, v| {
                b.z_index = Some(ZIndex(v));
            }),
        ),
        //
        // Display
        //
        (
            "display-none",
            Void(|b| {
                b.node.display = Display::None;
            }),
        ),
        //
        // Flex related
        //
        (
            "flex-row",
            Void(|b| {
                b.node.flex_direction = FlexDirection::Row;
            }),
        ),
        (
            "flex-row-center",
            Void(|b| {
                b.node.flex_direction = FlexDirection::Row;
                b.node.align_items = AlignItems::Center;
            }),
        ),
        (
            "flex-col",
            Void(|b| {
                b.node.flex_direction = FlexDirection::Column;
            }),
        ),
        (
            r"gap(\d+)",
            I32(|b, v| {
                b.node.column_gap = Val::Px(v as f32);
                b.node.row_gap = Val::Px(v as f32);
            }),
        ),
        (
            r"grow(\d+)",
            I32(|b, v| {
                b.node.flex_grow = v as f32;
            }),
        ),
        //
        // Overflow
        //
        (
            "scroll-y",
            Void(|b| {
                b.node.overflow = Overflow::scroll_y();
            }),
        ),
        //
        // Margins
        //
        (
            r"mt(\d+)",
            I32(|b, v| b.node.margin = UiRect::top(Val::Px(v as f32))),
        ),
        (
            r"mb(\d+)",
            I32(|b, v| b.node.margin = UiRect::bottom(Val::Px(v as f32))),
        ),
        (
            r"ml(\d+)",
            I32(|b, v| b.node.margin = UiRect::left(Val::Px(v as f32))),
        ),
        (
            r"mr(\d+)",
            I32(|b, v| b.node.margin = UiRect::right(Val::Px(v as f32))),
        ),
        (
            r"mx(\d+)",
            I32(|b, v| b.node.margin = UiRect::horizontal(Val::Px(v as f32))),
        ),
        (
            r"my(\d+)",
            I32(|b, v| b.node.margin = UiRect::vertical(Val::Px(v as f32))),
        ),
        (
            r"m(\d+)",
            I32(|b, v| b.node.margin = UiRect::all(Val::Px(v as f32))),
        ),
        //
        // Padding
        //
        (
            r"pt(\d+)",
            I32(|b, v| b.node.padding = UiRect::top(Val::Px(v as f32))),
        ),
        (
            r"pb(\d+)",
            I32(|b, v| b.node.padding = UiRect::bottom(Val::Px(v as f32))),
        ),
        (
            r"pl(\d+)",
            I32(|b, v| b.node.padding = UiRect::left(Val::Px(v as f32))),
        ),
        (
            r"pr(\d+)",
            I32(|b, v| b.node.padding = UiRect::right(Val::Px(v as f32))),
        ),
        (
            r"px(\d+)",
            I32(|b, v| b.node.padding = UiRect::horizontal(Val::Px(v as f32))),
        ),
        (
            r"py(\d+)",
            I32(|b, v| b.node.padding = UiRect::vertical(Val::Px(v as f32))),
        ),
        (
            r"p(\d+)",
            I32(|b, v| b.node.padding = UiRect::all(Val::Px(v as f32))),
        ),
        //
        // Backgrounds
        //
        (
            r"bg-srgba-\(([\d\.]+),([\d\.]+),([\d\.]+),([\d\.]+)\)",
            F32F32F32F32(|bundle, r, g, b, a| {
                let color = Color::srgba(r, g, b, a);
                bundle.background_color = Some(BackgroundColor(color));
            }),
        ),
    ];

    let mut compiled = Vec::new();
    for (pattern, handler) in patterns {
        if let Ok(regex) = Regex::new(&format!("^{}$", pattern)) {
            compiled.push((regex, handler));
        } else {
            log::warn!("Invalid regex pattern in console styles: {}", pattern);
        }
    }

    compiled
});

/// Uses a tailwind-like shorthand to allow for more concise UI definitions
fn node_style(commands: &mut EntityCommands, sl: &str) {
    let mut bundle = StyledBundle {
        node: Node { ..default() },
        z_index: None,
        background_color: None,
    };

    let tokens: Vec<&str> = sl.split_whitespace().collect();
    for token in tokens {
        let mut matched = false;

        for (regex, handler) in COMPILED_PATTERNS.iter() {
            use StyleHandler::*;
            let Some(captures) = regex.captures(token) else {
                continue;
            };
            matched = true;

            // Reminder first capture group is the whole match so all the length
            // checks are +1 of the number of arguments/sub-groups expected.
            match handler {
                Void(func) => {
                    if captures.len() != 1 {
                        log::warn!("Unexpected capture group for style: {}", token);
                        break;
                    }
                    func(&mut bundle);
                }
                I32(func) => {
                    if captures.len() != 2 {
                        log::warn!("No capture group for I32 style: {}", token);
                        break;
                    }
                    let Ok(value) = captures[1].parse::<i32>() else {
                        log::warn!("Invalid number in div style: {}", token);
                        break;
                    };
                    func(&mut bundle, value);
                }
                F32F32F32F32(func) => {
                    if captures.len() != 5 {
                        log::warn!(
                            "Incorrect number of capture groups for F32F32F32F32 style: {}",
                            token
                        );
                        break;
                    }
                    let Ok(v1) = captures[1].parse::<f32>() else {
                        log::warn!("Invalid first float in div style: {}", token);
                        break;
                    };
                    let Ok(v2) = captures[2].parse::<f32>() else {
                        log::warn!("Invalid second float in div style: {}", token);
                        break;
                    };
                    let Ok(v3) = captures[3].parse::<f32>() else {
                        log::warn!("Invalid third float in div style: {}", token);
                        break;
                    };
                    let Ok(v4) = captures[4].parse::<f32>() else {
                        log::warn!("Invalid fourth float in div style: {}", token);
                        break;
                    };
                    func(&mut bundle, v1, v2, v3, v4);
                }
            }
        }
        if !matched {
            log::warn!("Unknown div style: {}", token);
        }
    }

    commands.insert(bundle.node);
    if let Some(z_index) = bundle.z_index {
        commands.insert(z_index);
    }
    if let Some(background_color) = bundle.background_color {
        commands.insert(background_color);
    }
}

pub fn update_console_toggle(
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
        }
    }
}

const MAX_HISTORY_LINES: usize = 200;

pub fn update_console_input(
    time: Res<Time>,
    mut char_events: MessageReader<bevy::input::keyboard::KeyboardInput>,
    input: Res<ButtonInput<KeyCode>>,
    mut console_state: ResMut<ConsoleState>,
    mut stats: ResMut<PlayerStats>,
    mut cvars: ResMut<ConsoleVariableRegistry>,
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
    if input.just_pressed(KeyCode::ArrowUp) {
        if !console_state.command_history.is_empty() {
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
    }

    // Handle Down arrow - navigate to next command in history
    if input.just_pressed(KeyCode::ArrowDown) {
        if let Some(idx) = console_state.history_index {
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
    }

    // Handle Left arrow - move cursor left
    if input.just_pressed(KeyCode::ArrowLeft) || should_handle_arrow_left {
        if console_state.cursor_position > 0 {
            console_state.cursor_position -= 1;
        }
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
        let words: Vec<&str> = console_state.input_text.split_whitespace().collect();

        // Check if first word is setvar or getvar
        if words.len() >= 1 && (words[0] == "setvar" || words[0] == "getvar") {
            // Check if there's a second word (partial variable name)
            if words.len() >= 2 && !words[1].is_empty() {
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
        }
    }

    // Handle Enter key - submit command
    if input.just_pressed(KeyCode::Enter) {
        if !console_state.input_text.is_empty() {
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
    }

    // Handle Backspace - delete character before cursor
    if input.just_pressed(KeyCode::Backspace) || should_handle_backspace {
        if console_state.cursor_position > 0 {
            let char_indices: Vec<_> = console_state.input_text.char_indices().collect();
            if console_state.cursor_position <= char_indices.len() {
                let byte_pos = char_indices[console_state.cursor_position - 1].0;
                console_state.input_text.remove(byte_pos);
                console_state.cursor_position -= 1;
                console_state.history_index = None;
            }
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
        if event.state.is_pressed() {
            if let bevy::input::keyboard::Key::Character(ref s) = event.logical_key {
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

pub fn update_console_scroll(
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
