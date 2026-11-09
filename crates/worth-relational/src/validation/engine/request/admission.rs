use crate::validation::data::{InvariantCostClass, InvariantFailureEffect};

use super::super::policy::{cost_allowed, RelationalInvariantRuntime};

/// The admission decision for one catalog registration.
///
/// Blocking registrations are required at the checkpoint that owns their
/// enforcement effect.  A planning or audit checkpoint may still omit a
/// costly registration because it cannot lawfully publish or commit from that
/// checkpoint.  Only the enforcement checkpoint can make the registration
/// non-optional regardless of its declared scan cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InvariantRegistrationAdmission {
    Required,
    Optional,
    ExcludedByCost,
}

impl InvariantRegistrationAdmission {
    pub(crate) const fn is_admitted(self) -> bool {
        !matches!(self, Self::ExcludedByCost)
    }
}

pub(crate) fn for_failure_effect(
    runtime_policy: &RelationalInvariantRuntime,
    checkpoint: crate::validation::data::InvariantExecutionPoint,
    failure_effect: InvariantFailureEffect,
    cost: InvariantCostClass,
) -> InvariantRegistrationAdmission {
    let enforcement_checkpoint = matches!(
        (checkpoint, failure_effect),
        (
            crate::validation::data::InvariantExecutionPoint::ProposedState
                | crate::validation::data::InvariantExecutionPoint::CommitBoundary
                | crate::validation::data::InvariantExecutionPoint::MutationSensitive,
            InvariantFailureEffect::BlockCommit,
        ) | (
            crate::validation::data::InvariantExecutionPoint::SnapshotPublication
                | crate::validation::data::InvariantExecutionPoint::ProposedState,
            InvariantFailureEffect::BlockPublication,
        )
    );
    if enforcement_checkpoint {
        InvariantRegistrationAdmission::Required
    } else if cost_allowed(runtime_policy.max_cost_at(checkpoint), cost) {
        InvariantRegistrationAdmission::Optional
    } else {
        InvariantRegistrationAdmission::ExcludedByCost
    }
}

#[cfg(test)]
mod tests {
    use super::{for_failure_effect, InvariantRegistrationAdmission};
    use crate::validation::data::{
        InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect,
    };
    use crate::validation::engine::policy::{
        InvariantContext, InvariantScale, RelationalInvariantRuntime,
    };

    #[test]
    fn blocking_global_registration_is_required_above_optional_cost_ceiling() {
        let policy = RelationalInvariantRuntime::resolve(
            super::super::super::profile::InvariantRequestProfile::GraphComposition,
            InvariantContext {
                scale: InvariantScale::Large,
                version_depth: 2_000,
                snapshot_pressure: true,
            },
        );
        assert_eq!(
            policy.max_cost_at(InvariantExecutionPoint::GraphComposition),
            InvariantCostClass::Touched
        );
        assert_eq!(
            for_failure_effect(
                &policy,
                InvariantExecutionPoint::CommitBoundary,
                InvariantFailureEffect::BlockCommit,
                InvariantCostClass::Global,
            ),
            InvariantRegistrationAdmission::Required
        );
        assert_eq!(
            for_failure_effect(
                &policy,
                InvariantExecutionPoint::GraphComposition,
                InvariantFailureEffect::AuditOnly,
                InvariantCostClass::Global,
            ),
            InvariantRegistrationAdmission::ExcludedByCost
        );
    }

    #[test]
    fn graph_composition_blocking_registration_remains_cost_filtered() {
        let policy = RelationalInvariantRuntime::resolve(
            super::super::super::profile::InvariantRequestProfile::GraphComposition,
            InvariantContext {
                scale: InvariantScale::Large,
                version_depth: 2_000,
                snapshot_pressure: true,
            },
        );

        assert_eq!(
            for_failure_effect(
                &policy,
                InvariantExecutionPoint::GraphComposition,
                InvariantFailureEffect::BlockCommit,
                InvariantCostClass::Global,
            ),
            InvariantRegistrationAdmission::ExcludedByCost
        );
    }
}
