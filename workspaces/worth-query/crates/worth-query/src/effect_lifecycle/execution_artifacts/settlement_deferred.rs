use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::super::counters::EffectLifecycleCounters;
use super::super::lowering::LoweredEffectExecutionPlan;
use super::EffectExecutionDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectExecutionDeferred {
    kind: super::super::EffectExecutionDeferredKind,
    message: String,
    lowered_effect_execution_plan_identity: WorthQueryEvidenceIdentity,
    outcome_identity: WorthQueryEvidenceIdentity,
    counters: EffectLifecycleCounters,
}

impl EffectExecutionDeferred {
    pub(crate) fn new(
        lowered: &LoweredEffectExecutionPlan,
        kind: super::super::EffectExecutionDeferredKind,
        message: impl Into<String>,
    ) -> Self {
        let message = message.into();
        let plan_identity = lowered.lowered_effect_execution_plan_identity().clone();
        let outcome_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_execution_deferred_v1",
                )
                .field_evidence_identity(WorthQueryEvidenceTag::new("plan"), &plan_identity)
                .field_shape(WorthQueryEvidenceTag::new("message"), message.as_str())
                .seal();
        Self {
            kind,
            message,
            lowered_effect_execution_plan_identity: plan_identity,
            outcome_identity,
            counters: EffectLifecycleCounters::deferred(
                lowered.counters().effect_support_row_count(),
            ),
        }
    }

    pub fn kind(&self) -> super::super::EffectExecutionDeferredKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn lowered_effect_execution_plan_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.lowered_effect_execution_plan_identity
    }

    pub fn outcome_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.outcome_identity
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectExecutionSettlementDeferred {
    message: String,
    lowered_effect_execution_plan_identity: WorthQueryEvidenceIdentity,
    outcome_identity: WorthQueryEvidenceIdentity,
    counters: EffectLifecycleCounters,
    settlement: worth_relational::facade::publication::DeferredPublicationSettlement,
}

impl EffectExecutionSettlementDeferred {
    pub(crate) fn new(
        lowered: &LoweredEffectExecutionPlan,
        message: impl Into<String>,
        settlement: worth_relational::facade::publication::DeferredPublicationSettlement,
    ) -> Self {
        let message = message.into();
        let plan_identity = lowered.lowered_effect_execution_plan_identity().clone();
        let commit = settlement.commit();
        let outcome_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_execution_settlement_deferred_v1",
                )
                .field_evidence_identity(WorthQueryEvidenceTag::new("plan"), &plan_identity)
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
            lowered_effect_execution_plan_identity: plan_identity,
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

    pub fn lowered_effect_execution_plan_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.lowered_effect_execution_plan_identity
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
        super::super::EffectSettlementRepairError,
    > {
        super::super::settlement_repair::repair_effect_settlement(authority, &self.settlement)
    }

    #[cfg(test)]
    pub(crate) fn settlement(
        &self,
    ) -> &worth_relational::facade::publication::DeferredPublicationSettlement {
        &self.settlement
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectExecutionStop {
    Denied(EffectExecutionDenial),
    Deferred(EffectExecutionDeferred),
    ControlStopped(super::EffectExecutionControlStopped),
    SettlementDeferred(EffectExecutionSettlementDeferred),
}

impl EffectExecutionStop {
    pub fn denial(&self) -> Option<&EffectExecutionDenial> {
        match self {
            Self::Denied(denial) => Some(denial),
            Self::Deferred(_) | Self::ControlStopped(_) | Self::SettlementDeferred(_) => None,
        }
    }

    pub fn deferred(&self) -> Option<&EffectExecutionDeferred> {
        match self {
            Self::Deferred(deferred) => Some(deferred),
            Self::Denied(_) | Self::ControlStopped(_) | Self::SettlementDeferred(_) => None,
        }
    }

    pub fn settlement_deferred(&self) -> Option<&EffectExecutionSettlementDeferred> {
        match self {
            Self::Denied(_) | Self::Deferred(_) | Self::ControlStopped(_) => None,
            Self::SettlementDeferred(deferred) => Some(deferred),
        }
    }

    pub fn control_stopped(&self) -> Option<&super::EffectExecutionControlStopped> {
        match self {
            Self::ControlStopped(stopped) => Some(stopped),
            Self::Denied(_) | Self::Deferred(_) | Self::SettlementDeferred(_) => None,
        }
    }
}
