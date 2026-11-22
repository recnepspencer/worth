use worth_query_installation::facade::{
    InstalledAftermathPostcondition, InstalledCorrectionMechanism, PublishedAftermathPosture,
    WorthQueryInstalledAftermathContract,
};

pub fn reversal_posture(contract: Option<&WorthQueryInstalledAftermathContract>) -> String {
    let Some(contract) = contract else {
        return "none".to_owned();
    };
    match (contract.published_posture(), contract.mechanism()) {
        (PublishedAftermathPosture::Irreversible, _) => "irreversible".to_owned(),
        (
            PublishedAftermathPosture::Reversible,
            Some(InstalledCorrectionMechanism::RecordedInverse(inverse)),
        ) => format!(
            "exact-inverse:{}:{}:{}",
            inverse.inverse_operation_slot(),
            inverse.lowering_correspondence().correspondence_slot(),
            aftermath_postcondition(inverse.postcondition())
        ),
        (
            PublishedAftermathPosture::Compensatable,
            Some(InstalledCorrectionMechanism::Compensation(compensation)),
        ) => format!(
            "compensation:{}:{}",
            compensation.compensating_operation_slot(),
            aftermath_postcondition(compensation.postcondition())
        ),
        (PublishedAftermathPosture::Reconcilable, mechanism) => match mechanism {
            Some(InstalledCorrectionMechanism::RecordedInverse(inverse)) => format!(
                "reconcilable-exact-inverse:{}:{}",
                inverse.inverse_operation_slot(),
                aftermath_postcondition(inverse.postcondition())
            ),
            Some(InstalledCorrectionMechanism::Compensation(compensation)) => format!(
                "reconcilable-compensation:{}:{}",
                compensation.compensating_operation_slot(),
                aftermath_postcondition(compensation.postcondition())
            ),
            None => "reconcilable".to_owned(),
        },
        _ => "declaration-incomplete".to_owned(),
    }
}

fn aftermath_postcondition(postcondition: &InstalledAftermathPostcondition) -> String {
    match postcondition {
        InstalledAftermathPostcondition::ExactPriorTruth => "exact-prior-truth".to_owned(),
        InstalledAftermathPostcondition::InvariantRestored { invariant } => {
            format!("invariant-restored:{invariant}")
        }
        InstalledAftermathPostcondition::BusinessPostcondition { identity } => {
            format!("business-postcondition:{identity}")
        }
    }
}
