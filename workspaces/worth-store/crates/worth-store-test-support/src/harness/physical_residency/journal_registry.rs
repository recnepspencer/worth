//! Journal namespace declared by physical Store test homes.
use worth_store::physical_runtime::{
    OrderedJournalRegistry, RELATIONAL_BRANCH_TRANSITIONS, RELATIONAL_CANONICAL_COMMITS,
};
pub(super) fn declaration() -> OrderedJournalRegistry {
    OrderedJournalRegistry::declare(
        2,
        &[RELATIONAL_CANONICAL_COMMITS, RELATIONAL_BRANCH_TRANSITIONS],
        &["home", "relational.checkpoint"],
    )
    .unwrap()
}
