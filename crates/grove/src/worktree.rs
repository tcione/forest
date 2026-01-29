//! Worktree management operations

use regex::Regex;

/// Sanitize a branch name for use in directory names
/// Only allows [A-Za-z0-9\-_], replaces other chars with "--"
pub fn sanitize_branch_name(branch: &str) -> String {
    let re = Regex::new(r"[^A-Za-z0-9\-_]").unwrap();
    re.replace_all(branch, "--").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_branch_name() {
        assert_eq!(sanitize_branch_name("feature-test"), "feature-test");
        assert_eq!(sanitize_branch_name("feature/test"), "feature--test");
        assert_eq!(sanitize_branch_name("feat/sub/test"), "feat--sub--test");
        assert_eq!(sanitize_branch_name("feature_test"), "feature_test");
        assert_eq!(sanitize_branch_name("CamelCase"), "CamelCase");
        assert_eq!(sanitize_branch_name("with spaces"), "with--spaces");
        assert_eq!(sanitize_branch_name("special@chars!"), "special--chars--");
    }
}
