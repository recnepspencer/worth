use serde::{Deserialize, Serialize};

/// Opaque, host-defined semantic locality. A path is never a work partition.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "Vec<String>", into = "Vec<String>")]
pub struct ScopePath(Vec<String>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopePathError {
    Empty,
    TooDeep,
    EmptySegment,
    InvalidPadding,
}

impl ScopePath {
    pub const MAX_DEPTH: usize = 8;

    pub fn new(segments: impl IntoIterator<Item = String>) -> Result<Self, ScopePathError> {
        let segments: Vec<_> = segments.into_iter().collect();
        if segments.is_empty() {
            return Err(ScopePathError::Empty);
        }
        if segments.len() > Self::MAX_DEPTH {
            return Err(ScopePathError::TooDeep);
        }
        if segments.iter().any(String::is_empty) {
            return Err(ScopePathError::EmptySegment);
        }
        Ok(Self(segments))
    }

    pub fn one(segment: impl Into<String>) -> Result<Self, ScopePathError> {
        Self::new([segment.into()])
    }

    pub fn segments(&self) -> &[String] {
        &self.0
    }

    pub(crate) fn storage(&self) -> &Vec<String> {
        &self.0
    }

    pub(crate) fn retained_capacity_bytes(&self) -> usize {
        self.0
            .capacity()
            .saturating_mul(std::mem::size_of::<String>())
            .saturating_add(self.0.iter().fold(0usize, |sum, segment| {
                sum.saturating_add(segment.capacity())
            }))
    }

    pub fn depth(&self) -> usize {
        self.0.len()
    }

    pub fn with_segment(mut self, segment: impl Into<String>) -> Result<Self, ScopePathError> {
        let segment = segment.into();
        if segment.is_empty() {
            return Err(ScopePathError::EmptySegment);
        }
        if self.0.len() == Self::MAX_DEPTH {
            return Err(ScopePathError::TooDeep);
        }
        self.0.push(segment);
        Ok(self)
    }

    pub fn covers(&self, descendant: &Self) -> bool {
        descendant.0.starts_with(&self.0)
    }

    pub fn prefix(&self, depth: usize) -> Option<Self> {
        (1..=self.depth())
            .contains(&depth)
            .then(|| Self(self.0[..depth].to_vec()))
    }

    pub fn total_segment_bytes(&self) -> usize {
        self.0
            .iter()
            .fold(0usize, |sum, segment| sum.saturating_add(segment.len()))
    }

    pub fn checked_segment_bytes(&self) -> Option<usize> {
        self.0
            .iter()
            .try_fold(0usize, |sum, segment| sum.checked_add(segment.len()))
    }
}

impl TryFrom<Vec<String>> for ScopePath {
    type Error = ScopePathError;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<ScopePath> for Vec<String> {
    fn from(value: ScopePath) -> Self {
        value.0
    }
}

impl std::fmt::Display for ScopePathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Empty => "scope path is empty",
            Self::TooDeep => "scope path exceeds eight segments",
            Self::EmptySegment => "scope path contains an empty segment",
            Self::InvalidPadding => "interned scope path contains invalid padding",
        };
        f.write_str(message)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ScopeCoverage {
    Exact,
    Subtree,
}

pub fn scopes_overlap(
    left_path: &ScopePath,
    left_coverage: ScopeCoverage,
    right_path: &ScopePath,
    right_coverage: ScopeCoverage,
) -> bool {
    match (left_coverage, right_coverage) {
        (ScopeCoverage::Exact, ScopeCoverage::Exact) => left_path == right_path,
        (ScopeCoverage::Subtree, ScopeCoverage::Exact) => left_path.covers(right_path),
        (ScopeCoverage::Exact, ScopeCoverage::Subtree) => right_path.covers(left_path),
        (ScopeCoverage::Subtree, ScopeCoverage::Subtree) => {
            left_path.covers(right_path) || right_path.covers(left_path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_and_coverage_are_independent() {
        let root = ScopePath::one("rates").unwrap();
        let leaf = ScopePath::new(["rates", "usd", "tenor", "5y"].map(str::to_owned)).unwrap();
        let sibling = ScopePath::new(["rates", "eur", "tenor", "5y"].map(str::to_owned)).unwrap();
        assert!(scopes_overlap(
            &root,
            ScopeCoverage::Subtree,
            &leaf,
            ScopeCoverage::Exact
        ));
        assert!(!scopes_overlap(
            &root,
            ScopeCoverage::Exact,
            &leaf,
            ScopeCoverage::Exact
        ));
        assert!(!scopes_overlap(
            &leaf,
            ScopeCoverage::Subtree,
            &sibling,
            ScopeCoverage::Subtree
        ));
        assert!(ScopePath::new((0..8).map(|n| n.to_string())).is_ok());
        assert_eq!(
            ScopePath::new((0..9).map(|n| n.to_string())),
            Err(ScopePathError::TooDeep)
        );
    }

    #[test]
    fn deserialization_enforces_the_same_path_boundary() {
        assert!(serde_json::from_str::<ScopePath>(r#"[]"#).is_err());
        assert!(serde_json::from_str::<ScopePath>(r#"["rates", ""]"#).is_err());
        let too_deep =
            serde_json::to_string(&(0..9).map(|n| n.to_string()).collect::<Vec<_>>()).unwrap();
        assert!(serde_json::from_str::<ScopePath>(&too_deep).is_err());
        let valid =
            serde_json::to_string(&(0..8).map(|n| n.to_string()).collect::<Vec<_>>()).unwrap();
        assert_eq!(
            serde_json::from_str::<ScopePath>(&valid).unwrap().depth(),
            8
        );
    }
}
