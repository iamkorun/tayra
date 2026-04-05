use crate::commits::{compute_bump, count_by_type};
use crate::git_ops::AnalysisResult;
use crate::version::{BumpLevel, SemVer};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Format the full human-readable output.
pub fn format_full(result: &AnalysisResult, prefix: &str) -> String {
    let current = result
        .current_version
        .as_ref()
        .map(|vt| vt.version.clone())
        .unwrap_or_else(SemVer::zero);

    let bump = if result.commits.is_empty() {
        BumpLevel::Patch
    } else {
        compute_bump(&result.commits)
    };

    // Special case: if current is 0.0.0 and there are feat commits, suggest 0.1.0
    let suggested = if current == SemVer::zero() && !result.commits.is_empty() {
        let raw_bump = compute_bump(&result.commits);
        if raw_bump == BumpLevel::Patch {
            current.bump(BumpLevel::Minor)
        } else {
            current.bump(raw_bump)
        }
    } else {
        current.bump(bump)
    };

    let formatted_suggested = format!("{prefix}{suggested}");
    let current_display = match &result.current_version {
        Some(vt) => vt.tag_name.clone(),
        None => "none (assuming 0.0.0)".to_string(),
    };

    let mut output = String::new();
    output.push_str(&format!("tayra v{VERSION}\n"));
    output.push_str("\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\n");
    output.push_str(&format!("Current version: {current_display}\n"));
    output.push_str(&format!(
        "Suggested bump:  {bump} \u{2192} {formatted_suggested}\n"
    ));

    if !result.commits.is_empty() {
        output.push_str(&format!(
            "\nCommits since {}:\n",
            current_display
        ));
        for commit in &result.commits {
            let summary = commit.summary().trim_end();
            output.push_str(&format!("  {summary}\n"));
        }

        let counts = count_by_type(&result.commits);
        let breakdown: Vec<String> = counts.iter().map(|(t, c)| format!("{c} {t}")).collect();
        output.push_str(&format!(
            "\nBreakdown: {} \u{2192} {bump}\n",
            breakdown.join(", ")
        ));
    } else {
        output.push_str("\nNo new commits since last tag.\n");
    }

    output
}

/// Format the CI-friendly output (just the version string).
pub fn format_ci(result: &AnalysisResult, prefix: &str) -> String {
    let current = result
        .current_version
        .as_ref()
        .map(|vt| vt.version.clone())
        .unwrap_or_else(SemVer::zero);

    let suggested = if result.commits.is_empty() {
        current.bump(BumpLevel::Patch)
    } else if current == SemVer::zero() {
        let raw_bump = compute_bump(&result.commits);
        if raw_bump == BumpLevel::Patch {
            current.bump(BumpLevel::Minor)
        } else {
            current.bump(raw_bump)
        }
    } else {
        current.bump(compute_bump(&result.commits))
    };

    format!("{prefix}{suggested}")
}

/// Compute the suggested next version.
pub fn compute_suggested(result: &AnalysisResult) -> SemVer {
    let current = result
        .current_version
        .as_ref()
        .map(|vt| vt.version.clone())
        .unwrap_or_else(SemVer::zero);

    if result.commits.is_empty() {
        return current.bump(BumpLevel::Patch);
    }

    if current == SemVer::zero() {
        let raw_bump = compute_bump(&result.commits);
        if raw_bump == BumpLevel::Patch {
            current.bump(BumpLevel::Minor)
        } else {
            current.bump(raw_bump)
        }
    } else {
        current.bump(compute_bump(&result.commits))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commits::{parse_commit, ParsedCommit};
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
        let output = format_full(&result, "v");
        assert!(output.contains("tayra v"));
        assert!(output.contains("Current version: v1.0.0"));
        assert!(output.contains("minor"));
        assert!(output.contains("v1.1.0"));
    }

    #[test]
    fn full_output_no_commits() {
        let result = make_result(Some("v1.0.0"), &[]);
        let output = format_full(&result, "v");
        assert!(output.contains("No new commits"));
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
