use crate::version::BumpLevel;
use std::fmt;

/// The type of a conventional commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommitType {
    Feat,
    Fix,
    Chore,
    Docs,
    Refactor,
    Test,
    Ci,
    Perf,
    Style,
    Build,
    Other,
}

impl fmt::Display for CommitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommitType::Feat => write!(f, "feat"),
            CommitType::Fix => write!(f, "fix"),
            CommitType::Chore => write!(f, "chore"),
            CommitType::Docs => write!(f, "docs"),
            CommitType::Refactor => write!(f, "refactor"),
            CommitType::Test => write!(f, "test"),
            CommitType::Ci => write!(f, "ci"),
            CommitType::Perf => write!(f, "perf"),
            CommitType::Style => write!(f, "style"),
            CommitType::Build => write!(f, "build"),
            CommitType::Other => write!(f, "other"),
        }
    }
}

impl CommitType {
    /// The bump level implied by this commit type alone (ignoring breaking changes).
    pub fn bump_level(self) -> BumpLevel {
        match self {
            CommitType::Feat => BumpLevel::Minor,
            _ => BumpLevel::Patch,
        }
    }
}

/// A parsed conventional commit.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParsedCommit {
    pub commit_type: CommitType,
    pub scope: Option<String>,
    pub description: String,
    pub is_breaking: bool,
    pub raw_message: String,
}

impl ParsedCommit {
    /// Determine the bump level for this commit.
    pub fn bump_level(&self) -> BumpLevel {
        if self.is_breaking {
            BumpLevel::Major
        } else {
            self.commit_type.bump_level()
        }
    }

    /// Short summary for display (first line of commit).
    pub fn summary(&self) -> &str {
        self.raw_message
            .lines()
            .next()
            .unwrap_or(&self.raw_message)
    }
}

/// Parse a commit message into a ParsedCommit.
pub fn parse_commit(message: &str) -> ParsedCommit {
    let first_line = message.lines().next().unwrap_or(message).trim();
    let body = message.lines().skip(1).collect::<Vec<_>>().join("\n");

    let is_breaking_in_body = body.contains("BREAKING CHANGE:")
        || body.contains("BREAKING-CHANGE:");

    // Try to parse conventional commit format: type(scope)!: description
    if let Some(parsed) = try_parse_conventional(first_line, &body, is_breaking_in_body) {
        return parsed;
    }

    // Fallback: not a conventional commit
    ParsedCommit {
        commit_type: CommitType::Other,
        scope: None,
        description: first_line.to_string(),
        is_breaking: is_breaking_in_body,
        raw_message: message.to_string(),
    }
}

fn try_parse_conventional(
    first_line: &str,
    body: &str,
    is_breaking_in_body: bool,
) -> Option<ParsedCommit> {
    let colon_pos = first_line.find(':')?;
    let prefix = &first_line[..colon_pos];
    let description = first_line[colon_pos + 1..].trim().to_string();

    if description.is_empty() {
        return None;
    }

    // Parse prefix: type, optional (scope), optional !
    let (type_str, scope, bang) = parse_prefix(prefix)?;

    let commit_type = match type_str {
        "feat" => CommitType::Feat,
        "fix" => CommitType::Fix,
        "chore" => CommitType::Chore,
        "docs" => CommitType::Docs,
        "refactor" => CommitType::Refactor,
        "test" | "tests" => CommitType::Test,
        "ci" => CommitType::Ci,
        "perf" => CommitType::Perf,
        "style" => CommitType::Style,
        "build" => CommitType::Build,
        _ => return None,
    };

    Some(ParsedCommit {
        commit_type,
        scope,
        description,
        is_breaking: bang || is_breaking_in_body,
        raw_message: format!("{first_line}\n{body}"),
    })
}

/// Parse the prefix part before the colon.
/// Returns (type, optional scope, has_bang).
fn parse_prefix(prefix: &str) -> Option<(& str, Option<String>, bool)> {
    let prefix = prefix.trim();

    if prefix.is_empty() {
        return None;
    }

    // Check for bang at the end
    let (prefix, bang) = if let Some(stripped) = prefix.strip_suffix('!') {
        (stripped, true)
    } else {
        (prefix, false)
    };

    // Check for scope in parens
    if let Some(paren_start) = prefix.find('(') {
        let type_str = &prefix[..paren_start];
        let rest = &prefix[paren_start + 1..];
        let paren_end = rest.find(')')?;
        let scope = &rest[..paren_end];

        // Ensure nothing unexpected after closing paren
        if paren_end + 1 != rest.len() {
            return None;
        }

        if !type_str.chars().all(|c| c.is_ascii_alphanumeric()) {
            return None;
        }

        Some((type_str, Some(scope.to_string()), bang))
    } else {
        if !prefix.chars().all(|c| c.is_ascii_alphanumeric()) {
            return None;
        }
        Some((prefix, None, bang))
    }
}

