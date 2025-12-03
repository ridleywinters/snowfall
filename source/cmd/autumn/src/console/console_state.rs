use super::internal::*;

//=============================================================================
// Console State
//=============================================================================

#[derive(Resource)]
pub struct ConsoleState {
    pub visible: bool,
    pub input_text: String,
    pub cursor_position: usize, // Cursor position in the input text (in chars, not bytes)
    pub log: Vec<String>,
    pub command_history: Vec<String>, // Stores only commands (not output)
    pub history_index: Option<usize>, // Current position in command history

    // Manual implementation to handle key repeats in the console.
    // TODO: is there a a standard way to implement this so a manual implementation is
    // not necessary? Seems like this should be controlled on an OS level, not implemented
    // by this app.
    pub key_repeat_timer: f32,         // Timer for key repeat
    pub key_repeat_initial_delay: f32, // Initial delay before repeat starts
    pub key_repeat_rate: f32,          // Time between repeats once started
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
