use git2::{Oid, Repository, Sort};

use crate::commits::{ParsedCommit, parse_commit};
use crate::version::VersionTag;

/// Error type for git operations.
#[derive(Debug)]
pub enum GitError {
    NotARepo(String),
    Git2(git2::Error),
    NoCommits,
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::NotARepo(path) => write!(f, "not a git repository: {path}"),
            GitError::Git2(e) => write!(f, "git error: {e}"),
            GitError::NoCommits => write!(f, "no commits found in repository"),
        }
    }
}

impl std::error::Error for GitError {}

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
    Repository::discover(path).map_err(|_| GitError::NotARepo(path.to_string()))
}

/// Find the latest version tag in the repository.
pub fn find_latest_version_tag(repo: &Repository) -> Result<Option<VersionTag>, GitError> {
    let tag_names = repo.tag_names(None)?;

    let mut best: Option<VersionTag> = None;

    for tag_name in tag_names.iter().flatten() {
        if let Some(vt) = VersionTag::parse(tag_name) {
            match &best {
                Some(current) if vt.version > current.version => best = Some(vt),
                None => best = Some(vt),
                _ => {}
            }
        }
    }

    Ok(best)
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
    revwalk.push_head()?;

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
pub fn analyze(path: &str) -> Result<AnalysisResult, GitError> {
    let repo = open_repo(path)?;
    let latest_tag = find_latest_version_tag(&repo)?;
    let commits = commits_since_tag(&repo, latest_tag.as_ref())?;

    if commits.is_empty() && latest_tag.is_none() {
        // Check if there are any commits at all
        if repo.head().is_err() {
            return Err(GitError::NoCommits);
        }
    }

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
pub fn detect_prefix(tag: Option<&VersionTag>, user_prefix: Option<&str>) -> &'static str {
    if let Some(p) = user_prefix {
        if p == "v" {
            return "v";
        }
        return "";
    }
    match tag {
        Some(vt) if vt.has_v_prefix => "v",
        Some(_) => "",
        None => "v", // default to v prefix
    }
}
