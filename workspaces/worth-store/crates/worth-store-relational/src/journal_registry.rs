//! The closed journal set of Relational home format 2.
use worth_store::physical_runtime::{
    OrderedJournalRegistry, RELATIONAL_BRANCH_TRANSITIONS, RELATIONAL_CANONICAL_COMMITS,
};
pub(crate) fn registry() -> OrderedJournalRegistry {
    OrderedJournalRegistry::declare(
        2,
        &[RELATIONAL_CANONICAL_COMMITS, RELATIONAL_BRANCH_TRANSITIONS],
        &["home", "relational.checkpoint"],
    )
    .expect("the home format declares unique disjoint names")
}
