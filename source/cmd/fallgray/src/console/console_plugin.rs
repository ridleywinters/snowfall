use super::console_ui::*;
use super::internal::*;

//=============================================================================
// Console Plugin
//=============================================================================

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app //
            .add_systems(
                OnEnter(GameState::Playing), //
                startup_console,
            )
            .add_systems(
                Update,
                (
                    update_console_toggle,
                    update_console_input,
                    update_console_scroll,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
