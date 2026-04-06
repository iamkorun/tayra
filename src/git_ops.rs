use git2::{Oid, Repository, Sort};

use crate::commits::{ParsedCommit, parse_commit};
use crate::version::{SemVer, VersionTag};

/// Error type for git operations.
#[derive(Debug)]
pub enum GitError {
    NotARepo { path: String, source: git2::Error },
    Git2(git2::Error),
    NoCommits,
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::NotARepo { path, source } => {
                write!(f, "not a git repository: {path} ({source})")
            }
            GitError::Git2(e) => write!(f, "git error: {e}"),
            GitError::NoCommits => write!(f, "no commits found in repository"),
        }
    }
}

impl std::error::Error for GitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GitError::NotARepo { source, .. } => Some(source),
            GitError::Git2(e) => Some(e),
            GitError::NoCommits => None,
        }
    }
}

impl From<git2::Error> for GitError {
    fn from(e: git2::Error) -> Self {
        GitError::Git2(e)
    }
}

/// Result of analyzing the repository.
#[derive(Debug)]
pub struct AnalysisResult {
    pub current_version: Option<VersionTag>,
    pub commits: Vec<ParsedCommit>,
}

/// Open the git repository at the given path (searching upward).
pub fn open_repo(path: &str) -> Result<Repository, GitError> {
    Repository::discover(path).map_err(|source| GitError::NotARepo {
        path: path.to_string(),
        source,
    })
}

/// Find the latest version tag in the repository.
///
/// If `user_prefix` is provided (e.g. `"release-"`), it is stripped before
/// parsing so custom-prefixed tags like `release-1.2.3` are recognised.
pub fn find_latest_version_tag(
    repo: &Repository,
    user_prefix: Option<&str>,
) -> Result<Option<VersionTag>, GitError> {
    let tag_names = repo.tag_names(None)?;

    let mut best: Option<VersionTag> = None;

    for tag_name in tag_names.iter().flatten() {
        let parsed = parse_tag_with_optional_prefix(tag_name, user_prefix);
        if let Some(vt) = parsed {
            match &best {
                Some(current) if vt.version > current.version => best = Some(vt),
                None => best = Some(vt),
                _ => {}
            }
        }
    }

    Ok(best)
}

/// Parse a tag name, optionally trying a user-supplied prefix first.
///
/// Semantics:
/// 1. If `user_prefix` is set, try stripping it and parsing as SemVer.
/// 2. If that fails, fall back to the default parser (handles `v1.2.3` and `1.2.3`).
///
/// This lets `--prefix release-` match `release-1.0.0` while still recognising
/// the user may be *changing* the prefix (e.g. bare tag → `v` output).
fn parse_tag_with_optional_prefix(tag_name: &str, user_prefix: Option<&str>) -> Option<VersionTag> {
    if let Some(p) = user_prefix
        && !p.is_empty()
        && let Some(stripped) = tag_name.strip_prefix(p)
        && let Ok(version) = stripped.parse::<SemVer>()
    {
        return Some(VersionTag {
            version,
            tag_name: tag_name.to_string(),
            has_v_prefix: tag_name.starts_with('v'),
        });
    }
    VersionTag::parse(tag_name)
}

/// Get the commit OID that a tag points to (resolving annotated tags).
fn resolve_tag_to_commit(repo: &Repository, tag_name: &str) -> Result<Oid, GitError> {
    let reference = repo.find_reference(&format!("refs/tags/{tag_name}"))?;
    let commit = reference.peel_to_commit()?;
    Ok(commit.id())
}

/// Collect all commits since a given tag (or all commits if no tag).
pub fn commits_since_tag(
    repo: &Repository,
    tag: Option<&VersionTag>,
) -> Result<Vec<ParsedCommit>, GitError> {
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)?;

    // Empty repos have no HEAD — treat as "no commits" rather than an opaque git error.
    if let Err(e) = revwalk.push_head() {
        if repo.head().is_err() {
            return Err(GitError::NoCommits);
        }
        return Err(GitError::Git2(e));
    }

    let stop_at = match tag {
        Some(vt) => Some(resolve_tag_to_commit(repo, &vt.tag_name)?),
        None => None,
    };

    let mut commits = Vec::new();

    for oid_result in revwalk {
        let oid = oid_result?;
        if Some(oid) == stop_at {
            break;
        }
        let commit = repo.find_commit(oid)?;
        let message = commit.message().unwrap_or("").to_string();
        if !message.is_empty() {
            commits.push(parse_commit(&message));
        }
    }

    Ok(commits)
}

/// Analyze the repository: find latest tag and collect commits since it.
///
/// `user_prefix` is forwarded to tag detection so custom-prefixed tags are
/// recognised (e.g. `--prefix release-` makes `release-1.0.0` parseable).
pub fn analyze(path: &str, user_prefix: Option<&str>) -> Result<AnalysisResult, GitError> {
    let repo = open_repo(path)?;
    let latest_tag = find_latest_version_tag(&repo, user_prefix)?;
    let commits = commits_since_tag(&repo, latest_tag.as_ref())?;

    Ok(AnalysisResult {
        current_version: latest_tag,
        commits,
    })
}

/// Create a git tag at HEAD.
pub fn create_tag(path: &str, tag_name: &str) -> Result<(), GitError> {
    let repo = open_repo(path)?;
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;
    repo.tag_lightweight(tag_name, commit.as_object(), false)?;
    Ok(())
}

/// Determine the tag prefix to use based on existing tags or user preference.
///
/// Precedence:
/// 1. User-supplied `--prefix` (any string, including empty).
/// 2. Existing tag's prefix (`v` if present, else empty).
/// 3. Default `v` prefix for fresh repos.
pub fn detect_prefix(tag: Option<&VersionTag>, user_prefix: Option<&str>) -> String {
    if let Some(p) = user_prefix {
        return p.to_string();
    }
    match tag {
        Some(vt) if vt.has_v_prefix => "v".to_string(),
        Some(_) => String::new(),
        None => "v".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_prefix_user_overrides_v() {
        let tag = VersionTag::parse("v1.0.0");
        assert_eq!(detect_prefix(tag.as_ref(), Some("")), "");
    }

    #[test]
    fn detect_prefix_user_any_string() {
        assert_eq!(detect_prefix(None, Some("release-")), "release-");
    }

    #[test]
    fn detect_prefix_falls_back_to_tag_style() {
        let v_tag = VersionTag::parse("v1.0.0");
        let plain_tag = VersionTag::parse("1.0.0");
        assert_eq!(detect_prefix(v_tag.as_ref(), None), "v");
        assert_eq!(detect_prefix(plain_tag.as_ref(), None), "");
    }

    #[test]
    fn detect_prefix_defaults_to_v_for_fresh_repo() {
        assert_eq!(detect_prefix(None, None), "v");
    }
}
