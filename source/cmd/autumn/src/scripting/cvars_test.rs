#[cfg(test)]
mod tests {
    use super::super::cvars::{CVarRegistry, CVarValue};

    #[test]
    fn test_cvarvalue_as_f32() {
        assert_eq!(CVarValue::F32(3.14).as_f32(), Some(3.14));
        assert_eq!(CVarValue::Int32(42).as_f32(), None);
        assert_eq!(CVarValue::Bool(true).as_f32(), None);
        assert_eq!(CVarValue::Bool(false).as_f32(), None);
        assert_eq!(CVarValue::String("2.5".to_string()).as_f32(), None);
    }

    #[test]
    fn test_cvarvalue_as_i32() {
        assert_eq!(CVarValue::F32(3.14).as_i32(), None);
        assert_eq!(CVarValue::Int32(42).as_i32(), Some(42));
        assert_eq!(CVarValue::Bool(true).as_i32(), None);
        assert_eq!(CVarValue::Bool(false).as_i32(), None);
        assert_eq!(CVarValue::String("99".to_string()).as_i32(), None);
    }

    #[test]
    fn test_cvarvalue_as_bool() {
        assert_eq!(CVarValue::Bool(true).as_bool(), Some(true));
        assert_eq!(CVarValue::Bool(false).as_bool(), Some(false));
        assert_eq!(CVarValue::Int32(1).as_bool(), None);
        assert_eq!(CVarValue::F32(1.0).as_bool(), None);
        assert_eq!(CVarValue::String("true".to_string()).as_bool(), None);
    }

    #[test]
    fn test_cvarvalue_as_string() {
        assert_eq!(CVarValue::F32(3.14).as_string(), "3.14");
        assert_eq!(CVarValue::Int32(42).as_string(), "42");
        assert_eq!(CVarValue::Bool(true).as_string(), "true");
        assert_eq!(CVarValue::Bool(false).as_string(), "false");
        assert_eq!(CVarValue::String("hello".to_string()).as_string(), "hello");
    }

    #[test]
    fn test_cvarvalue_display() {
        assert_eq!(format!("{}", CVarValue::F32(3.14)), "3.14");
        assert_eq!(format!("{}", CVarValue::Int32(42)), "42");
        assert_eq!(format!("{}", CVarValue::Bool(true)), "true");
        assert_eq!(format!("{}", CVarValue::Bool(false)), "false");
        assert_eq!(format!("{}", CVarValue::String("test".to_string())), "test");
    }

    #[test]
    fn test_is_valid_name() {
        // Valid names
        assert!(CVarRegistry::is_valid_name("var"));
        assert!(CVarRegistry::is_valid_name("myVar"));
        assert!(CVarRegistry::is_valid_name("my_var"));
        assert!(CVarRegistry::is_valid_name("weapon_anim_duration"));
        assert!(CVarRegistry::is_valid_name("player.health"));
        assert!(CVarRegistry::is_valid_name("g_speed"));
        assert!(CVarRegistry::is_valid_name("a1"));
        assert!(CVarRegistry::is_valid_name("test123"));
        assert!(CVarRegistry::is_valid_name("cl_fov"));

        // Invalid names
        assert!(!CVarRegistry::is_valid_name(""));
        assert!(!CVarRegistry::is_valid_name("123var"));
        assert!(!CVarRegistry::is_valid_name("_var"));
        assert!(!CVarRegistry::is_valid_name(".var"));
        assert!(!CVarRegistry::is_valid_name("my-var"));
        assert!(!CVarRegistry::is_valid_name("my var"));
        assert!(!CVarRegistry::is_valid_name("my@var"));
        assert!(!CVarRegistry::is_valid_name("my#var"));
    }

    #[test]
    fn test_registry_new() {
        let registry = CVarRegistry::new();
        assert!(!registry.exists("anything"));
    }

    #[test]
    fn test_init_float() {
        let mut registry = CVarRegistry::new();
        assert!(registry.init("speed", CVarValue::F32(5.0)).is_ok());
        assert!(registry.exists("speed"));
        assert_eq!(registry.get_f32("speed"), 5.0);
    }

    #[test]
    fn test_init_int() {
        let mut registry = CVarRegistry::new();
        assert!(registry.init("lives", CVarValue::Int32(3)).is_ok());
        assert!(registry.exists("lives"));
        assert_eq!(registry.get_i32("lives"), 3);
    }

    #[test]
    fn test_init_string() {
        let mut registry = CVarRegistry::new();
        assert!(
            registry
                .init("name", CVarValue::String("player".to_string()))
                .is_ok()
        );
        assert!(registry.exists("name"));
        assert_eq!(registry.get_string("name"), "player");
    }

