mod cmd_add_gold;
mod cmd_add_stamina;
mod cmd_do_damage;
mod cmd_getvar;
mod cmd_listvars;
mod cmd_quit;
mod cmd_savecvars;
mod cmd_setvar;
mod cvars;
mod process_script;
mod scripting_plugin;

#[cfg(test)]
mod cmd_setvar_test;
#[cfg(test)]
mod cvars_test;

pub use cvars::*;
pub use process_script::*;
pub use scripting_plugin::ScriptingPlugin;