/// Compute the overall bump level from a list of parsed commits.
pub fn compute_bump(commits: &[ParsedCommit]) -> BumpLevel {
    commits
        .iter()
        .map(|c| c.bump_level())
        .max()
        .unwrap_or(BumpLevel::Patch)
}

/// Count commits by type, returning a sorted vec of (type_name, count).
pub fn count_by_type(commits: &[ParsedCommit]) -> Vec<(String, usize)> {
    use std::collections::HashMap;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for commit in commits {
        *counts.entry(commit.commit_type.to_string()).or_insert(0) += 1;
    }
    let mut result: Vec<_> = counts.into_iter().collect();
    result.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_feat() {
        let c = parse_commit("feat: add login");
        assert_eq!(c.commit_type, CommitType::Feat);
        assert_eq!(c.description, "add login");
        assert!(!c.is_breaking);
        assert!(c.scope.is_none());
    }

    #[test]
    fn parse_scoped_fix() {
        let c = parse_commit("fix(auth): correct token expiry");
        assert_eq!(c.commit_type, CommitType::Fix);
        assert_eq!(c.scope.as_deref(), Some("auth"));
        assert_eq!(c.description, "correct token expiry");
    }

    #[test]
    fn parse_breaking_bang() {
        let c = parse_commit("feat!: remove deprecated API");
        assert_eq!(c.commit_type, CommitType::Feat);
        assert!(c.is_breaking);
    }

    #[test]
    fn parse_breaking_in_body() {
        let c = parse_commit("feat: new API\n\nBREAKING CHANGE: old API removed");
        assert_eq!(c.commit_type, CommitType::Feat);
        assert!(c.is_breaking);
    }

    #[test]
    fn parse_chore() {
        let c = parse_commit("chore: update deps");
        assert_eq!(c.commit_type, CommitType::Chore);
        assert_eq!(c.bump_level(), BumpLevel::Patch);
    }

    #[test]
    fn parse_non_conventional() {
        let c = parse_commit("Updated the readme file");
        assert_eq!(c.commit_type, CommitType::Other);
        assert_eq!(c.bump_level(), BumpLevel::Patch);
    }

    #[test]
    fn parse_docs() {
        let c = parse_commit("docs: update changelog");
        assert_eq!(c.commit_type, CommitType::Docs);
    }

    #[test]
    fn parse_refactor() {
        let c = parse_commit("refactor(core): simplify parser");
        assert_eq!(c.commit_type, CommitType::Refactor);
        assert_eq!(c.scope.as_deref(), Some("core"));
    }

    #[test]
    fn compute_bump_feat_wins() {
        let commits = vec![
            parse_commit("fix: bug"),
            parse_commit("feat: feature"),
            parse_commit("chore: deps"),
        ];
        assert_eq!(compute_bump(&commits), BumpLevel::Minor);
    }

    #[test]
    fn compute_bump_breaking_wins() {
        let commits = vec![
            parse_commit("feat: feature"),
            parse_commit("fix!: breaking fix"),
        ];
        assert_eq!(compute_bump(&commits), BumpLevel::Major);
    }

    #[test]
    fn compute_bump_only_fixes() {
        let commits = vec![
            parse_commit("fix: bug1"),
            parse_commit("fix: bug2"),
        ];
        assert_eq!(compute_bump(&commits), BumpLevel::Patch);
    }

    #[test]
    fn count_by_type_groups_correctly() {
        let commits = vec![
            parse_commit("feat: a"),
            parse_commit("feat: b"),
            parse_commit("fix: c"),
        ];
        let counts = count_by_type(&commits);
        assert_eq!(counts[0], ("feat".to_string(), 2));
        assert_eq!(counts[1], ("fix".to_string(), 1));
    }

    #[test]
    fn parse_empty_description_is_other() {
        let c = parse_commit("feat:");
        assert_eq!(c.commit_type, CommitType::Other);
    }

    #[test]
    fn parse_breaking_change_dash_in_body() {
        let c = parse_commit("fix: something\n\nBREAKING-CHANGE: old removed");
        assert!(c.is_breaking);
    }

    #[test]
    fn bump_level_ordering() {
        assert!(BumpLevel::Patch < BumpLevel::Minor);
        assert!(BumpLevel::Minor < BumpLevel::Major);
    }

    #[test]
    fn parse_test_type() {
        let c = parse_commit("test: add unit tests");
        assert_eq!(c.commit_type, CommitType::Test);
    }

    #[test]
    fn parse_ci_type() {
        let c = parse_commit("ci: add github actions");
        assert_eq!(c.commit_type, CommitType::Ci);
    }

    #[test]
    fn parse_perf_type() {
        let c = parse_commit("perf: optimize hot path");
        assert_eq!(c.commit_type, CommitType::Perf);
    }

    #[test]
    fn parse_scoped_breaking() {
        let c = parse_commit("feat(api)!: redesign endpoints");
        assert_eq!(c.commit_type, CommitType::Feat);
        assert_eq!(c.scope.as_deref(), Some("api"));
        assert!(c.is_breaking);
    }
}
