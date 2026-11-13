//! Supplier emission permission is distinct from realized payload and recovery authority.
use super::{Admission, Denial, Use};
use worth_query_installation::facade::{
    InstalledAftermathRecoveryContract, InstalledCorrectionAuthority, InstalledCorrectionMechanism,
    InstalledExternalEffectContract, PublishedAftermathPosture,
    WorthQueryInstalledAftermathContract,
};

pub(super) fn classify(
    aftermath: &WorthQueryInstalledAftermathContract,
    usage: Use<'_>,
) -> Result<Admission, Denial> {
    let contracts = match usage {
        Use::OriginalExecution(contracts) => contracts,
        Use::CorrectionPreparation => return Err(Denial::UnsupportedCorrectionCompensation),
    };
    if contracts.body().is_some()
        || contracts.aftermath() != Some(aftermath)
        || aftermath.correction_disclosure().is_some()
        || aftermath.authority() != InstalledCorrectionAuthority::RuntimeWithExternalOwner
        || aftermath.published_posture() != PublishedAftermathPosture::Reconcilable
        || aftermath.recovery()
            != (InstalledAftermathRecoveryContract::Admissible {
                posture: PublishedAftermathPosture::Reconcilable,
            })
    {
        return Err(Denial::UnsupportedCorrectionCompensation);
    }
    let Some(InstalledCorrectionMechanism::Compensation(compensation)) = aftermath.mechanism()
    else {
        return Err(Denial::UnsupportedCorrectionCompensation);
    };
    let Some(reconciliation) = aftermath.reconciliation() else {
        return Err(Denial::UnsupportedCorrectionCompensation);
    };
    if compensation.compensating_operation_slot().is_empty()
        || reconciliation.procedure_slot().is_empty()
    {
        return Err(Denial::UnsupportedCorrectionCompensation);
    }
    let InstalledExternalEffectContract::Declared {
        effect,
        correlation_family,
        maximum_payload_bytes,
        inbound,
        ..
    } = contracts.external_effect()
    else {
        return Err(Denial::UnsupportedCorrectionCompensation);
    };
    if *maximum_payload_bytes == 0
        || aftermath.external_effect().correlation_family() != Some(correlation_family)
        || !contracts
            .emissions()
            .emissions()
            .iter()
            .any(|emission| emission.effect() == effect)
        || !inbound.as_ref().is_some_and(|inbound| {
            inbound.effect() == effect && !inbound.source_identity().is_empty()
        })
    {
        return Err(Denial::UnsupportedCorrectionCompensation);
    }
    // The installed contract owns payload/protocol/bounds; the emission ceiling
    // only says MAY emit. Existing typed candidate/outbox gates require the real payload.
    Ok(Admission::ExternalOriginalRecoveryOnly)
}

/// Native scalar-output compensation is distinct from external original recovery.
pub(super) fn classify_native(
    aftermath: &WorthQueryInstalledAftermathContract,
    binding: &worth_query_installation::facade::InstalledCorrectionBinding,
    usage: Use<'_>,
) -> Result<Admission, Denial> {
    let declaration = binding.declaration();
    if let Use::OriginalExecution(contracts) = usage {
        if contracts.body().is_some()
            || contracts.aftermath() != Some(aftermath)
            || !matches!(
                contracts.external_effect(),
                InstalledExternalEffectContract::None
            )
        {
            return Err(Denial::UnsupportedCorrectionCompensation);
        }
    }
    if aftermath.authority() != InstalledCorrectionAuthority::RuntimeAlone
        || aftermath.published_posture() != PublishedAftermathPosture::Compensatable
        || aftermath.recovery()
            != (InstalledAftermathRecoveryContract::Admissible {
                posture: PublishedAftermathPosture::Compensatable,
            })
        || aftermath.correction_disclosure().is_none()
        || aftermath.reconciliation().is_some()
        || !matches!(
            aftermath.external_effect(),
            worth_query_installation::facade::InstalledExternalEffectPosture::None
        )
    {
        return Err(Denial::UnsupportedCorrectionCompensation);
    }
    let Some(InstalledCorrectionMechanism::Compensation(mechanism)) = aftermath.mechanism() else {
        return Err(Denial::UnsupportedCorrectionCompensation);
    };
    if mechanism.compensating_operation_slot() != declaration.inverse().operation() {
        return Err(Denial::UnsupportedCorrectionCompensation);
    }
    if !declaration.coupled_outputs().is_empty() {
        return Err(Denial::UnsupportedCorrectionCoupledOutputs);
    }
    if declaration.authored_identity().is_some()
        || declaration.source_output_fields().is_empty()
        || declaration.source_output_fields().iter().any(|field| {
            let field = field.parts();
            &field.source_binding != declaration.source().binding_identity()
                || field.content_format.is_some()
        })
    {
        return Err(Denial::UnsupportedCorrectionOutputMaterial);
    }
    Ok(Admission::NativeRecordedCorrection)
}
