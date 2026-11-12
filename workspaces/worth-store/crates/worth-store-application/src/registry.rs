//! The closed application journal namespace and its current owner admission.

use worth_store::physical_runtime::{
    OrderedJournalRegistry, RELATIONAL_BRANCH_TRANSITIONS, RELATIONAL_CANONICAL_COMMITS,
};

pub(crate) const HOME_ANCHOR: &str = "home";
pub(crate) const CHECKPOINT_ANCHOR: &str = "home.checkpoint";
pub(crate) const RELATIONAL_INITIAL_INPUTS_ANCHOR: &str = "relational.initial-inputs";

#[derive(Clone, Copy)]
pub(crate) enum HomeJournalAdmission {
    Native,
    Signal,
    World,
    Transition,
    QueryObligations,
    QueryCorrections,
    QueryDispatch,
}

#[derive(Clone, Copy)]
pub(crate) struct HomeJournal {
    pub(crate) name: &'static str,
    pub(crate) admission: HomeJournalAdmission,
}

pub(crate) const HOME_JOURNALS: [HomeJournal; 11] = [
    HomeJournal {
        name: "home.transitions",
        admission: HomeJournalAdmission::Transition,
    },
    HomeJournal {
        name: RELATIONAL_CANONICAL_COMMITS,
        admission: HomeJournalAdmission::Native,
    },
    HomeJournal {
        name: RELATIONAL_BRANCH_TRANSITIONS,
        admission: HomeJournalAdmission::Native,
    },
    HomeJournal {
        name: "world.occurrences",
        admission: HomeJournalAdmission::World,
    },
    HomeJournal {
        name: "world.branches",
        admission: HomeJournalAdmission::World,
    },
    HomeJournal {
        name: "world.custody",
        admission: HomeJournalAdmission::World,
    },
    HomeJournal {
        name: "signal.definitions",
        admission: HomeJournalAdmission::Signal,
    },
    HomeJournal {
        name: "signal.advances",
        admission: HomeJournalAdmission::Signal,
    },
    HomeJournal {
        name: "query.obligations",
        admission: HomeJournalAdmission::QueryObligations,
    },
    HomeJournal {
        name: "query.dispatch",
        admission: HomeJournalAdmission::QueryDispatch,
    },
    HomeJournal {
        name: "query.corrections",
        admission: HomeJournalAdmission::QueryCorrections,
    },
];

pub(crate) fn registry() -> OrderedJournalRegistry {
    OrderedJournalRegistry::declare(
        3,
        &HOME_JOURNALS.map(|journal| journal.name),
        &[
            HOME_ANCHOR,
            CHECKPOINT_ANCHOR,
            RELATIONAL_INITIAL_INPUTS_ANCHOR,
        ],
    )
    .expect("the application registry declares unique disjoint names")
}
