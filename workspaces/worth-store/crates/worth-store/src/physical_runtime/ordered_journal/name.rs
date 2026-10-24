//! The registry of ordered-journal names: the one place a home's ordered
//! journals are named.
//!
//! The set is closed. An ordered-journal name is a registered one, and every
//! registered name is reserved against the anchored-record door, so the two
//! doors never share an anchor and a journal's anchor only ever selects that
//! journal's records.

use worth_store_physical_format::RecordAnchorName;

/// The journal holding Relational's canonical commit artifacts.
pub const RELATIONAL_CANONICAL_COMMITS: &str = "relational.canonical-commits";

/// The journal holding Relational's branch transition artifacts.
pub const RELATIONAL_BRANCH_TRANSITIONS: &str = "relational.branch-transitions";

/// Every ordered journal a Store home may hold.
const REGISTERED: [&str; 2] = [RELATIONAL_CANONICAL_COMMITS, RELATIONAL_BRANCH_TRANSITIONS];

/// The name of one ordered journal: a valid anchor name registered as an
/// ordered journal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrderedJournalName(RecordAnchorName);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OrderedJournalNameDenial {
    /// Not 1 to 32 bytes of `[a-z0-9.-]`.
    Malformed,
    /// A valid anchor name that names no ordered journal.
    Unregistered,
}

impl OrderedJournalName {
    pub fn new(name: &str) -> Result<Self, OrderedJournalNameDenial> {
        let anchor = RecordAnchorName::new(name).ok_or(OrderedJournalNameDenial::Malformed)?;
        if !is_ordered_journal_name(anchor) {
            return Err(OrderedJournalNameDenial::Unregistered);
        }
        Ok(Self(anchor))
    }

    /// The journal of Relational's canonical commits.
    pub fn relational_canonical_commits() -> Self {
        Self::registered(RELATIONAL_CANONICAL_COMMITS)
    }

    /// The journal of Relational's branch transitions.
    pub fn relational_branch_transitions() -> Self {
        Self::registered(RELATIONAL_BRANCH_TRANSITIONS)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub(in crate::physical_runtime) const fn anchor(self) -> RecordAnchorName {
        self.0
    }

    fn registered(name: &'static str) -> Self {
        Self::new(name).expect("a registered journal name is a valid ordered-journal name")
    }
}

/// Whether `anchor` belongs to an ordered journal. Other Store doors refuse
/// such a name.
pub(in crate::physical_runtime) fn is_ordered_journal_name(anchor: RecordAnchorName) -> bool {
    REGISTERED.contains(&anchor.as_str())
}
