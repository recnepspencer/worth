use crate::runtime::{UiAllocationInvalidationFamily, UiAllocationInvalidationTarget};

pub(super) fn narrow_host_measurement(
    family: UiAllocationInvalidationFamily,
    measurement: &crate::host::UiAdmittedHostMeasurement,
    ordinal: u16,
    authority: &super::UiAllocationInvalidationAuthority,
    target_ceiling: u16,
    counters: &mut super::UiAllocationInvalidationNarrowingCounters,
) -> Result<UiAllocationInvalidationTarget, super::UiAllocationInvalidationNarrowingDenial> {
    counted(counters.lookup_graph_target(), ordinal)?;
    if family == UiAllocationInvalidationFamily::PortalAnchorMovement {
        let movement = authority
            .portal_movement(measurement.result())
            .map_err(|denial| match denial {
                super::portal_binding_index::UiPortalMovementLookupDenial::NormalizationAuthorityMismatch =>
                    super::UiAllocationInvalidationNarrowingDenial::HostNormalizationAuthorityMismatch { ordinal },
                super::portal_binding_index::UiPortalMovementLookupDenial::SuccessorBasis(reason) =>
                    match reason {
                        crate::runtime::UiPortalAnchorSuccessorDenial::StaleEvidenceGeneration =>
                            super::UiAllocationInvalidationNarrowingDenial::PortalAnchorEvidenceStale { ordinal },
                        crate::runtime::UiPortalAnchorSuccessorDenial::EvidenceCategoryMismatch
                        | crate::runtime::UiPortalAnchorSuccessorDenial::NormalizationAuthorityMismatch
                        | crate::runtime::UiPortalAnchorSuccessorDenial::ObservationInvalid =>
                            super::UiAllocationInvalidationNarrowingDenial::PortalAnchorSuccessorBasisDenied { ordinal },
                    },
            })?
            .ok_or(super::UiAllocationInvalidationNarrowingDenial::PortalAnchorNotAdmitted { ordinal })?;
        counted(
            counters.record_authority_probes(movement.authority_probes()),
            ordinal,
        )?;
        return Ok(UiAllocationInvalidationTarget::PortalAnchor {
            movement: Box::new(movement),
        });
    }
    if matches!(
        family,
        UiAllocationInvalidationFamily::ScrollExtentObservation
            | UiAllocationInvalidationFamily::ScrollOwnedExtentChange
    ) {
        let lookup = authority
            .scroll_target(measurement.result().authority_witness())
            .map_err(|denial| super::denial::map_host_lookup_denial(denial, ordinal))?;
        counted(counters.record_authority_probes(lookup.probes()), ordinal)?;
        if lookup.is_empty() {
            return Err(
                super::UiAllocationInvalidationNarrowingDenial::ScrollOwnershipNotAdmitted {
                    ordinal,
                },
            );
        }
        return Ok(UiAllocationInvalidationTarget::ScrollOwnedExtent {
            evidence_generation: measurement.result().evidence_generation(),
            bindings: lookup.materialize_bindings(),
        });
    }
    let lookup = authority.host_target(measurement.result().authority_witness());
    let lookup = match lookup {
        Ok(lookup) => lookup,
        Err(denial) => {
            counted(counters.record_authority_probes(1), ordinal)?;
            return Err(super::denial::map_host_lookup_denial(denial, ordinal));
        }
    };
    counted(counters.record_authority_probes(lookup.probes), ordinal)?;
    if family == UiAllocationInvalidationFamily::ViewportExtentChange {
        enforce_viewport_budget(lookup.target_count(), ordinal, target_ceiling, counters)?;
    }
    let target = lookup.materialize_target().ok_or(
        super::UiAllocationInvalidationNarrowingDenial::HostMeasurementTargetNotAdmitted {
            ordinal,
        },
    )?;
    counted(counters.materialize_host_target_set(), ordinal)?;
    Ok(UiAllocationInvalidationTarget::HostMeasurement {
        measurement: measurement.result().clone(),
        target,
    })
}

fn enforce_viewport_budget(
    target_count: usize,
    ordinal: u16,
    maximum: u16,
    counters: &super::UiAllocationInvalidationNarrowingCounters,
) -> Result<(), super::UiAllocationInvalidationNarrowingDenial> {
    let attempted = usize::from(counters.emitted_targets())
        .checked_add(target_count)
        .and_then(|value| u16::try_from(value).ok())
        .ok_or(
            super::UiAllocationInvalidationNarrowingDenial::AuthorityCounterExhausted { ordinal },
        )?;
    if attempted > maximum {
        return Err(
            super::UiAllocationInvalidationNarrowingDenial::ViewportTargetBudgetExceeded {
                ordinal,
                attempted,
                maximum,
            },
        );
    }
    Ok(())
}

fn counted(
    result: Result<(), ()>,
    ordinal: u16,
) -> Result<(), super::UiAllocationInvalidationNarrowingDenial> {
    result.map_err(
        |()| super::UiAllocationInvalidationNarrowingDenial::AuthorityCounterExhausted { ordinal },
    )
}