    #[test]
    fn test_init_bool() {
        let mut registry = CVarRegistry::new();
        assert!(registry.init("enabled", CVarValue::Bool(true)).is_ok());
        assert!(registry.exists("enabled"));
        assert_eq!(registry.get_bool("enabled"), true);
    }

    #[test]
    fn test_init_bool_convenience() {
        let mut registry = CVarRegistry::new();
        registry.init_bool("flag", false);
        assert!(registry.exists("flag"));
        assert_eq!(registry.get_bool("flag"), false);
    }

    #[test]
    fn test_init_f32_convenience() {
        let mut registry = CVarRegistry::new();
        registry.init_f32("fov", 90.0);
        assert!(registry.exists("fov"));
        assert_eq!(registry.get_f32("fov"), 90.0);
    }

    #[test]
    fn test_init_invalid_name() {
        let mut registry = CVarRegistry::new();
        let result = registry.init("123invalid", CVarValue::Int32(1));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid variable name"));
    }

    #[test]
    fn test_init_duplicate() {
        let mut registry = CVarRegistry::new();
        registry.init("var", CVarValue::Int32(1)).unwrap();
        let result = registry.init("var", CVarValue::Int32(2));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_set_float() {
        let mut registry = CVarRegistry::new();
        registry.init("speed", CVarValue::F32(1.0)).unwrap();
        assert!(registry.set("speed", CVarValue::F32(2.5)).is_ok());
        assert_eq!(registry.get_f32("speed"), 2.5);
    }

    #[test]
    fn test_set_int() {
        let mut registry = CVarRegistry::new();
        registry.init("count", CVarValue::Int32(10)).unwrap();
        assert!(registry.set("count", CVarValue::Int32(20)).is_ok());
        assert_eq!(registry.get_i32("count"), 20);
    }

    #[test]
    fn test_set_string() {
        let mut registry = CVarRegistry::new();
        registry
            .init("name", CVarValue::String("old".to_string()))
            .unwrap();
        assert!(
            registry
                .set("name", CVarValue::String("new".to_string()))
                .is_ok()
        );
        assert_eq!(registry.get_string("name"), "new");
    }

    #[test]
    fn test_set_bool() {
        let mut registry = CVarRegistry::new();
        registry.init("flag", CVarValue::Bool(false)).unwrap();
        assert!(registry.set("flag", CVarValue::Bool(true)).is_ok());
        assert_eq!(registry.get_bool("flag"), true);
    }

    #[test]
    fn test_set_f32_convenience() {
        let mut registry = CVarRegistry::new();
        registry.init_f32("alpha", 0.5);
        registry.set_f32("alpha", 1.0);
        assert_eq!(registry.get_f32("alpha"), 1.0);
    }

    #[test]
    fn test_set_nonexistent() {
        let mut registry = CVarRegistry::new();
        let result = registry.set("missing", CVarValue::Int32(1));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_set_type_mismatch_float_to_int() {
        let mut registry = CVarRegistry::new();
        registry.init("var", CVarValue::F32(1.0)).unwrap();
        let result = registry.set("var", CVarValue::Int32(1));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Type mismatch"));
    }

    #[test]
    fn test_set_type_mismatch_int_to_string() {
        let mut registry = CVarRegistry::new();
        registry.init("var", CVarValue::Int32(1)).unwrap();
        let result = registry.set("var", CVarValue::String("1".to_string()));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Type mismatch"));
    }

    #[test]
    fn test_set_type_mismatch_string_to_float() {
        let mut registry = CVarRegistry::new();
        registry
            .init("var", CVarValue::String("text".to_string()))
            .unwrap();
        let result = registry.set("var", CVarValue::F32(1.0));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Type mismatch"));
    }

    #[test]
    fn test_set_type_mismatch_bool_to_int() {
        let mut registry = CVarRegistry::new();
        registry.init("var", CVarValue::Bool(true)).unwrap();
        let result = registry.set("var", CVarValue::Int32(1));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Type mismatch"));
    }

    #[test]
    fn test_set_type_mismatch_float_to_bool() {
        let mut registry = CVarRegistry::new();
        registry.init("var", CVarValue::F32(1.0)).unwrap();
        let result = registry.set("var", CVarValue::Bool(true));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Type mismatch"));
    }

    #[test]
    fn test_get() {
        let mut registry = CVarRegistry::new();
        registry.init("var", CVarValue::Int32(42)).unwrap();

        let value = registry.get("var");
        assert!(value.is_some());
        match value.unwrap() {
            CVarValue::Int32(v) => assert_eq!(*v, 42),
            _ => panic!("Expected Int variant"),
        }
    }

    #[test]
    fn test_get_missing() {
        let registry = CVarRegistry::new();
        assert!(registry.get("missing").is_none());
    }

    #[test]
    fn test_get_f32() {
        let mut registry = CVarRegistry::new();
        registry.init("var", CVarValue::F32(3.14)).unwrap();
        assert_eq!(registry.get_f32("var"), 3.14);
    }

    #[test]
    fn test_get_i32() {
        let mut registry = CVarRegistry::new();
        registry.init("var", CVarValue::Int32(99)).unwrap();
        assert_eq!(registry.get_i32("var"), 99);
    }

    #[test]
    fn test_get_string() {
        let mut registry = CVarRegistry::new();
        registry
            .init("var", CVarValue::String("hello".to_string()))
            .unwrap();
        assert_eq!(registry.get_string("var"), "hello");
    }

    #[test]
    fn test_get_bool() {
        let mut registry = CVarRegistry::new();
        registry.init("var", CVarValue::Bool(true)).unwrap();
        assert_eq!(registry.get_bool("var"), true);
    }

    #[test]
    fn test_exists() {
        let mut registry = CVarRegistry::new();
        assert!(!registry.exists("var"));
        registry.init("var", CVarValue::Int32(1)).unwrap();
        assert!(registry.exists("var"));
    }

    #[test]
    fn test_list_empty() {
        let registry = CVarRegistry::new();
        let list = registry.list();
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_list_single() {
        let mut registry = CVarRegistry::new();
        registry.init("var", CVarValue::Int32(1)).unwrap();
        let list = registry.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "var");
    }

    #[test]
    fn test_list_sorted() {
        let mut registry = CVarRegistry::new();
        registry.init("zebra", CVarValue::Int32(1)).unwrap();
        registry.init("apple", CVarValue::Int32(2)).unwrap();
        registry.init("monkey", CVarValue::Int32(3)).unwrap();

        let list = registry.list();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].0, "apple");
        assert_eq!(list[1].0, "monkey");
        assert_eq!(list[2].0, "zebra");
    }

