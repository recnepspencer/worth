//! The work of one envelope step, its undo and its redo.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_testkit_document_retention::facade::{
    document_retention_model::host::{primary_graph, runtime, WorthQueryApplicationLimits},
    document_world_resources::{world_resources_with_capacity, WorldLifecycleCapacity},
};

/// One step that edits the model and member `m` with both output trees bound,
/// then its undo and its redo.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StepWork {
    /// Charged native work units of each operation.
    pub(crate) recording: u64,
    pub(crate) undo: u64,
    pub(crate) redo: u64,
    /// The history journal's own storage accesses over all three: node,
    /// attempt, custody, child-index and selection.
    pub(crate) journal: [u64; 5],
}

/// Room for the deepest measured history. Every depth uses the same limits.
fn limits() -> WorthQueryApplicationLimits {
    WorthQueryApplicationLimits::new(
        world_resources_with_capacity(
            1024,
            128 * 1024 * 1024,
            1024,
            WorldLifecycleCapacity {
                live_product_branches: 128,
                active_observations: 1024,
                owner_created_component_custody_records: 256,
            },
            WorthQueryAdmissionClock::system(),
        ),
        runtime::WorthQueryApplicationQueryResourceProfile::new(NonZeroUsize::new(128).unwrap()),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}

/// The work every request charged since the last call.
fn charged() -> u64 {
    let requests = advancement_observations_on_this_thread_for_test();
    assert!(requests.iter().all(|request| request.refusal.is_none()));
    requests
        .iter()
        .map(|request| request.report.charged_work())
        .sum()
}

/// Install a world with `siblings` members of another model, record `depth`
/// steps that end at the same model and member values, then measure one more
/// step, its undo and its redo.
pub(crate) fn measure(siblings: usize, depth: usize) -> StepWork {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_configured::<program::Program>(
        limits(),
        1,
        None,
        world::Configuration {
            sibling_members: siblings,
            ..Default::default()
        },
    );
    let mut document = attach_with_both_trees(&host);
    for remaining in (0..depth as u64).rev() {
        let key: &'static str = Box::leak(format!("earlier-{remaining}").into_boxed_str());
        document = record_step(&host, document, key, 2 + remaining % 7, 6 + remaining).0;
    }
    let before = snapshot(&host, &document);
    assert_eq!((before.model, before.member), (2, 6));
    let journal = document.observe_history_enrollment_for_test();
    // The court's own snapshots page the history, so journal accesses are
    // counted around the three operations only.
    let accesses = || {
        let probes = journal.snapshot();
        [
            probes.node_probes,
            probes.attempt_probes,
            probes.custody_probes,
            probes.child_index_probes,
            probes.selection_accesses,
        ]
    };
    let mut counted = [0; 5];
    let mut count = |from: [u64; 5]| {
        for (total, (from, to)) in counted.iter_mut().zip(from.into_iter().zip(accesses())) {
            *total += to - from;
        }
    };
    charged();
    let from = accesses();
    let (document, step) = record_step(&host, document, "measured", 9, 99);
    count(from);
    let recording = charged();
    let after = snapshot(&host, &document);
    charged();
    let from = accesses();
    navigate(&host, &document, Move::Undo, 0x57e0_c001);
    count(from);
    let undo = charged();
    assert_eq!(snapshot(&host, &document), before);
    charged();
    let from = accesses();
    navigate(&host, &document, Move::RedoTo(step.id()), 0x57e0_c002);
    count(from);
    let redo = charged();
    assert_eq!(snapshot(&host, &document), after);
    close(&host, document);
    StepWork {
        recording,
        undo,
        redo,
        journal: counted,
    }
}
