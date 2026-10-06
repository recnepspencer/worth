use worth_relational::facade::transactions::{CommitResult, MergeExecutionOutcome};
use worth_runtime_bridge::facade::{
    BridgeAdmittedWritebackExecution, BridgeAdmittedWritebackExecutionReceipt,
    BridgeWritebackAuthorityOutcome, BridgeWritebackFailureClass, BridgeWritebackOutcomeClass,
    TruthWritebackReceipt,
};
#[path = "execution_artifacts/authority_and_denial.rs"]
mod authority_and_denial;
#[path = "execution_artifacts/control_stopped.rs"]
mod control_stopped;
#[path = "execution_artifacts/settlement_deferred.rs"]
mod settlement_deferred;
pub(crate) use authority_and_denial::EffectMergeAuthority;
pub use authority_and_denial::{
    EffectExecutionAuthority, EffectExecutionDenial, EffectExecutionDenialKind,
};
pub use control_stopped::EffectExecutionControlStopped;
pub use settlement_deferred::{
    EffectExecutionDeferred, EffectExecutionSettlementDeferred, EffectExecutionStop,
};

use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::counters::EffectLifecycleCounters;
use super::lowering::LoweredEffectExecutionPlan;
use super::planning::EffectAuthorityOwner;
use super::receipt::EffectExecutionReceipt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutedEffectAuthorityArtifact {
    Mutation(CommitResult),
    Merge(MergeExecutionOutcome),
    Writeback {
        execution: BridgeAdmittedWritebackExecution,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutedEffectPlan {
    lowered: LoweredEffectExecutionPlan,
    artifact: ExecutedEffectAuthorityArtifact,
    effect_execution_identity: WorthQueryEvidenceIdentity,
    counters: EffectLifecycleCounters,
}

impl ExecutedEffectPlan {
    pub(crate) fn new(
        lowered: LoweredEffectExecutionPlan,
        artifact: ExecutedEffectAuthorityArtifact,
        effect_execution_width: usize,
    ) -> Self {
        let effect_execution_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "executed_effect_plan_v1",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("plan"),
                    lowered.lowered_effect_execution_plan_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("artifact"),
                    &executed_authority_artifact_identity(&artifact),
                )
                .seal();
        let counters = EffectLifecycleCounters::executed(
            lowered.counters().effect_support_row_count(),
            lowered.counters().effect_lowering_width(),
            lowered.counters().effect_executor_rediscovery_count(),
            effect_execution_width,
        );
        Self {
            lowered,
            artifact,
            effect_execution_identity,
            counters,
        }
    }

    pub fn lowered(&self) -> &LoweredEffectExecutionPlan {
        &self.lowered
    }

    pub fn artifact(&self) -> &ExecutedEffectAuthorityArtifact {
        &self.artifact
    }

    pub fn authority_owner(&self) -> EffectAuthorityOwner {
        self.lowered.authority_owner()
    }

    pub fn as_mutation(&self) -> Option<&CommitResult> {
        match &self.artifact {
            ExecutedEffectAuthorityArtifact::Mutation(result) => Some(result),
            ExecutedEffectAuthorityArtifact::Merge(_) => None,
            ExecutedEffectAuthorityArtifact::Writeback { .. } => None,
        }
    }

    pub fn as_merge(&self) -> Option<&MergeExecutionOutcome> {
        match &self.artifact {
            ExecutedEffectAuthorityArtifact::Mutation(_) => None,
            ExecutedEffectAuthorityArtifact::Merge(result) => Some(result),
            ExecutedEffectAuthorityArtifact::Writeback { .. } => None,
        }
    }

    pub fn as_writeback(
        &self,
    ) -> Option<(&BridgeWritebackAuthorityOutcome, &TruthWritebackReceipt)> {
        match &self.artifact {
            ExecutedEffectAuthorityArtifact::Writeback { execution } => {
                Some((execution.outcome(), execution.authority_receipt()))
            }
            _ => None,
        }
    }

    pub fn writeback_execution(&self) -> Option<&BridgeAdmittedWritebackExecution> {
        match &self.artifact {
            ExecutedEffectAuthorityArtifact::Writeback { execution } => Some(execution),
            _ => None,
        }
    }

    pub fn effect_execution_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.effect_execution_identity
    }

    pub fn effect_execution_for_reporting(&self) -> &str {
        self.effect_execution_identity.as_str()
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }

    pub fn receipt(&self) -> EffectExecutionReceipt {
        EffectExecutionReceipt::from_scalar(self)
    }

    pub(crate) fn published_snapshot(
        &self,
    ) -> Option<&worth_relational::facade::snapshots::SnapshotHandle> {
        match &self.artifact {
            ExecutedEffectAuthorityArtifact::Mutation(result) => Some(&result.snapshot),
            ExecutedEffectAuthorityArtifact::Merge(result) => Some(&result.commit.snapshot),
            ExecutedEffectAuthorityArtifact::Writeback { .. } => None,
        }
    }
}

