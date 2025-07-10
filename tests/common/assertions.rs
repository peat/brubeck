//! Common assertion helpers for tests

/// Helper for asserting string contains substring with context
pub fn assert_contains_with_context(haystack: &str, needle: &str, context: &str) {
    if !haystack.contains(needle) {
        panic!("{context}: Expected to find '{needle}' in output\n  Actual output: {haystack}");
    }
}
