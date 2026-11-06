use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::super::batch_execution::ExecutedEffectBatchPlan;
use super::super::counters::EffectLifecycleCounters;
use super::super::execution::{ExecutedEffectAuthorityArtifact, ExecutedEffectPlan};
use super::super::execution_artifacts::executed_authority_artifact_identity;
use super::super::planning::EffectAuthorityOwner;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectReceiptTargetEvidence {
    MutationCommit {
        commit_id: u64,
        version_id: u64,
    },
    MergeCommit {
        commit_id: u64,
        version_id: u64,
    },
    Writeback {
        outcome_identity: WorthQueryEvidenceIdentity,
        authority_receipt_identity: WorthQueryEvidenceIdentity,
        execution_receipt_identity: WorthQueryEvidenceIdentity,
    },
    BatchMutation {
        commit_id: u64,
        version_id: u64,
        component_count: usize,
    },
}

impl EffectReceiptTargetEvidence {
    pub fn writeback_outcome_for_reporting(&self) -> Option<&str> {
        match self {
            Self::Writeback {
                outcome_identity, ..
            } => Some(outcome_identity.as_str()),
            _ => None,
        }
    }

    pub fn writeback_authority_receipt_for_reporting(&self) -> Option<&str> {
        match self {
            Self::Writeback {
                authority_receipt_identity,
                ..
            } => Some(authority_receipt_identity.as_str()),
            _ => None,
        }
    }

    pub fn writeback_execution_receipt_for_reporting(&self) -> Option<&str> {
        match self {
            Self::Writeback {
                execution_receipt_identity,
                ..
            } => Some(execution_receipt_identity.as_str()),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectReceiptDecisionTrace {
    admitted_or_batch_identity: WorthQueryEvidenceIdentity,
    lowered_identity: WorthQueryEvidenceIdentity,
    authority_owner: EffectAuthorityOwner,
    decision_trace_identity: WorthQueryEvidenceIdentity,
}

impl EffectReceiptDecisionTrace {
    pub(super) fn scalar(executed: &ExecutedEffectPlan) -> Result<Self, IdentityDerivationDenial> {
        let admitted_or_batch_identity = executed
            .lowered()
            .authority_scoped_plan()
            .admitted()
            .admitted_identity()
            .clone();
        let lowered_identity = executed
            .lowered()
            .lowered_effect_execution_plan_identity()
            .clone();
        Ok(Self::new(
            admitted_or_batch_identity,
            lowered_identity,
            executed.authority_owner(),
            "admitted",
        )?)
    }

    pub(super) fn batch(
        executed: &ExecutedEffectBatchPlan,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            executed.lowered().admitted_batch_identity().clone(),
            executed.lowered().batch_identity().clone(),
            executed.authority_owner(),
            "admitted_batch",
        )?)
    }

    fn new(
        admitted_or_batch_identity: WorthQueryEvidenceIdentity,
        lowered_identity: WorthQueryEvidenceIdentity,
        authority_owner: EffectAuthorityOwner,
        admitted_tag: &'static str,
    ) -> Result<Self, IdentityDerivationDenial> {
        let decision_trace_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_receipt_decision_trace_v1",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new(admitted_tag),
                    &admitted_or_batch_identity,
                )
                .field_evidence_identity(WorthQueryEvidenceTag::new("lowered"), &lowered_identity)
                .field_shape(
                    WorthQueryEvidenceTag::new("authority_owner"),
                    authority_owner.as_str(),
                )
                .seal()?;
        Ok(Self {
            admitted_or_batch_identity,
            lowered_identity,
            authority_owner,
            decision_trace_identity,
        })
    }

    pub fn admitted_or_batch_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.admitted_or_batch_identity
    }
    pub fn admitted_or_batch_for_reporting(&self) -> &str {
        self.admitted_or_batch_identity.as_str()
    }
    pub fn lowered_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.lowered_identity
    }
    pub fn lowered_for_reporting(&self) -> &str {
        self.lowered_identity.as_str()
    }
    pub fn authority_owner(&self) -> EffectAuthorityOwner {
        self.authority_owner
    }
    pub fn decision_trace_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.decision_trace_identity
    }
    pub fn decision_trace_for_reporting(&self) -> &str {
        self.decision_trace_identity.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectReceiptIntegrityMarkers {
    authority_artifact_identity: WorthQueryEvidenceIdentity,
    counter_snapshot_identity: WorthQueryEvidenceIdentity,
    integrity_identity: WorthQueryEvidenceIdentity,
}

impl EffectReceiptIntegrityMarkers {
    pub(super) fn new(
        authority_artifact: &ExecutedEffectAuthorityArtifact,
        counters: &EffectLifecycleCounters,
        receipt_identity: &WorthQueryEvidenceIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        let authority_artifact_identity = executed_authority_artifact_identity(authority_artifact)?;
        let counter_snapshot_identity = counters.evidence_identity();
        let integrity_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_receipt_integrity_markers_v1",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("authority_artifact"),
                    &authority_artifact_identity,
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("counters"),
                    &counter_snapshot_identity,
                )
                .field_evidence_identity(WorthQueryEvidenceTag::new("receipt"), receipt_identity)
                .seal()?;
        Ok(Self {
            authority_artifact_identity,
            counter_snapshot_identity,
            integrity_identity,
        })
    }

    pub fn authority_artifact_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.authority_artifact_identity
    }
    pub fn authority_artifact_for_reporting(&self) -> &str {
        self.authority_artifact_identity.as_str()
    }
    pub fn counter_snapshot_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.counter_snapshot_identity
    }
    pub fn counter_snapshot_for_reporting(&self) -> &str {
        self.counter_snapshot_identity.as_str()
    }
    pub fn integrity_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.integrity_identity
    }
    pub fn integrity_for_reporting(&self) -> &str {
        self.integrity_identity.as_str()
    }
}
