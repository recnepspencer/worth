//! Well-formed ordered-journal names. Membership is granted only by the
//! closed registry declared on the runtime's open or initialization request.

use worth_store_physical_format::RecordAnchorName;

/// The journal holding Relational's canonical commit artifacts.
pub const RELATIONAL_CANONICAL_COMMITS: &str = "relational.canonical-commits";

/// The journal holding Relational's branch transition artifacts.
pub const RELATIONAL_BRANCH_TRANSITIONS: &str = "relational.branch-transitions";

/// A well-formed journal name. A runtime checks membership in its own
/// home-declared registry before granting journal access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrderedJournalName(RecordAnchorName);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OrderedJournalNameDenial {
    /// Not 1 to 32 bytes of `[a-z0-9.-]`.
    Malformed,
}

impl OrderedJournalName {
    pub fn new(name: &str) -> Result<Self, OrderedJournalNameDenial> {
        let anchor = RecordAnchorName::new(name).ok_or(OrderedJournalNameDenial::Malformed)?;
        Ok(Self(anchor))
    }

    /// The journal of Relational's canonical commits.
    pub fn relational_canonical_commits() -> Self {
        Self::well_formed(RELATIONAL_CANONICAL_COMMITS)
    }

    /// The journal of Relational's branch transitions.
    pub fn relational_branch_transitions() -> Self {
        Self::well_formed(RELATIONAL_BRANCH_TRANSITIONS)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub(in crate::physical_runtime) const fn anchor(self) -> RecordAnchorName {
        self.0
    }

    fn well_formed(name: &'static str) -> Self {
        Self::new(name).expect("the named journal has valid syntax")
    }
}
