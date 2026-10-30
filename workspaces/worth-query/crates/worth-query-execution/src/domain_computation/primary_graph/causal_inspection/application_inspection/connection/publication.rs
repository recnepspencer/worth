//! Select the publication that actually created the disclosed output revision.
use super::*;
use crate::domain_computation::primary_graph::application_output_demand::OriginalSettledProducerObservation;
use worth_execution::MapKernelContext;
use worth_runtime_bridge::facade::BridgeConditionalCausalObservation;

pub(super) struct ConnectedOutputPublication {
    pub(super) role: usize,
    pub(super) changes: WorthQueryApplicationCommittedChanges,
    pub(super) inputs: FactPayload,
    pub(super) native: BridgeConditionalCausalObservation,
}

pub(super) fn select(
    observed: &overlap::ObservationConnection<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
    target: &WorthQueryApplicationInspectionTarget,
    source: &WorthQueryApplicationCommitReceipt,
    supplier: &OriginalSettledProducerObservation,
    source_diagnostic: Option<&DiagnosticComparisonAdmission<'_>>,
) -> Result<ConnectedOutputPublication, MapKernelFailure<ConnectionRefusal>> {
    let OriginalSettledProducerSource::Committed {
        changes,
        inputs: Some(inputs),
        conditional: Some(native),
        original_publication,
        ..
    } = supplier.source()
    else {
        return Err(MapKernelFailure::Domain(
            WorthQueryProducerConnectionDenial::ProducerUnavailable.into(),
        ));
    };
    let failure = match observed.output_role(context, changes, supplier.correspondence()) {
        Ok(role) => {
            return Ok(ConnectedOutputPublication {
                role,
                changes: changes.clone(),
                inputs: inputs.clone(),
                native: native.clone(),
            })
        }
        Err(failure) => failure,
    };
    if !matches!(
        &failure,
        MapKernelFailure::Domain(ConnectionRefusal::Output(
            WorthQueryProducerOutputConnectionDenial::NativeRevisionNotCreatedByProducer,
        ))
    ) {
        return Err(failure);
    }
    let Some(publication) = original_publication else {
        return Err(failure);
    };
    let Some(native) = publication.conditional.as_ref() else {
        return Err(failure);
    };
    // Current supplier admission remains mandatory. The creator must also
    // independently have consumed this exact performed source occurrence.
    match source::require_input(
        observed,
        context,
        target,
        source.committed_changes(),
        &publication.inputs,
        source_diagnostic,
    ) {
        Ok(()) => {}
        Err(MapKernelFailure::Domain(_)) => return Err(failure),
        Err(stop) => return Err(stop),
    }
    let role =
        match observed.output_role(context, &publication.changes, &publication.correspondence) {
            Ok(role) => role,
            Err(MapKernelFailure::Domain(_)) => return Err(failure),
            Err(stop) => return Err(stop),
        };
    if !preserves_output_role(&publication.correspondence, supplier.correspondence(), role) {
        return Err(failure);
    }
    Ok(ConnectedOutputPublication {
        role,
        changes: publication.changes.clone(),
        inputs: publication.inputs.clone(),
        native: native.clone(),
    })
}

fn preserves_output_role(
    original: &WorthQueryApplicationOutputCorrespondence,
    current: &WorthQueryApplicationOutputCorrespondence,
    role: usize,
) -> bool {
    use crate::domain_computation::primary_graph::WorthQueryApplicationOutputPosture;
    matches!(
        (original.native_witness_roles().nth(role), current.native_witness_roles().nth(role)),
        (Some(old), Some(now))
            if now.1 != WorthQueryApplicationOutputPosture::Retire
                && (old.0, old.2, old.3) == (now.0, now.2, now.3)
    )
}
