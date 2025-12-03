#[cfg(test)]
mod tests {
    use super::super::cmd_setvar::cmd_setvar_worker;
    use super::super::cvars::{CVarRegistry, CVarValue};

    // Helper to verify type is preserved after parsing
    fn verify_type_match(original: &CVarValue, parsed: &CVarValue) -> bool {
        match (original, parsed) {
            (CVarValue::Bool(_), CVarValue::Bool(_)) => true,
            (CVarValue::F32(_), CVarValue::F32(_)) => true,
            (CVarValue::Int32(_), CVarValue::Int32(_)) => true,
            (CVarValue::String(_), CVarValue::String(_)) => true,
            _ => false,
        }
    }

    // Common test setup with multiple variables of different types
    fn setup_cvars() -> CVarRegistry {
        let mut cvars = CVarRegistry::new();
        cvars.init("bool_var", CVarValue::Bool(false)).unwrap();
        cvars.init("f32_var", CVarValue::F32(1.0)).unwrap();
        cvars.init("int32_var", CVarValue::Int32(0)).unwrap();
        cvars
            .init("string_var", CVarValue::String("old".to_string()))
            .unwrap();
        cvars
    }

    // Test tuple: (name, command_string, expected_value)
    type Test = (
        &'static str,
        &'static str,
        Option<(&'static str, CVarValue)>,
    );

    #[test]
    fn test_cmd_setvar_worker_table() {
        let tests: Vec<Test> = vec![
            // Success cases
            (
                "Setting bool var to true succeeds",
                "setvar bool_var true",
                Some(("bool_var", CVarValue::Bool(true))),
            ),
            (
                "Setting bool var to false succeeds",
                "setvar bool_var false",
                Some(("bool_var", CVarValue::Bool(false))),
            ),
            (
                "Setting f32 var to 3.14 succeeds",
                "setvar f32_var 3.14",
                Some(("f32_var", CVarValue::F32(3.14))),
            ),
            (
                "Setting int32 var to 42 succeeds",
                "setvar int32_var 42",
                Some(("int32_var", CVarValue::Int32(42))),
            ),
            (
                "Setting string var to new value succeeds",
                "setvar string_var new",
                Some(("string_var", CVarValue::String("new".to_string()))),
            ),
            // Error cases
            ("Setting nonexistent variable fails", "setvar x value", None),
            (
                "Calling setvar with insufficient args fails",
                "setvar bool_var",
                None,
            ),
            ("Setting bool var to 1 fails", "setvar bool_var 1", None),
            (
                "Setting f32 var to non-numeric value fails",
                "setvar f32_var notafloat",
                None,
            ),
            (
                "Setting int32 var to non-numeric value fails",
                "setvar int32_var notanint",
                None,
            ),
            // Type mismatch: int32 to other types
            (
                "Setting f32 var to integer value succeeds",
                "setvar f32_var 42",
                Some(("f32_var", CVarValue::F32(42.0))),
            ),
            (
                "Setting bool var to integer value fails",
                "setvar bool_var 42",
                None,
            ),
            (
                "Setting string var to integer value succeeds",
                "setvar string_var 42",
                Some(("string_var", CVarValue::String("42".to_string()))),
            ),
            // Type mismatch: f32 to other types
            (
                "Setting int32 var to float value fails",
                "setvar int32_var 3.14",
                None,
            ),
            (
                "Setting bool var to float value fails",
                "setvar bool_var 3.14",
                None,
            ),
            (
                "Setting string var to float value succeeds",
                "setvar string_var 3.14",
                Some(("string_var", CVarValue::String("3.14".to_string()))),
            ),
        ];

        for (name, cmd_str, verify) in tests {
            let mut cvars = setup_cvars();

            // Tokenize command string
            let tokens: Vec<&str> = cmd_str.split_whitespace().collect();

            // Run command (return value is for display only, not tested)
            cmd_setvar_worker(&tokens, &mut cvars);

            if let Some((var_name, expected_value)) = verify {
                let actual = cvars.get(var_name).expect(&format!(
                    "Test '{}': variable '{}' should exist",
                    name, var_name
                ));

                assert!(
                    verify_type_match(&expected_value, actual),
                    "Test '{}': variable '{}' type mismatch",
                    name,
                    var_name
                );

                match (expected_value, actual) {
                    (CVarValue::Bool(e), CVarValue::Bool(a)) => {
                        assert_eq!(e, *a, "Test '{}': Bool value mismatch", name)
                    }
                    (CVarValue::F32(e), CVarValue::F32(a)) => {
                        assert_eq!(e, *a, "Test '{}': F32 value mismatch", name)
                    }
                    (CVarValue::Int32(e), CVarValue::Int32(a)) => {
                        assert_eq!(e, *a, "Test '{}': Int32 value mismatch", name)
                    }
                    (CVarValue::String(e), CVarValue::String(a)) => {
                        assert_eq!(e, *a, "Test '{}': String value mismatch", name)
                    }
                    _ => panic!("Test '{}': unexpected type combination", name),
                }
            }
        }
    }
}
