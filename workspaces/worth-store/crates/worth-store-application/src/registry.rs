//! The closed application journal namespace; later owners use its reserved names.

use worth_store::physical_runtime::{
    OrderedJournalRegistry, RELATIONAL_BRANCH_TRANSITIONS, RELATIONAL_CANONICAL_COMMITS,
};

pub(crate) const HOME_ANCHOR: &str = "home";
pub(crate) const CHECKPOINT_ANCHOR: &str = "home.checkpoint";

pub(crate) fn registry() -> OrderedJournalRegistry {
    OrderedJournalRegistry::declare(
        3,
        &[
            "home.transitions",
            RELATIONAL_CANONICAL_COMMITS,
            RELATIONAL_BRANCH_TRANSITIONS,
            "world.occurrences",
            "world.branches",
            "world.custody",
            "signal.definitions",
            "signal.advances",
            "query.obligations",
            "query.dispatch",
            "query.corrections",
        ],
        &[HOME_ANCHOR, CHECKPOINT_ANCHOR],
    )
    .expect("the application registry declares unique disjoint names")
}
