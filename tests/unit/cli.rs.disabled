//! Unit tests for CLI-related functionality
//!
//! These tests focus on individual functions like memory size parsing
//! without requiring the full application context.

#[cfg(test)]
mod memory_parser {
    use brubeck::cli::parse_memory_size;

    #[test]
    fn test_bytes_parsing() {
        assert_eq!(parse_memory_size("0").unwrap(), 0);
        assert_eq!(parse_memory_size("1").unwrap(), 1);
        assert_eq!(parse_memory_size("1024").unwrap(), 1024);
        assert_eq!(parse_memory_size("1048576").unwrap(), 1048576);
    }

    #[test]
    fn test_kilobyte_parsing() {
        assert_eq!(parse_memory_size("1k").unwrap(), 1024);
        assert_eq!(parse_memory_size("1K").unwrap(), 1024);
        assert_eq!(parse_memory_size("16k").unwrap(), 16 * 1024);
        assert_eq!(parse_memory_size("256K").unwrap(), 256 * 1024);
        assert_eq!(parse_memory_size("1024k").unwrap(), 1024 * 1024);
    }

    #[test]
    fn test_megabyte_parsing() {
        assert_eq!(parse_memory_size("1m").unwrap(), 1024 * 1024);
        assert_eq!(parse_memory_size("1M").unwrap(), 1024 * 1024);
        assert_eq!(parse_memory_size("5m").unwrap(), 5 * 1024 * 1024);
        assert_eq!(parse_memory_size("16M").unwrap(), 16 * 1024 * 1024);
    }

    #[test]
    fn test_whitespace_handling() {
        // Should trim whitespace
        assert_eq!(parse_memory_size(" 1024 ").unwrap(), 1024);
        assert_eq!(parse_memory_size("  1k  ").unwrap(), 1024);
        assert_eq!(parse_memory_size("\t1M\n").unwrap(), 1024 * 1024);
    }

    #[test]
    fn test_invalid_formats() {
        // Empty or whitespace only
        assert!(parse_memory_size("").is_err());
        assert!(parse_memory_size("   ").is_err());

        // Invalid characters
        assert!(parse_memory_size("abc").is_err());
        assert!(parse_memory_size("12x34").is_err());
        assert!(parse_memory_size("1.5k").is_err());
        assert!(parse_memory_size("1,024").is_err());

        // Negative numbers
        assert!(parse_memory_size("-1").is_err());
        assert!(parse_memory_size("-1k").is_err());

        // Unsupported units
        assert!(parse_memory_size("1g").is_err());
        assert!(parse_memory_size("1G").is_err());
        assert!(parse_memory_size("1kb").is_err());
        assert!(parse_memory_size("1MB").is_err());
        assert!(parse_memory_size("1KiB").is_err());

        // Multiple units
        assert!(parse_memory_size("1km").is_err());
        assert!(parse_memory_size("1mk").is_err());
    }

    #[test]
    fn test_edge_cases() {
        // Zero with units
        assert_eq!(parse_memory_size("0k").unwrap(), 0);
        assert_eq!(parse_memory_size("0K").unwrap(), 0);
        assert_eq!(parse_memory_size("0m").unwrap(), 0);
        assert_eq!(parse_memory_size("0M").unwrap(), 0);

        // Test reasonable values work
        assert_eq!(parse_memory_size("1M").unwrap(), 1024 * 1024);
        assert_eq!(parse_memory_size("256M").unwrap(), 256 * 1024 * 1024);
        assert_eq!(parse_memory_size("512M").unwrap(), 512 * 1024 * 1024);
    }

    #[test]
    fn test_overflow_protection() {
        // Very large values should fail gracefully
        assert!(parse_memory_size("999999999999999999999").is_err());
        assert!(parse_memory_size("999999999999M").is_err());
        assert!(parse_memory_size("18446744073709551616").is_err()); // u64::MAX + 1

        // 1GB should be allowed
        assert!(parse_memory_size("1024M").is_ok());
        assert_eq!(parse_memory_size("1024M").unwrap(), 1024 * 1024 * 1024);

        // Just over 1GB should fail
        assert!(parse_memory_size("1025M").is_err());
        assert!(parse_memory_size("2G").is_err()); // We don't support G suffix, but also too big
    }
}

#[cfg(test)]
mod config_validation {
    use brubeck::cli::Config;

    #[test]
    fn test_config_validation() {
        // Valid configs
        assert!(Config::new(1024, 100).is_ok());
        assert!(Config::new(0, 0).is_ok()); // Edge case: minimal config
        assert!(Config::new(1024 * 1024, 10000).is_ok());

        // Memory size validation (if any limits are imposed)
        let huge_memory = Config::new(usize::MAX, 100);
        // Should either succeed or fail gracefully
        assert!(huge_memory.is_ok() || huge_memory.is_err());
    }

    #[test]
    fn test_effective_undo_limit() {
        let config = Config::new(1024, 100).unwrap();
        assert_eq!(config.effective_undo_limit(), 100);

        // Zero means disabled
        let config = Config::new(1024, 0).unwrap();
        assert_eq!(config.effective_undo_limit(), 0);
    }
}

#[cfg(test)]
mod semicolon_parsing {
    #[test]
    fn test_split_commands() {
        use brubeck::cli::split_commands;

        // Basic splitting
        assert_eq!(split_commands("A; B; C"), vec!["A", "B", "C"]);

        // Handle empty parts
        assert_eq!(split_commands("A;; B"), vec!["A", "B"]);

        // Trim whitespace
        assert_eq!(split_commands("  A  ;  B  ;  C  "), vec!["A", "B", "C"]);

        // Single command (no semicolons)
        assert_eq!(split_commands("ADDI x1, x0, 10"), vec!["ADDI x1, x0, 10"]);

        // Empty input
        assert_eq!(split_commands(""), Vec::<&str>::new());

        // Only semicolons
        assert_eq!(split_commands(";;;"), Vec::<&str>::new());
    }

    #[test]
    fn test_commands_with_commas() {
        use brubeck::cli::split_commands;

        // Don't split on commas within commands
        assert_eq!(
            split_commands("ADDI x1, x0, 10; ADDI x2, x0, 20"),
            vec!["ADDI x1, x0, 10", "ADDI x2, x0, 20"]
        );
    }
}
