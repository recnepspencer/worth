use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::super::counters::EffectLifecycleCounters;
use super::super::lowering::LoweredEffectExecutionPlan;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectExecutionControlStopped {
    kind: super::super::EffectExecutionControlStopKind,
    message: String,
    lowered_effect_execution_plan_identity: WorthQueryEvidenceIdentity,
    outcome_identity: Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial>,
    counters: EffectLifecycleCounters,
}

impl EffectExecutionControlStopped {
    pub(crate) fn new(
        lowered: &LoweredEffectExecutionPlan,
        kind: super::super::EffectExecutionControlStopKind,
        message: impl Into<String>,
    ) -> Self {
        let message = message.into();
        let plan_identity = lowered.lowered_effect_execution_plan_identity().clone();
        let outcome_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_execution_control_stopped_v1",
                )
                .field_evidence_identity(WorthQueryEvidenceTag::new("plan"), &plan_identity)
                .field_shape(WorthQueryEvidenceTag::new("kind"), format!("{kind:?}"))
                .seal();
        Self {
            kind,
            message,
            lowered_effect_execution_plan_identity: plan_identity,
            outcome_identity,
            counters: EffectLifecycleCounters::execution_denied(
                lowered.counters().effect_support_row_count(),
                lowered.counters().effect_lowering_width(),
                lowered.counters().effect_executor_rediscovery_count(),
            ),
        }
    }

    pub const fn kind(&self) -> super::super::EffectExecutionControlStopKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn lowered_effect_execution_plan_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.lowered_effect_execution_plan_identity
    }

    pub fn outcome_identity(
        &self,
    ) -> Result<&WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        self.outcome_identity.as_ref().map_err(|cause| *cause)
    }

    pub const fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }
}
