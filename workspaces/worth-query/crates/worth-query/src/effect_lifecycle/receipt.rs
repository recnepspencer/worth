use crate::basis_lifecycle::BasisFamily;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::batch_execution::ExecutedEffectBatchPlan;
use super::counters::EffectLifecycleCounters;
use super::diagnostics::{EffectDiagnosticsMaterialization, EffectDiagnosticsRequest};
use super::envelope::SelfDescribingEffectEnvelope;
use super::execution::{ExecutedEffectAuthorityArtifact, ExecutedEffectPlan};
use super::execution_artifacts::{
    executed_authority_artifact_identity, writeback_bridge_evidence_identity,
    writeback_bridge_execution_receipt_evidence_identity,
    writeback_bridge_receipt_evidence_identity,
};
use super::inventory::EffectReceiptArtifactKind;
use super::planning::EffectAuthorityOwner;
use super::receipt_transitions::EffectReceiptTransitionRules;
use super::taxonomy::{EffectAuthorityLane, EffectFamily};
mod evidence;
pub use evidence::{
    EffectReceiptDecisionTrace, EffectReceiptIntegrityMarkers, EffectReceiptTargetEvidence,
};

#[derive(Clone, Debug, PartialEq)]
pub struct EffectExecutionReceipt {
    target_evidence: EffectReceiptTargetEvidence,
    write_count: usize,
    receipt_family: EffectReceiptArtifactKind,
    declared_effect_family: EffectFamily,
    authority_lane: EffectAuthorityLane,
    basis_family: BasisFamily,
    receipt_identity: WorthQueryEvidenceIdentity,
    decision_trace: EffectReceiptDecisionTrace,
    integrity_markers: EffectReceiptIntegrityMarkers,
    counters: EffectLifecycleCounters,
}

impl EffectExecutionReceipt {
    pub(super) fn from_scalar(executed: &ExecutedEffectPlan) -> Self {
        let receipt_family = match executed.artifact() {
            ExecutedEffectAuthorityArtifact::Writeback { .. } => {
                EffectReceiptArtifactKind::WorthQueryWriteReceipt
            }
            ExecutedEffectAuthorityArtifact::Mutation(_)
            | ExecutedEffectAuthorityArtifact::Merge(_) => {
                EffectReceiptArtifactKind::WorthQueryIntentExecution
            }
        };
        let declared_effect_family = executed.lowered().family();
        let authority_lane = executed.lowered().authority_lane();
        let basis_family = executed
            .lowered()
            .authority_scoped_plan()
            .admitted()
            .normalized()
            .basis_family();
        let execution_identity = scalar_execution_receipt_identity(executed, receipt_family);
        let receipt_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_execution_receipt_v1",
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("family"),
                    receipt_family.as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("execution"),
                    &execution_identity,
                )
                .seal();
        let decision_trace = EffectReceiptDecisionTrace::scalar(executed);
        let integrity_markers = EffectReceiptIntegrityMarkers::new(
            executed.artifact(),
            executed.counters(),
            &receipt_identity,
        );
        let counters = executed.counters().clone();
        let target_evidence = scalar_target_evidence(executed);
        Self {
            target_evidence,
            write_count: 1,
            receipt_family,
            declared_effect_family,
            authority_lane,
            basis_family,
            receipt_identity,
            decision_trace,
            integrity_markers,
            counters,
        }
    }

    pub(super) fn from_batch(executed: &ExecutedEffectBatchPlan) -> Self {
        let receipt_family = EffectReceiptArtifactKind::WorthQueryBatchWriteReceipt;
        let declared_effect_family = EffectFamily::Mutation;
        let authority_lane = executed.authority_lane();
        let basis_family = executed.basis_family();
        let execution_identity = batch_execution_receipt_identity(executed);
        let receipt_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_execution_receipt_v1",
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("family"),
                    receipt_family.as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("execution"),
                    &execution_identity,
                )
                .seal();
        let decision_trace = EffectReceiptDecisionTrace::batch(executed);
        let integrity_markers = EffectReceiptIntegrityMarkers::new(
            executed.aggregate_artifact(),
            executed.counters(),
            &receipt_identity,
        );
        let counters = executed.counters().clone();
        let aggregate = executed
            .aggregate_mutation()
            .expect("phase 5 batch receipts remain mutation-only");
        let target_evidence = EffectReceiptTargetEvidence::BatchMutation {
            commit_id: aggregate.outcome().commit.commit_id.0,
            version_id: aggregate.outcome().commit.version_id.0,
            component_count: executed.components().len(),
        };
        Self {
            target_evidence,
            write_count: executed.components().len(),
            receipt_family,
            declared_effect_family,
            authority_lane,
            basis_family,
            receipt_identity,
            decision_trace,
            integrity_markers,
            counters,
        }
    }

    pub fn receipt_family(&self) -> EffectReceiptArtifactKind {
        self.receipt_family
    }

    pub fn declared_effect_family(&self) -> EffectFamily {
        self.declared_effect_family
    }

    pub fn authority_lane(&self) -> EffectAuthorityLane {
        self.authority_lane
    }

    pub fn basis_lane(&self) -> BasisFamily {
        self.basis_family
    }

    pub fn authority_owner(&self) -> EffectAuthorityOwner {
        self.decision_trace.authority_owner()
    }

    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }

    pub fn receipt_for_reporting(&self) -> &str {
        self.receipt_identity.as_str()
    }

    pub fn decision_trace(&self) -> &EffectReceiptDecisionTrace {
        &self.decision_trace
    }

    pub fn integrity_markers(&self) -> &EffectReceiptIntegrityMarkers {
        &self.integrity_markers
    }

    pub fn delivery_counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }

    pub fn write_count(&self) -> usize {
        self.write_count
    }

    pub fn effect_envelope(&self) -> SelfDescribingEffectEnvelope {
        SelfDescribingEffectEnvelope::from_receipt(self)
    }

    pub fn materialize_diagnostics(
        &self,
        request: EffectDiagnosticsRequest,
    ) -> EffectDiagnosticsMaterialization {
        let envelope = self.effect_envelope();
        EffectDiagnosticsMaterialization::from_receipt(self, &envelope, request)
    }

    pub fn transition_rules(&self) -> EffectReceiptTransitionRules {
        EffectReceiptTransitionRules::for_receipt_family(self.receipt_family)
    }

    pub fn target_evidence(&self) -> EffectReceiptTargetEvidence {
        self.target_evidence.clone()
    }

    pub fn lowered_for_reporting(&self) -> &str {
        self.decision_trace.lowered_for_reporting()
    }
}