pub(crate) fn executed_authority_artifact_identity(
    artifact: &ExecutedEffectAuthorityArtifact,
) -> WorthQueryEvidenceIdentity {
    match artifact {
        ExecutedEffectAuthorityArtifact::Mutation(result) => {
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "executed_effect_mutation_authority_artifact_v1",
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("commit_id"),
                    result.outcome().commit.commit_id.0 as usize,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("version_id"),
                    result.outcome().commit.version_id.0 as usize,
                )
                .seal()
        }
        ExecutedEffectAuthorityArtifact::Merge(result) => {
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "executed_effect_merge_authority_artifact_v1",
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("commit_id"),
                    result.commit.outcome().commit.commit_id.0 as usize,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("version_id"),
                    result.commit.outcome().commit.version_id.0 as usize,
                )
                .seal()
        }
        ExecutedEffectAuthorityArtifact::Writeback { execution } => {
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "executed_effect_writeback_authority_artifact_v1",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("execution"),
                    &executed_writeback_execution_identity(execution),
                )
                .seal()
        }
    }
}

fn executed_writeback_execution_identity(
    execution: &BridgeAdmittedWritebackExecution,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "executed_writeback_execution_v1",
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("outcome"),
            &writeback_bridge_evidence_identity("outcome", execution.outcome()),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("authority_receipt"),
            &writeback_bridge_receipt_evidence_identity(
                "authority_receipt",
                execution.authority_receipt(),
            ),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("execution_receipt"),
            &writeback_bridge_execution_receipt_evidence_identity(
                "execution_receipt",
                execution.execution_receipt(),
            ),
        )
        .seal()
}

pub(crate) fn writeback_bridge_evidence_identity(
    role: &str,
    outcome: &BridgeWritebackAuthorityOutcome,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "effect_writeback_bridge_outcome_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("role"), role)
        .field_shape(
            WorthQueryEvidenceTag::new("outcome_class"),
            writeback_outcome_class_label(outcome.outcome_class()),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("idempotence_digest"),
            outcome.idempotence_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("authoritative_artifact_digest"),
            outcome.authoritative_artifact_digest(),
        )
        .seal()
}

pub(crate) fn writeback_bridge_receipt_evidence_identity(
    role: &str,
    receipt: &TruthWritebackReceipt,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "effect_writeback_bridge_receipt_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("role"), role)
        .field_shape(
            WorthQueryEvidenceTag::new("outcome_class"),
            writeback_outcome_class_label(receipt.outcome_class()),
        )
        .optional_shape(
            WorthQueryEvidenceTag::new("failure_class"),
            receipt.failure_class().map(writeback_failure_class_label),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("request_digest"),
            receipt.request_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("authoritative_artifact_digest"),
            receipt.authoritative_artifact_digest(),
        )
        .seal()
}

pub(crate) fn writeback_bridge_execution_receipt_evidence_identity(
    role: &str,
    receipt: &BridgeAdmittedWritebackExecutionReceipt,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "effect_writeback_bridge_execution_receipt_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("role"), role)
        .field_shape(
            WorthQueryEvidenceTag::new("request_digest"),
            receipt.request_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("contract_digest"),
            receipt.contract_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("lowered_policy_digest"),
            receipt.lowered_policy_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("writeback_effect_artifact_digest"),
            receipt.writeback_effect_artifact_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("effect_intent_digest"),
            receipt.effect_intent_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("idempotence_digest"),
            receipt.idempotence_digest(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("authority_outcome_digest"),
            receipt.authority_outcome_digest(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("authority_receipt"),
            &writeback_bridge_receipt_evidence_identity(
                "authority_receipt",
                receipt.authority_receipt(),
            ),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("replay_bundle_digest"),
            receipt.replay_bundle_digest(),
        )
        .seal()
}

fn writeback_outcome_class_label(outcome_class: BridgeWritebackOutcomeClass) -> &'static str {
    match outcome_class {
        BridgeWritebackOutcomeClass::CanonicalNoop => "canonical-noop",
        BridgeWritebackOutcomeClass::AuthoritativeCommit => "authoritative-commit",
        BridgeWritebackOutcomeClass::Rejected => "rejected",
    }
}

fn writeback_failure_class_label(failure_class: BridgeWritebackFailureClass) -> &'static str {
    match failure_class {
        BridgeWritebackFailureClass::WritebackNotRequested => "writeback-not-requested",
        BridgeWritebackFailureClass::PolicyRejected => "policy-rejected",
        BridgeWritebackFailureClass::StrategyUnavailable => "strategy-unavailable",
        BridgeWritebackFailureClass::FamilyBindingMismatch => "family-binding-mismatch",
        BridgeWritebackFailureClass::StrategyDescriptorMismatch => "strategy-descriptor-mismatch",
        BridgeWritebackFailureClass::IdempotenceBasisMismatch => "idempotence-basis-mismatch",
        BridgeWritebackFailureClass::CausalityEffectMismatch => "causality-effect-mismatch",
        BridgeWritebackFailureClass::StaleTruthBasis => "stale-truth-basis",
        BridgeWritebackFailureClass::InvariantRejected => "invariant-rejected",
        BridgeWritebackFailureClass::MergeAuthorityRejected => "merge-authority-rejected",
        BridgeWritebackFailureClass::StrategyFailed => "strategy-failed",
        BridgeWritebackFailureClass::StrategyPanicked => "strategy-panicked",
        BridgeWritebackFailureClass::ReplayMismatch => "replay-mismatch",
        BridgeWritebackFailureClass::AuthorityDenied => "authority-denied",
        BridgeWritebackFailureClass::PreviewWritebackRejected => "preview-writeback-rejected",
    }
}
