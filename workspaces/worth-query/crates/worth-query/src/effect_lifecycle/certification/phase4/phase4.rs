#[path = "phase4_boundary_rows.rs"]
mod phase4_boundary_rows;
#[path = "phase4_execution_rows.rs"]
mod phase4_execution_rows;

use crate::basis_lifecycle::BasisFamily;
use crate::effect_lifecycle::counters::EffectLifecycleCounters;
use crate::identity::hash_parts;

use super::super::taxonomy::EffectFamily;
use super::EffectLifecycleSeededCertificationBundle;

use phase4_boundary_rows::{
    batch_lane_denial_row, deferred_replay_row, host_override_denial_row, preview_rebind_row,
    seeded_replay_row, stale_after_admission_row, stale_after_lowering_row,
};
use phase4_execution_rows::{
    batch_execution_row, branch_mutation_execution_row, bridge_oracle_row,
    bridge_writeback_execution_row, relational_merge_execution_row, relational_oracle_row,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectLifecyclePhase4LaneKind {
    BranchMutationExecution,
    RelationalMergeExecution,
    BridgeWritebackExecution,
    BatchExecution,
    BatchLaneDenial,
    PreviewRebind,
    DeferredReplay,
    HostOverrideDenial,
    StaleAfterAdmission,
    StaleAfterLowering,
    RelationalOracle,
    BridgeOracle,
    SeededReplay,
}

impl EffectLifecyclePhase4LaneKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BranchMutationExecution => "branch_mutation_execution",
            Self::RelationalMergeExecution => "relational_merge_execution",
            Self::BridgeWritebackExecution => "bridge_writeback_execution",
            Self::BatchExecution => "batch_execution",
            Self::BatchLaneDenial => "batch_lane_denial",
            Self::PreviewRebind => "preview_rebind",
            Self::DeferredReplay => "deferred_replay",
            Self::HostOverrideDenial => "host_override_denial",
            Self::StaleAfterAdmission => "stale_after_admission",
            Self::StaleAfterLowering => "stale_after_lowering",
            Self::RelationalOracle => "relational_oracle",
            Self::BridgeOracle => "bridge_oracle",
            Self::SeededReplay => "seeded_replay",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectLifecyclePhase4LaneOutcome {
    Executed,
    Denied,
    RebindRequired,
    Deferred,
    Verified,
    Certified,
}

impl EffectLifecyclePhase4LaneOutcome {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Executed => "executed",
            Self::Denied => "denied",
            Self::RebindRequired => "rebind_required",
            Self::Deferred => "deferred",
            Self::Verified => "verified",
            Self::Certified => "certified",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectLifecyclePhase4CertificationRow {
    lane_kind: EffectLifecyclePhase4LaneKind,
    outcome: EffectLifecyclePhase4LaneOutcome,
    basis_family: BasisFamily,
    effect_family: EffectFamily,
    evidence_digest: String,
    evidence_detail: String,
    counters: EffectLifecycleCounters,
    row_digest: String,
}

impl EffectLifecyclePhase4CertificationRow {
    pub(super) fn new(
        lane_kind: EffectLifecyclePhase4LaneKind,
        outcome: EffectLifecyclePhase4LaneOutcome,
        basis_family: BasisFamily,
        effect_family: EffectFamily,
        evidence_digest: String,
        evidence_detail: String,
        counters: EffectLifecycleCounters,
    ) -> Self {
        let row_digest = hash_parts(&[
            "effect_lifecycle_phase4_certification_row_v1".to_string(),
            format!("lane:{}", lane_kind.as_str()),
            format!("outcome:{}", outcome.as_str()),
            format!("basis:{}", basis_family.as_str()),
            format!("family:{}", effect_family.as_str()),
            format!("evidence:{evidence_digest}"),
            format!("detail:{evidence_detail}"),
            format!("counters:{}", counters.counter_for_reporting()),
        ]);
        Self {
            lane_kind,
            outcome,
            basis_family,
            effect_family,
            evidence_digest,
            evidence_detail,
            counters,
            row_digest,
        }
    }

    pub fn lane_kind(&self) -> EffectLifecyclePhase4LaneKind {
        self.lane_kind
    }

    pub fn outcome(&self) -> EffectLifecyclePhase4LaneOutcome {
        self.outcome
    }

    pub fn evidence_digest(&self) -> &str {
        &self.evidence_digest
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }

    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectLifecyclePhase4CertificationBundle {
    rows: Vec<EffectLifecyclePhase4CertificationRow>,
    seeded_bundle_digest: String,
    phase4_bundle_digest: String,
}

impl EffectLifecyclePhase4CertificationBundle {
    fn new(
        rows: Vec<EffectLifecyclePhase4CertificationRow>,
        seeded: &EffectLifecycleSeededCertificationBundle,
    ) -> Self {
        let seeded_bundle_digest = seeded.certification_bundle_digest().to_string();
        let phase4_bundle_digest = hash_parts(
            &std::iter::once("effect_lifecycle_phase4_certification_bundle_v1".to_string())
                .chain(std::iter::once(format!("seeded:{seeded_bundle_digest}")))
                .chain(rows.iter().map(|row| format!("row:{}", row.row_digest())))
                .collect::<Vec<_>>(),
        );
        Self {
            rows,
            seeded_bundle_digest,
            phase4_bundle_digest,
        }
    }

    pub fn rows(&self) -> &[EffectLifecyclePhase4CertificationRow] {
        &self.rows
    }

    #[cfg(test)]
    pub fn seeded_bundle_digest(&self) -> &str {
        &self.seeded_bundle_digest
    }

    pub fn phase4_bundle_digest(&self) -> &str {
        &self.phase4_bundle_digest
    }
}

pub fn certify_effect_lifecycle_phase4(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> EffectLifecyclePhase4CertificationBundle {
    let seeded = super::certify_effect_lifecycle_seeded(execution, 17, 12);
    let rows = vec![
        branch_mutation_execution_row(execution),
        relational_merge_execution_row(execution),
        bridge_writeback_execution_row(execution),
        batch_execution_row(execution),
        batch_lane_denial_row(),
        preview_rebind_row(),
        deferred_replay_row(),
        host_override_denial_row(execution),
        stale_after_admission_row(execution),
        stale_after_lowering_row(execution),
        relational_oracle_row(execution),
        bridge_oracle_row(execution),
        seeded_replay_row(&seeded),
    ];
    EffectLifecyclePhase4CertificationBundle::new(rows, &seeded)
}
