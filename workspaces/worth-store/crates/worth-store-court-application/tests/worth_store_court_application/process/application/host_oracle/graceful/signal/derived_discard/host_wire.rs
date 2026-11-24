//! Serializable wire protocol types and receipt helpers for the Query Host + Signal D14 child.
use super::super::{opening, operations};
use std::collections::BTreeMap;
use worth_foundational::facade::RecordedProductBranchReference;
use worth_query_host::facade::primary_graph::WorthQueryRecoveredApplicationCommitReceipt;

pub(super) const EXPECTED_CADENCE_KEYS: [&str; 5] = [
    "relational.branch-transitions",
    "relational.canonical-commits",
    "world.branches",
    "world.custody",
    "world.occurrences",
];

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct ReceiptSnapshot {
    pub(super) ordinal: u8,
    pub(super) commit_id: u64,
    pub(super) version_id: u64,
    pub(super) truth_version: u64,
    pub(super) outcome_identity: Option<u64>,
    pub(super) changed_record_count: usize,
    pub(super) emitted_effect_count: usize,
    pub(super) idempotency_key: [u8; 32],
    pub(super) product_incarnation: u64,
    pub(super) product_generation: u64,
    pub(super) composite_commit: u64,
    pub(super) attempt_ordinal: u64,
    pub(super) native_journal_position: u64,
    pub(super) world_journal_position: Option<u64>,
    pub(super) native_artifact_digest: [u8; 32],
    pub(super) world_artifact_digest: Option<[u8; 32]>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) enum Command {
    PrepareCheckpointDiscard,
    ReplayCommitted,
    Continue,
    Close { expect_discarded: bool },
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Reply {
    pub(super) states: Vec<operations::State>,
    pub(super) frontiers: Vec<(u64, u64, u64)>,
    pub(super) initialized: usize,
    pub(super) contacts: (usize, usize, usize, usize),
    pub(super) cadence: BTreeMap<String, (u64, u64)>,
    pub(super) discard_generation: Option<u64>,
    pub(super) close_checkpoint_generation: Option<u64>,
    pub(super) original_receipts: Vec<ReceiptSnapshot>,
    pub(super) replayed_receipts: Vec<ReceiptSnapshot>,
    pub(super) closed: bool,
}

pub(super) fn inspect_cadence(
    home: &opening::Home,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> BTreeMap<String, (u64, u64)> {
    home.inspect_checkpoint(execution)
        .unwrap()
        .journals
        .into_iter()
        .map(|j| (j.name.to_owned(), (j.covered, j.head)))
        .collect()
}

fn recorded_head_coords(head: &RecordedProductBranchReference) -> (u64, u64, u64) {
    match head {
        RecordedProductBranchReference::Home(h) => (
            h.incarnation().ordinal(),
            h.generation().get(),
            h.selected_commit().ordinal(),
        ),
        RecordedProductBranchReference::Memory(h) => {
            (h.incarnation(), h.generation(), h.selected_commit())
        }
    }
}

pub(super) fn snapshot_recovered(
    ordinal: u8,
    receipt: &WorthQueryRecoveredApplicationCommitReceipt,
) -> ReceiptSnapshot {
    let (inc, generation, commit) = recorded_head_coords(receipt.new_product_head());
    let native_artifact_digest = *receipt.native_artifact_digest();
    let world_artifact_digest = receipt.world_artifact_digest().copied();
    assert_ne!(native_artifact_digest, [0_u8; 32]);
    assert!(world_artifact_digest.is_some_and(|d| d != [0_u8; 32]));
    ReceiptSnapshot {
        ordinal,
        commit_id: receipt.commit_reference().commit_id.0,
        version_id: receipt.commit_reference().version_id.as_u64(),
        truth_version: receipt.basis_descriptor().truth_version().as_u64(),
        outcome_identity: Some(receipt.outcome_identity().get()),
        changed_record_count: receipt.changed_record_count(),
        emitted_effect_count: receipt.emitted_effect_count(),
        idempotency_key: *receipt.idempotency_key_identity(),
        product_incarnation: inc,
        product_generation: generation,
        composite_commit: commit,
        attempt_ordinal: receipt.attempt_identity().ordinal(),
        native_journal_position: receipt.native_journal_position().0,
        world_journal_position: receipt.world_journal_position(),
        native_artifact_digest,
        world_artifact_digest,
    }
}
