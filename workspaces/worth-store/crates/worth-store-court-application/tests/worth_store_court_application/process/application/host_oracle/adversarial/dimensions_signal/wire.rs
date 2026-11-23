//! Serializable wire protocol snapshots, custody records, and receipts for
//! C4b.3 fresh-process Signal+Bridge tests.
use super::super::signal_bridge_io_kills::SignalSnapshot;
use std::collections::BTreeMap;
use worth_foundational::facade::RecordedProductBranchReference;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryRecoveredApplicationCommitReceipt,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct WireSnapshot {
    pub(super) intent: (String, u64, String, String),
    pub(super) world_commit_ordinal: u64,
    pub(super) signal_generation: u64,
    pub(super) world_reference_generation: u64,
    pub(super) installed_conditional_definition_count: usize,
    pub(super) installed_conditional_target_ref_count: usize,
    pub(super) owned_signal_active_node_count: usize,
    pub(super) head_attempt_ordinal: Option<u64>,
    pub(super) head_native_commit_id: Option<u64>,
    pub(super) head_native_version_id: Option<u64>,
    pub(super) head_relational_changed: bool,
    pub(super) head_signal_changed: bool,
    pub(super) head_signal_movement: Option<String>,
    pub(super) branch_id: String,
    pub(super) truth_version: u64,
    pub(super) schema_commitment: [u8; 32],
    pub(super) posture: String,
}

impl WireSnapshot {
    pub(super) fn from_snapshot(s: &SignalSnapshot) -> Self {
        Self {
            intent: s.intent.clone(),
            world_commit_ordinal: s.world_commit_ordinal,
            signal_generation: s.details.signal_generation,
            world_reference_generation: s.details.world_reference_generation,
            installed_conditional_definition_count: s
                .details
                .installed_conditional_definition_count,
            installed_conditional_target_ref_count: s
                .details
                .installed_conditional_target_ref_count,
            owned_signal_active_node_count: s.details.owned_signal_active_node_count,
            head_attempt_ordinal: s.details.head_attempt_ordinal,
            head_native_commit_id: s.details.head_native_commit_id,
            head_native_version_id: s.details.head_native_version_id,
            head_relational_changed: s.details.head_relational_changed,
            head_signal_changed: s.details.head_signal_changed,
            head_signal_movement: s.details.head_signal_movement.clone(),
            branch_id: format!("{:?}", s.native_basis.branch_id),
            truth_version: s.native_basis.truth_version.as_u64(),
            schema_commitment: s.native_basis.schema_commitment,
            posture: format!("{:?}", s.native_basis.posture),
        }
    }

    pub(super) fn same_durable_semantics(&self, other: &Self) -> bool {
        self.intent == other.intent
            && self.signal_generation == other.signal_generation
            && self.world_reference_generation == other.world_reference_generation
            && self.installed_conditional_definition_count >= 1
            && self.installed_conditional_target_ref_count
                == self.installed_conditional_definition_count
            && self.owned_signal_active_node_count == other.owned_signal_active_node_count
            && self.head_relational_changed == other.head_relational_changed
            && self.head_signal_changed == other.head_signal_changed
            && self.head_signal_movement == other.head_signal_movement
            && self.branch_id == other.branch_id
            && self.truth_version == other.truth_version
            && self.schema_commitment == other.schema_commitment
            && self.posture == other.posture
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct WireCustody {
    pub(super) owner_debug: String,
    pub(super) custody_ordinal: u64,
    pub(super) attempt_ordinal: u64,
    pub(super) native_commit_id: Option<u64>,
    pub(super) native_version_id: Option<u64>,
    pub(super) expected_commit: u64,
    pub(super) owner_effect_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct WireReceipt {
    pub(super) commit_id: u64,
    pub(super) version_id: u64,
    pub(super) truth_version: u64,
    pub(super) idempotency_key: [u8; 32],
    pub(super) product_incarnation: u64,
    pub(super) product_generation: u64,
    pub(super) composite_commit: u64,
    pub(super) attempt_ordinal: u64,
}

impl WireReceipt {
    pub(super) fn from_live(r: &WorthQueryApplicationCommitReceipt) -> Self {
        let p = r.committed_product_publication();
        Self {
            commit_id: r.commit_reference().commit_id.0,
            version_id: r.commit_reference().version_id.as_u64(),
            truth_version: r.basis_descriptor().truth_version().as_u64(),
            idempotency_key: *r.idempotency_binding().key_identity(),
            product_incarnation: p.product_incarnation().ordinal(),
            product_generation: p.product_generation().get(),
            composite_commit: p.composite_commit().ordinal(),
            attempt_ordinal: p.publication_attempt().ordinal(),
        }
    }

    pub(super) fn from_recovered(r: &WorthQueryRecoveredApplicationCommitReceipt) -> Self {
        let (inc, generation, commit) = match r.new_product_head() {
            RecordedProductBranchReference::Home(h) => (
                h.incarnation().ordinal(),
                h.generation().get(),
                h.selected_commit().ordinal(),
            ),
            RecordedProductBranchReference::Memory(h) => {
                (h.incarnation(), h.generation(), h.selected_commit())
            }
        };
        Self {
            commit_id: r.commit_reference().commit_id.0,
            version_id: r.commit_reference().version_id.as_u64(),
            truth_version: r.basis_descriptor().truth_version().as_u64(),
            idempotency_key: *r.idempotency_key_identity(),
            product_incarnation: inc,
            product_generation: generation,
            composite_commit: commit,
            attempt_ordinal: r.attempt_identity().ordinal(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) enum SignalCmd {
    CheckpointAndDiscard,
    CommitMixed(u8),
    ContinueCustody,
    VerifyRecoveredBridge,
    VerifyBridgeHealthy {
        before_ord: u64,
        before_sig_gen: u64,
    },
    Close {
        expect_discarded: bool,
    },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct SignalReply {
    pub(super) snapshot: Option<WireSnapshot>,
    pub(super) custody: Vec<WireCustody>,
    pub(super) receipt: Option<WireReceipt>,
    pub(super) initialized: usize,
    pub(super) contacts: (usize, usize, usize, usize),
    pub(super) cadence: BTreeMap<String, (u64, u64)>,
    pub(super) commit_tag: Option<String>,
    pub(super) definition_identity: Option<[u8; 32]>,
    pub(super) discard_gen: Option<u64>,
    pub(super) close_gen: Option<u64>,
    pub(super) closed: bool,
}
