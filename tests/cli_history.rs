//! Tests for command history CLI configuration

#[cfg(feature = "repl")]
mod cli_history_tests {
    use brubeck::cli::Cli;
    use clap::Parser;

    #[test]
    fn test_history_defaults() {
        let cli = Cli::parse_from(["brubeck"]);
        assert_eq!(cli.history_size, 1000);
        assert!(!cli.no_history);
    }

    #[test]
    fn test_history_size_flag() {
        let cli = Cli::parse_from(["brubeck", "--history-size", "500"]);
        assert_eq!(cli.history_size, 500);
        assert!(!cli.no_history);
    }

    #[test]
    fn test_no_history_flag() {
        let cli = Cli::parse_from(["brubeck", "--no-history"]);
        assert!(cli.no_history);
    }

    #[test]
    fn test_history_conflicts() {
        // Should fail when both flags are specified
        let result = Cli::try_parse_from(["brubeck", "--no-history", "--history-size", "500"]);
        assert!(result.is_err());
    }
}