fn scalar_target_evidence(executed: &ExecutedEffectPlan) -> EffectReceiptTargetEvidence {
    match executed.artifact() {
        ExecutedEffectAuthorityArtifact::Mutation(result) => {
            EffectReceiptTargetEvidence::MutationCommit {
                commit_id: result.outcome().commit.commit_id.0,
                version_id: result.outcome().commit.version_id.0,
            }
        }
        ExecutedEffectAuthorityArtifact::Merge(result) => {
            EffectReceiptTargetEvidence::MergeCommit {
                commit_id: result.commit.outcome().commit.commit_id.0,
                version_id: result.commit.outcome().commit.version_id.0,
            }
        }
        ExecutedEffectAuthorityArtifact::Writeback { execution } => {
            EffectReceiptTargetEvidence::Writeback {
                outcome_identity: writeback_bridge_evidence_identity(
                    "outcome",
                    execution.outcome(),
                ),
                authority_receipt_identity: writeback_bridge_receipt_evidence_identity(
                    "authority_receipt",
                    execution.authority_receipt(),
                ),
                execution_receipt_identity: writeback_bridge_execution_receipt_evidence_identity(
                    "execution_receipt",
                    execution.execution_receipt(),
                ),
            }
        }
    }
}

fn scalar_execution_receipt_identity(
    executed: &ExecutedEffectPlan,
    receipt_family: EffectReceiptArtifactKind,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "effect_execution_receipt_execution_v1",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("family"),
            receipt_family.as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("lowered"),
            executed.lowered().lowered_effect_execution_plan_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("authority_artifact"),
            &executed_authority_artifact_identity(executed.artifact()),
        )
        .seal()
}

fn batch_execution_receipt_identity(
    executed: &ExecutedEffectBatchPlan,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "effect_execution_receipt_execution_v1",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("family"),
            EffectReceiptArtifactKind::WorthQueryBatchWriteReceipt.as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("lowered"),
            executed.lowered().batch_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("authority_artifact"),
            &executed_authority_artifact_identity(executed.aggregate_artifact()),
        )
        .seal()
}
