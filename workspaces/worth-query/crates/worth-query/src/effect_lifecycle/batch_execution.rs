use crate::basis_lifecycle::BasisFamily;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::batch::LoweredEffectBatchExecutionPlan;
use super::counters::EffectLifecycleCounters;
use super::execution::{
    EffectExecutionDenialKind, ExecutedEffectAuthorityArtifact, ExecutedEffectPlan,
};
use super::execution_artifacts::executed_authority_artifact_identity;
use super::planning::EffectAuthorityOwner;
use super::receipt::EffectExecutionReceipt;
use super::taxonomy::EffectAuthorityLane;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectBatchExecutionDenialKind {
    ComponentExecutionDenied(EffectExecutionDenialKind),
    AggregateExecutionDenied(EffectExecutionDenialKind),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectBatchExecutionDenial {
    component_index: Option<usize>,
    kind: EffectBatchExecutionDenialKind,
    message: String,
}

impl EffectBatchExecutionDenial {
    pub(crate) fn aggregate(kind: EffectExecutionDenialKind, message: impl Into<String>) -> Self {
        Self {
            component_index: None,
            kind: EffectBatchExecutionDenialKind::AggregateExecutionDenied(kind),
            message: message.into(),
        }
    }

    pub fn component_index(&self) -> Option<usize> {
        self.component_index
    }

    pub fn kind(&self) -> &EffectBatchExecutionDenialKind {
        &self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectBatchExecutionDeferred {
    kind: super::EffectExecutionDeferredKind,
    message: String,
    batch_identity: WorthQueryEvidenceIdentity,
    counters: EffectLifecycleCounters,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectBatchExecutionControlStopped {
    kind: super::EffectExecutionControlStopKind,
    message: String,
    batch_identity: WorthQueryEvidenceIdentity,
    counters: EffectLifecycleCounters,
}

impl EffectBatchExecutionControlStopped {
    pub(crate) fn new(
        lowered: &LoweredEffectBatchExecutionPlan,
        kind: super::EffectExecutionControlStopKind,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            batch_identity: lowered.batch_identity().clone(),
            counters: EffectLifecycleCounters::execution_denied(
                lowered.counters().effect_support_row_count(),
                lowered.counters().effect_lowering_width(),
                lowered.counters().effect_executor_rediscovery_count(),
            ),
        }
    }

    pub const fn kind(&self) -> super::EffectExecutionControlStopKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn batch_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.batch_identity
    }

    pub const fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }
}

impl EffectBatchExecutionDeferred {
    pub(crate) fn new(
        lowered: &LoweredEffectBatchExecutionPlan,
        kind: super::EffectExecutionDeferredKind,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            batch_identity: lowered.batch_identity().clone(),
            counters: EffectLifecycleCounters::deferred(
                lowered.counters().effect_support_row_count(),
            ),
        }
    }

    pub fn kind(&self) -> super::EffectExecutionDeferredKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn batch_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.batch_identity
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectBatchSettlementDeferred {
    message: String,
    batch_identity: WorthQueryEvidenceIdentity,
    outcome_identity: WorthQueryEvidenceIdentity,
    counters: EffectLifecycleCounters,
    settlement: worth_relational::facade::publication::DeferredPublicationSettlement,
}

impl EffectBatchSettlementDeferred {
    pub(crate) fn new(
        lowered: &LoweredEffectBatchExecutionPlan,
        message: impl Into<String>,
        settlement: worth_relational::facade::publication::DeferredPublicationSettlement,
    ) -> Self {
        let message = message.into();
        let batch_identity = lowered.batch_identity().clone();
        let commit = settlement.commit();
        let outcome_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_batch_settlement_deferred_v1",
                )
                .field_evidence_identity(WorthQueryEvidenceTag::new("batch"), &batch_identity)
                .field_usize(
                    WorthQueryEvidenceTag::new("commit_id"),
                    commit.commit_id.0 as usize,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("version_id"),
                    commit.version_id.0 as usize,
                )
                .seal();
        Self {
            message,
            batch_identity,
            outcome_identity,
            counters: EffectLifecycleCounters::publication_settlement_deferred(
                lowered.counters().effect_support_row_count(),
                lowered.counters().effect_lowering_width(),
                lowered.counters().effect_executor_rediscovery_count(),
                1,
            ),
            settlement,
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn batch_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.batch_identity
    }

    pub fn outcome_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.outcome_identity
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }

    pub fn commit_id(&self) -> worth_relational::facade::history::CommitId {
        self.settlement.commit().commit_id
    }

    pub fn repair_with(
        &self,
        authority: super::EffectExecutionAuthority<'_>,
    ) -> Result<
        worth_relational::facade::history::RelationalCommitReceipt,
        super::EffectSettlementRepairError,
    > {
        super::settlement_repair::repair_effect_settlement(authority, &self.settlement)
    }

    #[cfg(test)]
    pub(crate) fn settlement(
        &self,
    ) -> &worth_relational::facade::publication::DeferredPublicationSettlement {
        &self.settlement
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectBatchExecutionStop {
    Denied(EffectBatchExecutionDenial),
    Deferred(EffectBatchExecutionDeferred),
    ControlStopped(EffectBatchExecutionControlStopped),
    SettlementDeferred(EffectBatchSettlementDeferred),
}

impl EffectBatchExecutionStop {
    pub fn denial(&self) -> Option<&EffectBatchExecutionDenial> {
        match self {
            Self::Denied(denial) => Some(denial),
            Self::Deferred(_) | Self::ControlStopped(_) | Self::SettlementDeferred(_) => None,
        }
    }

    pub fn deferred(&self) -> Option<&EffectBatchExecutionDeferred> {
        match self {
            Self::Deferred(deferred) => Some(deferred),
            Self::Denied(_) | Self::ControlStopped(_) | Self::SettlementDeferred(_) => None,
        }
    }

    pub fn settlement_deferred(&self) -> Option<&EffectBatchSettlementDeferred> {
        match self {
            Self::Denied(_) | Self::Deferred(_) | Self::ControlStopped(_) => None,
            Self::SettlementDeferred(deferred) => Some(deferred),
        }
    }

    pub fn control_stopped(&self) -> Option<&EffectBatchExecutionControlStopped> {
        match self {
            Self::ControlStopped(stopped) => Some(stopped),
            Self::Denied(_) | Self::Deferred(_) | Self::SettlementDeferred(_) => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutedEffectBatchPlan {
    lowered: LoweredEffectBatchExecutionPlan,
    authority_lane: EffectAuthorityLane,
    basis_family: BasisFamily,
    authority_owner: EffectAuthorityOwner,
    aggregate_artifact: ExecutedEffectAuthorityArtifact,
    components: Vec<ExecutedEffectPlan>,
    batch_identity: WorthQueryEvidenceIdentity,
    counters: EffectLifecycleCounters,
}

impl ExecutedEffectBatchPlan {
    pub(crate) fn new(
        lowered: LoweredEffectBatchExecutionPlan,
        authority_lane: EffectAuthorityLane,
        basis_family: BasisFamily,
        authority_owner: EffectAuthorityOwner,
        aggregate_artifact: ExecutedEffectAuthorityArtifact,
        components: Vec<ExecutedEffectPlan>,
    ) -> Self {
        let batch_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "executed_effect_batch_plan_v2",
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("authority"),
                    authority_lane.as_str(),
                )
                .field_shape(WorthQueryEvidenceTag::new("basis"), basis_family.as_str())
                .field_shape(
                    WorthQueryEvidenceTag::new("owner"),
                    authority_owner.as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("lowered"),
                    lowered.batch_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("aggregate"),
                    &executed_authority_artifact_identity(&aggregate_artifact),
                )
                .field_evidence_identity_sequence(
                    WorthQueryEvidenceTag::new("component"),
                    components
                        .iter()
                        .map(ExecutedEffectPlan::effect_execution_identity),
                )
                .seal();
        let counters = EffectLifecycleCounters::executed_batch(
            components.len(),
            components
                .iter()
                .map(|component| component.counters().effect_lowering_width())
                .sum(),
            components
                .iter()
                .map(|component| component.counters().effect_executor_rediscovery_count())
                .sum(),
            1,
        );
        Self {
            lowered,
            authority_lane,
            basis_family,
            authority_owner,
            aggregate_artifact,
            components,
            batch_identity,
            counters,
        }
    }

    pub fn authority_lane(&self) -> EffectAuthorityLane {
        self.authority_lane
    }

    pub fn lowered(&self) -> &LoweredEffectBatchExecutionPlan {
        &self.lowered
    }

    pub fn basis_family(&self) -> BasisFamily {
        self.basis_family
    }

    pub fn authority_owner(&self) -> EffectAuthorityOwner {
        self.authority_owner
    }

    pub fn aggregate_artifact(&self) -> &ExecutedEffectAuthorityArtifact {
        &self.aggregate_artifact
    }

    pub fn aggregate_mutation(
        &self,
    ) -> Option<&worth_relational::facade::transactions::CommitResult> {
        match &self.aggregate_artifact {
            ExecutedEffectAuthorityArtifact::Mutation(result) => Some(result),
            _ => None,
        }
    }

    pub fn components(&self) -> &[ExecutedEffectPlan] {
        &self.components
    }

    pub fn batch_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.batch_identity
    }

    pub fn batch_for_reporting(&self) -> &str {
        self.batch_identity.as_str()
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }

    pub fn receipt(&self) -> EffectExecutionReceipt {
        EffectExecutionReceipt::from_batch(self)
    }

    pub(crate) fn published_snapshot(
        &self,
    ) -> Option<&worth_relational::facade::snapshots::SnapshotHandle> {
        match &self.aggregate_artifact {
            ExecutedEffectAuthorityArtifact::Mutation(result) => Some(&result.snapshot),
            ExecutedEffectAuthorityArtifact::Merge(result) => Some(&result.commit.snapshot),
            ExecutedEffectAuthorityArtifact::Writeback { .. } => None,
        }
    }
}
