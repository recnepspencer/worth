//! Request stops carried by World's no-effect publication outcomes.
use super::WorthQueryAdvancementDenial;
use worth_runtime_world::facade::{NoEffectCause, RuntimeWorldSettledRelationalAdoptionDenial};

pub(super) fn publication(cause: NoEffectCause) -> Option<WorthQueryAdvancementDenial> {
    use super::WorthQueryManagedComputationInterruption as Interruption;
    match cause {
        NoEffectCause::CancelledBeforeEffect => Some(WorthQueryAdvancementDenial::Interrupted(
            Interruption::Cancelled,
        )),
        NoEffectCause::DeadlineBeforeEffect => Some(WorthQueryAdvancementDenial::Interrupted(
            Interruption::DeadlineExceeded,
        )),
        NoEffectCause::ExecutionRequest(cause) => {
            Some(super::application_contribution::advancement_denial(cause))
        }
        NoEffectCause::StaleExpectedProductHead
        | NoEffectCause::OwnerDeniedBeforeEffect
        | NoEffectCause::CorrespondenceRebindRequired
        | NoEffectCause::ReferenceGenerationExhausted
        | NoEffectCause::CapacityExhausted
        | NoEffectCause::OwnerUnavailable
        | NoEffectCause::RelationalDeferred(_)
        | NoEffectCause::PreEffectFailure => None,
    }
}

pub(super) fn adoption(
    denial: &RuntimeWorldSettledRelationalAdoptionDenial,
) -> Option<WorthQueryAdvancementDenial> {
    match denial {
        RuntimeWorldSettledRelationalAdoptionDenial::Publication(no_effect) => {
            publication(no_effect.cause())
        }
        RuntimeWorldSettledRelationalAdoptionDenial::Recovery(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::WorthQueryManagedComputationInterruption as Stop;
    #[test]
    fn publication_stop_classification_preserves_control_and_ordinary_causes() {
        for (cause, expected) in [
            (NoEffectCause::CancelledBeforeEffect, Stop::Cancelled),
            (NoEffectCause::DeadlineBeforeEffect, Stop::DeadlineExceeded),
        ] {
            assert_eq!(
                publication(cause),
                Some(WorthQueryAdvancementDenial::Interrupted(expected))
            );
        }
        for cause in [
            NoEffectCause::StaleExpectedProductHead,
            NoEffectCause::OwnerDeniedBeforeEffect,
            NoEffectCause::CorrespondenceRebindRequired,
            NoEffectCause::ReferenceGenerationExhausted,
            NoEffectCause::CapacityExhausted,
            NoEffectCause::OwnerUnavailable,
            NoEffectCause::PreEffectFailure,
        ] {
            assert_eq!(publication(cause), None);
        }
    }
}