    #[test]
    fn test_list_mixed_types() {
        let mut registry = CVarRegistry::new();
        registry.init("float_var", CVarValue::F32(1.5)).unwrap();
        registry.init("int_var", CVarValue::Int32(42)).unwrap();
        registry
            .init("string_var", CVarValue::String("test".to_string()))
            .unwrap();
        registry.init("bool_var", CVarValue::Bool(true)).unwrap();

        let list = registry.list();
        assert_eq!(list.len(), 4);

        // Verify all types are present (alphabetically sorted)
        assert_eq!(list[0].0, "bool_var");
        assert_eq!(list[1].0, "float_var");
        assert_eq!(list[2].0, "int_var");
        assert_eq!(list[3].0, "string_var");
    }

    #[test]
    fn test_multiple_operations() {
        let mut registry = CVarRegistry::new();

        // Initialize multiple variables
        registry.init_f32("player_speed", 5.0);
        registry
            .init("player_health", CVarValue::Int32(100))
            .unwrap();
        registry
            .init("player_name", CVarValue::String("Hero".to_string()))
            .unwrap();

        // Modify them
        registry.set_f32("player_speed", 7.5);
        registry.set("player_health", CVarValue::Int32(85)).unwrap();

        // Verify
        assert_eq!(registry.get_f32("player_speed"), 7.5);
        assert_eq!(registry.get_i32("player_health"), 85);
        assert_eq!(registry.get_string("player_name"), "Hero");

        let list = registry.list();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_dotted_names() {
        let mut registry = CVarRegistry::new();
        registry
            .init("player.health", CVarValue::Int32(100))
            .unwrap();
        registry.init("player.mana", CVarValue::Int32(50)).unwrap();
        registry.init("enemy.health", CVarValue::Int32(75)).unwrap();

        assert!(registry.exists("player.health"));
        assert!(registry.exists("player.mana"));
        assert!(registry.exists("enemy.health"));
        assert_eq!(registry.get_i32("player.health"), 100);
    }

    #[test]
    fn test_cvarvalue_clone() {
        let original = CVarValue::F32(3.14);
        let cloned = original.clone();
        assert_eq!(original.as_f32(), cloned.as_f32());
    }

    #[test]
    fn test_cvarvalue_debug() {
        let float_val = CVarValue::F32(3.14);
        let debug_str = format!("{:?}", float_val);
        assert!(debug_str.contains("F32"));
        assert!(debug_str.contains("3.14"));
    }
}
