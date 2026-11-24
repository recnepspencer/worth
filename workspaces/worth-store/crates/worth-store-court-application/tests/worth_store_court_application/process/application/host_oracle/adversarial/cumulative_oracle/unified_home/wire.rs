//! Wire types and run-wide correspondence parity check for C4b.4 Single-Home Unified 4-Family Oracle.
use super::super::families_child::ops::ReceiptId;
use super::super::StepCtx;
use std::collections::BTreeMap;
use worth_store_testkit_application_oracle::facade::{
    host_observation::{comparison, projection::HeadProjection},
    DurableIdentityCorrespondence,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct UnifiedBranchSnap {
    pub(super) retention: u64,
    pub(super) program_rev: String,
    pub(super) branch_ordinal: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct UnifiedSnap {
    pub(super) main_head: HeadProjection,
    pub(super) branches: Vec<UnifiedBranchSnap>,
    pub(super) custody_count: usize,
    pub(super) wf_states: Vec<String>,
    pub(super) cadence: BTreeMap<String, (u64, u64)>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) enum UnifiedCmd {
    SetRetention {
        branch_idx: usize,
        days: u64,
        key: u64,
    },
    ForkBranch,
    SeedWorkflow,
    RunCorrectableStep,
    RunSecondWfToHistorical,
    AdvanceWorkflowHistorical,
    AdoptP1,
    ContinueCustody,
    CheckpointAndDiscard,
    Close {
        expect_discarded: bool,
    },
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub(super) struct UnifiedReply {
    pub(super) snap: Option<UnifiedSnap>,
    pub(super) fate_tag: Option<String>,
    pub(super) receipt_id: Option<ReceiptId>,
    pub(super) wf_locator: Option<Vec<u8>>,
    pub(super) discard_gen: Option<u64>,
    pub(super) close_gen: Option<u64>,
    pub(super) closed: bool,
}

pub(super) fn assert_unified_parity(
    map: &mut DurableIdentityCorrespondence,
    actual: &UnifiedSnap,
    reference: &UnifiedSnap,
    ctx: &StepCtx,
) {
    assert_eq!(
        actual.branches.len(),
        reference.branches.len(),
        "{ctx}: branch count"
    );
    assert_eq!(
        actual.wf_states, reference.wf_states,
        "{ctx}: workflow instance states"
    );
    comparison::compare_head(map, &actual.main_head, &reference.main_head).unwrap_or_else(|e| {
        panic!("{ctx}: main head divergence under run-wide correspondence: {e:?}")
    });
    for (idx, (a, r)) in actual.branches.iter().zip(&reference.branches).enumerate() {
        assert_eq!(a.retention, r.retention, "{ctx}: branch#{idx} retention");
        assert_eq!(
            a.program_rev, r.program_rev,
            "{ctx}: branch#{idx} program_rev"
        );
        assert_eq!(
            a.branch_ordinal, r.branch_ordinal,
            "{ctx}: branch#{idx} ordinal"
        );
    }
}
