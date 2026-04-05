use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// Represents a semantic version (major.minor.patch).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemVer {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl SemVer {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn zero() -> Self {
        Self::new(0, 0, 0)
    }

    /// Bump the version according to the given bump level.
    pub fn bump(&self, level: BumpLevel) -> Self {
        match level {
            BumpLevel::Major => Self::new(self.major + 1, 0, 0),
            BumpLevel::Minor => Self::new(self.major, self.minor + 1, 0),
            BumpLevel::Patch => Self::new(self.major, self.minor, self.patch + 1),
        }
    }
}

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
    }
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    InvalidFormat,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid semver format")
    }
}

impl std::error::Error for ParseError {}

impl FromStr for SemVer {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix('v').unwrap_or(s);
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(ParseError::InvalidFormat);
        }
        let major = parts[0].parse().map_err(|_| ParseError::InvalidFormat)?;
        let minor = parts[1].parse().map_err(|_| ParseError::InvalidFormat)?;
        let patch = parts[2].parse().map_err(|_| ParseError::InvalidFormat)?;
        Ok(Self::new(major, minor, patch))
    }
}

/// The level of version bump to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BumpLevel {
    Patch,
    Minor,
    Major,
}

impl fmt::Display for BumpLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BumpLevel::Patch => write!(f, "patch"),
            BumpLevel::Minor => write!(f, "minor"),
            BumpLevel::Major => write!(f, "major"),
        }
    }
}

/// Information about a version tag found in git.
#[derive(Debug, Clone)]
pub struct VersionTag {
    pub version: SemVer,
    pub tag_name: String,
    pub has_v_prefix: bool,
}

impl VersionTag {
    /// Parse a tag name into a VersionTag, returning None if it doesn't match.
    pub fn parse(tag_name: &str) -> Option<Self> {
        let has_v_prefix = tag_name.starts_with('v');
        let version: SemVer = tag_name.parse().ok()?;
        Some(Self {
            version,
            tag_name: tag_name.to_string(),
            has_v_prefix,
        })
    }

    /// Format the next version with the same prefix style as this tag.
    pub fn format_next(&self, next: &SemVer) -> String {
        if self.has_v_prefix {
            format!("v{next}")
        } else {
            next.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_semver() {
        assert_eq!("1.2.3".parse::<SemVer>().unwrap(), SemVer::new(1, 2, 3));
        assert_eq!("v1.2.3".parse::<SemVer>().unwrap(), SemVer::new(1, 2, 3));
        assert_eq!("0.0.0".parse::<SemVer>().unwrap(), SemVer::zero());
    }

    #[test]
    fn parse_semver_invalid() {
        assert!("abc".parse::<SemVer>().is_err());
        assert!("1.2".parse::<SemVer>().is_err());
        assert!("1.2.3.4".parse::<SemVer>().is_err());
    }

    #[test]
    fn semver_ordering() {
        let v1 = SemVer::new(1, 0, 0);
        let v2 = SemVer::new(1, 1, 0);
        let v3 = SemVer::new(2, 0, 0);
        assert!(v1 < v2);
        assert!(v2 < v3);
    }

    #[test]
    fn bump_patch() {
        let v = SemVer::new(1, 2, 3);
        assert_eq!(v.bump(BumpLevel::Patch), SemVer::new(1, 2, 4));
    }

    #[test]
    fn bump_minor() {
        let v = SemVer::new(1, 2, 3);
        assert_eq!(v.bump(BumpLevel::Minor), SemVer::new(1, 3, 0));
    }

    #[test]
    fn bump_major() {
        let v = SemVer::new(1, 2, 3);
        assert_eq!(v.bump(BumpLevel::Major), SemVer::new(2, 0, 0));
    }

    #[test]
    fn version_tag_parse() {
        let tag = VersionTag::parse("v1.2.3").unwrap();
        assert!(tag.has_v_prefix);
        assert_eq!(tag.version, SemVer::new(1, 2, 3));

        let tag = VersionTag::parse("1.2.3").unwrap();
        assert!(!tag.has_v_prefix);

        assert!(VersionTag::parse("not-a-version").is_none());
    }

    #[test]
    fn version_tag_format_next() {
        let tag = VersionTag::parse("v1.2.3").unwrap();
        let next = SemVer::new(1, 3, 0);
        assert_eq!(tag.format_next(&next), "v1.3.0");

        let tag = VersionTag::parse("1.2.3").unwrap();
        assert_eq!(tag.format_next(&next), "1.3.0");
    }

    #[test]
    fn semver_display() {
        assert_eq!(SemVer::new(1, 2, 3).to_string(), "1.2.3");
    }

    #[test]
    fn zero_bump_to_minor() {
        let v = SemVer::zero();
        assert_eq!(v.bump(BumpLevel::Minor), SemVer::new(0, 1, 0));
    }
}
