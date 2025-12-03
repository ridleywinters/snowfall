use super::cvars::CVarRegistry;
use crate::actor::Actor;
use crate::hud::PlayerStats;
use bevy::prelude::*;

use super::cmd_add_gold::cmd_add_gold;
use super::cmd_add_stamina::cmd_add_stamina;
use super::cmd_do_damage::cmd_do_damage;
use super::cmd_getvar::cmd_getvar;
use super::cmd_listvars::cmd_listvars;
use super::cmd_quit::cmd_quit;
use super::cmd_savecvars::cmd_savecvars;
use super::cmd_setvar::cmd_setvar;

pub fn process_script(
    script: &str,
    stats: &mut ResMut<PlayerStats>,
    cvars: &mut ResMut<CVarRegistry>,
) -> Vec<String> {
    process_script_with_actor(script, stats, cvars, None)
}

pub fn process_script_with_actor(
    script: &str,
    stats: &mut ResMut<PlayerStats>,
    cvars: &mut ResMut<CVarRegistry>,
    mut actor: Option<&mut Actor>,
) -> Vec<String> {
    let mut output = Vec::new();

    for line in script.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Skip comment lines
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        let tokens = tokenize_command(trimmed);
        if tokens.is_empty() {
            continue;
        }

        // Convert to &str for compatibility with existing command handlers
        let tokens: Vec<&str> = tokens.iter().map(|s| s.as_str()).collect();

        // Dispatch to command handlers
        let command_output = match tokens[0] {
            "setvar" => cmd_setvar(&tokens, stats, cvars),
            "getvar" => cmd_getvar(&tokens, stats, cvars),
            "listvars" => cmd_listvars(&tokens, stats, cvars),
            "savecvars" => cmd_savecvars(&tokens, stats, cvars),
            "add_gold" => cmd_add_gold(&tokens, stats, cvars),
            "add_stamina" => cmd_add_stamina(&tokens, stats, cvars),
            "quit" => cmd_quit(&tokens, stats, cvars),
            "do_damage" => {
                if let Some(ref mut actor_ref) = actor {
                    cmd_do_damage(&tokens, actor_ref)
                } else {
                    "do_damage can only be used on actors".to_string()
                }
            }
            _ => format!("Unknown command: {}", tokens.join(" ")),
        };

        output.push(command_output);
    }

    output
}

/// Tokenize a command line, treating quoted strings as single tokens.
/// Handles basic C-like escape sequences within quoted strings.
fn tokenize_command(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '\\' {
                // Handle escape sequences
                if let Some(&next_ch) = chars.peek() {
                    chars.next(); // consume the escaped character
                    match next_ch {
                        'n' => current_token.push('\n'),
                        't' => current_token.push('\t'),
                        'r' => current_token.push('\r'),
                        '\\' => current_token.push('\\'),
                        '"' => current_token.push('"'),
                        '0' => current_token.push('\0'),
                        _ => {
                            // For unknown escape sequences, include both the backslash and character
                            current_token.push('\\');
                            current_token.push(next_ch);
                        }
                    }
                } else {
                    // Backslash at end of string
                    current_token.push('\\');
                }
            } else if ch == '"' {
                // End of quoted string
                in_quotes = false;
                tokens.push(current_token.clone());
                current_token.clear();
            } else {
                current_token.push(ch);
            }
        } else {
            if ch == '"' {
                // Start of quoted string
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
                in_quotes = true;
            } else if ch.is_whitespace() {
                // Token separator
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
            } else {
                current_token.push(ch);
            }
        }
    }

    // Push any remaining token
    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::tokenize_command;

    #[test]
    fn test_tokenize_simple_command() {
        let result = tokenize_command("setvar x 42");
        assert_eq!(result, vec!["setvar", "x", "42"]);
    }

    #[test]
    fn test_tokenize_with_quoted_string() {
        let result = tokenize_command(r#"setvar message "Hello World""#);
        assert_eq!(result, vec!["setvar", "message", "Hello World"]);
    }

    #[test]
    fn test_tokenize_with_multiple_spaces() {
        let result = tokenize_command("setvar   x    42");
        assert_eq!(result, vec!["setvar", "x", "42"]);
    }

    #[test]
    fn test_tokenize_quoted_string_with_newline() {
        let result = tokenize_command(r#"setvar msg "Line1\nLine2""#);
        assert_eq!(result, vec!["setvar", "msg", "Line1\nLine2"]);
    }

    #[test]
    fn test_tokenize_quoted_string_with_tab() {
        let result = tokenize_command(r#"setvar msg "col1\tcol2""#);
        assert_eq!(result, vec!["setvar", "msg", "col1\tcol2"]);
    }

    #[test]
    fn test_tokenize_quoted_string_with_escaped_quote() {
        let result = tokenize_command(r#"setvar msg "He said \"Hi\"""#);
        assert_eq!(result, vec!["setvar", "msg", r#"He said "Hi""#]);
    }

    #[test]
    fn test_tokenize_quoted_string_with_escaped_backslash() {
        let result = tokenize_command(r#"setvar path "C:\\Users\\test""#);
        assert_eq!(result, vec!["setvar", "path", r"C:\Users\test"]);
    }

    #[test]
    fn test_tokenize_quoted_string_with_carriage_return() {
        let result = tokenize_command(r#"setvar msg "text\rmore""#);
        assert_eq!(result, vec!["setvar", "msg", "text\rmore"]);
    }

    #[test]
    fn test_tokenize_quoted_string_with_null() {
        let result = tokenize_command(r#"setvar msg "text\0null""#);
        assert_eq!(result, vec!["setvar", "msg", "text\0null"]);
    }

    #[test]
    fn test_tokenize_quoted_string_with_unknown_escape() {
        let result = tokenize_command(r#"setvar msg "text\x""#);
        assert_eq!(result, vec!["setvar", "msg", r"text\x"]);
    }

    #[test]
    fn test_tokenize_empty_quoted_string() {
        let result = tokenize_command(r#"setvar msg """#);
        assert_eq!(result, vec!["setvar", "msg", ""]);
    }

    #[test]
    fn test_tokenize_multiple_quoted_strings() {
        let result = tokenize_command(r#"cmd "arg1" "arg2" "arg3""#);
        assert_eq!(result, vec!["cmd", "arg1", "arg2", "arg3"]);
    }

    #[test]
    fn test_tokenize_mixed_quoted_and_unquoted() {
        let result = tokenize_command(r#"setvar x "Hello World" 42"#);
        assert_eq!(result, vec!["setvar", "x", "Hello World", "42"]);
    }

    #[test]
    fn test_tokenize_quoted_string_at_start() {
        let result = tokenize_command(r#""quoted" arg1 arg2"#);
        assert_eq!(result, vec!["quoted", "arg1", "arg2"]);
    }

    #[test]
    fn test_tokenize_empty_string() {
        let result = tokenize_command("");
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_tokenize_only_whitespace() {
        let result = tokenize_command("   ");
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_tokenize_escaped_quote_in_string() {
        let result = tokenize_command(r#"setvar msg "text\"more""#);
        assert_eq!(result, vec!["setvar", "msg", r#"text"more"#]);
    }
}
