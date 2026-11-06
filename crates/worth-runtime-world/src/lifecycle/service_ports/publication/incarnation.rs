//! Prepared publication retains its issuing owner across every receiver route.
use super::*;
pub(super) fn publication_owner_denial(
    current: crate::identity::RuntimeWorldOwnerIdentity,
    expected: &ProductBranchObservation,
) -> Option<NoEffectCompositePublication> {
    let cause = match current.admit_incarnation(expected.owner_identity()) {
        Err(worth_proof::HomeIncarnationDenial::StaleIncarnation) => {
            NoEffectCause::StaleIncarnation
        }
        Err(worth_proof::HomeIncarnationDenial::ForeignHome) => {
            NoEffectCause::OwnerDeniedBeforeEffect
        }
        Ok(()) => return None,
    };
    Some(NoEffectCompositePublication::new(
        cause,
        Some(expected.clone()),
    ))
}
