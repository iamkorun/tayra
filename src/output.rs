use crate::commits::{compute_bump, count_by_type};
use crate::git_ops::AnalysisResult;
use crate::version::{BumpLevel, SemVer};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const SEPARATOR: &str = "━━━━━━━━━━━━━━━━━━━━━";

/// Compute (bump_level, suggested_version) for an analysis result.
///
/// Rules:
/// - No commits → patch bump from current (or 0.0.0 → 0.0.1).
/// - Current == 0.0.0 and any non-breaking commits → minor bump (0.0.0 → 0.1.0).
/// - Otherwise → highest bump level across commits.
fn compute_bump_and_version(result: &AnalysisResult) -> (BumpLevel, SemVer) {
    let current = result
        .current_version
        .as_ref()
        .map(|vt| vt.version.clone())
        .unwrap_or_else(SemVer::zero);

    if result.commits.is_empty() {
        return (BumpLevel::Patch, current.bump(BumpLevel::Patch));
    }

    let raw_bump = compute_bump(&result.commits);

    // Special case: 0.0.0 → 0.1.0 when the raw bump is patch.
    // (Breaking or feat commits would already push to minor/major.)
    let effective_bump = if current == SemVer::zero() && raw_bump == BumpLevel::Patch {
        BumpLevel::Minor
    } else {
        raw_bump
    };

    (effective_bump, current.bump(effective_bump))
}

/// Format the full human-readable output.
pub fn format_full(result: &AnalysisResult, prefix: &str, verbose: bool) -> String {
    let (bump, suggested) = compute_bump_and_version(result);
    let formatted_suggested = format!("{prefix}{suggested}");
    let current_display = match &result.current_version {
        Some(vt) => vt.tag_name.clone(),
        None => "none (assuming 0.0.0)".to_string(),
    };

    let mut output = String::new();
    output.push_str(&format!("tayra v{VERSION}\n"));
    output.push_str(SEPARATOR);
    output.push('\n');
    output.push_str(&format!("Current version: {current_display}\n"));
    output.push_str(&format!(
        "Suggested bump:  {bump} → {formatted_suggested}\n"
    ));

    if !result.commits.is_empty() {
        output.push_str(&format!("\nCommits since {current_display}:\n"));
        for commit in &result.commits {
            let summary = commit.summary().trim_end();
            if verbose && commit.is_breaking {
                output.push_str(&format!("  {summary}  [BREAKING]\n"));
            } else {
                output.push_str(&format!("  {summary}\n"));
            }
        }

        let counts = count_by_type(&result.commits);
        let breakdown: Vec<String> = counts.iter().map(|(t, c)| format!("{c} {t}")).collect();
        output.push_str(&format!("\nBreakdown: {} → {bump}\n", breakdown.join(", ")));
    } else {
        output.push_str("\nNo new commits since last tag.\n");
    }

    output
}

/// Format the CI-friendly output (just the version string).
pub fn format_ci(result: &AnalysisResult, prefix: &str) -> String {
    let (_, suggested) = compute_bump_and_version(result);
    format!("{prefix}{suggested}")
}

/// Compute the suggested next version.
pub fn compute_suggested(result: &AnalysisResult) -> SemVer {
    compute_bump_and_version(result).1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commits::{ParsedCommit, parse_commit};
    use crate::version::VersionTag;

    fn make_result(tag: Option<&str>, messages: &[&str]) -> AnalysisResult {
        let current_version = tag.and_then(VersionTag::parse);
        let commits: Vec<ParsedCommit> = messages.iter().map(|m| parse_commit(m)).collect();
        AnalysisResult {
            current_version,
            commits,
        }
    }

    #[test]
    fn ci_output_minor_bump() {
        let result = make_result(Some("v1.2.3"), &["feat: add login", "fix: typo"]);
        assert_eq!(format_ci(&result, "v"), "v1.3.0");
    }

    #[test]
    fn ci_output_patch_bump() {
        let result = make_result(Some("v1.2.3"), &["fix: bug"]);
        assert_eq!(format_ci(&result, "v"), "v1.2.4");
    }

    #[test]
    fn ci_output_major_bump() {
        let result = make_result(Some("v1.2.3"), &["feat!: breaking change"]);
        assert_eq!(format_ci(&result, "v"), "v2.0.0");
    }

    #[test]
    fn ci_output_no_prefix() {
        let result = make_result(Some("1.2.3"), &["fix: bug"]);
        assert_eq!(format_ci(&result, ""), "1.2.4");
    }

    #[test]
    fn ci_output_no_tags() {
        let result = make_result(None, &["feat: initial"]);
        assert_eq!(format_ci(&result, "v"), "v0.1.0");
    }

    #[test]
    fn ci_output_no_tags_no_commits() {
        let result = make_result(None, &[]);
        assert_eq!(format_ci(&result, "v"), "v0.0.1");
    }

    #[test]
    fn full_output_contains_header() {
        let result = make_result(Some("v1.0.0"), &["feat: new thing"]);
        let output = format_full(&result, "v", false);
        assert!(output.contains("tayra v"));
        assert!(output.contains("Current version: v1.0.0"));
        assert!(output.contains("minor"));
        assert!(output.contains("v1.1.0"));
    }

    #[test]
    fn full_output_no_commits() {
        let result = make_result(Some("v1.0.0"), &[]);
        let output = format_full(&result, "v", false);
        assert!(output.contains("No new commits"));
    }

    #[test]
    fn full_output_verbose_marks_breaking() {
        let result = make_result(Some("v1.0.0"), &["feat!: drop old api"]);
        let output = format_full(&result, "v", true);
        assert!(output.contains("[BREAKING]"));
    }

    #[test]
    fn full_output_non_verbose_no_breaking_marker() {
        let result = make_result(Some("v1.0.0"), &["feat!: drop old api"]);
        let output = format_full(&result, "v", false);
        assert!(!output.contains("[BREAKING]"));
    }

    #[test]
    fn compute_bump_and_version_no_tag_no_commits() {
        let result = make_result(None, &[]);
        let (bump, ver) = compute_bump_and_version(&result);
        assert_eq!(bump, BumpLevel::Patch);
        assert_eq!(ver, SemVer::new(0, 0, 1));
    }

    #[test]
    fn compute_bump_and_version_zero_with_breaking_is_major() {
        let result = make_result(None, &["feat!: breaking"]);
        let (bump, ver) = compute_bump_and_version(&result);
        assert_eq!(bump, BumpLevel::Major);
        assert_eq!(ver, SemVer::new(1, 0, 0));
    }

    #[test]
    fn compute_suggested_no_tags_with_feat() {
        let result = make_result(None, &["feat: init"]);
        assert_eq!(compute_suggested(&result), SemVer::new(0, 1, 0));
    }

    #[test]
    fn compute_suggested_existing_tag_with_feat() {
        let result = make_result(Some("v2.1.0"), &["feat: new"]);
        assert_eq!(compute_suggested(&result), SemVer::new(2, 2, 0));
    }
}
