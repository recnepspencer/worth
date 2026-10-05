use std::mem::size_of;

use worth_query_admission::facade::graph_read_access::{
    WorthQueryGraphIndexInventory, WorthQueryGraphIndexInventoryAdmissionStop,
    WorthQueryGraphIndexSupportRow, WorthQueryGraphReadAccessRequirementKind,
    WorthQueryGraphReadAccessRequirementRow, WorthQueryGraphReadAccessRequirementSet,
    WorthQueryGraphReadOrderingPosture,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationContinuationContract, WorthQueryInstalledApplicationLiveContract,
};

use crate::domain_computation::primary_graph::schema_layout::{
    WorthQueryPrimaryGraphLayout, WorthQuerySupportLookupStop,
};

type Stop<Admission> = WorthQueryGraphIndexInventoryAdmissionStop<Admission>;

fn lookup_stop<Admission>(stop: WorthQuerySupportLookupStop<Admission>) -> Stop<Admission> {
    match stop {
        WorthQuerySupportLookupStop::Admission(stop) => Stop::Admission(stop),
        WorthQuerySupportLookupStop::AccountingOverflow => Stop::AccountingOverflow,
    }
}

fn runtime_supports_admitted<Admission>(
    layout: &WorthQueryPrimaryGraphLayout,
    continuation: Option<&WorthQueryInstalledApplicationContinuationContract>,
    live: Option<&WorthQueryInstalledApplicationLiveContract>,
    requirement: &WorthQueryGraphReadAccessRequirementRow,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Admission>,
) -> Result<bool, Stop<Admission>> {
    admit(1, 0).map_err(Stop::Admission)?;
    match requirement.kind() {
        WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency
        | WorthQueryGraphReadAccessRequirementKind::ReverseAdjacency => {
            let Some(relation) = requirement.relation_name() else {
                return Ok(false);
            };
            layout
                .supports_relation_admitted(relation, admit)
                .map_err(lookup_stop)
        }
        WorthQueryGraphReadAccessRequirementKind::PredicateSupport => {
            for field in requirement.predicate_field_authorities() {
                admit(1, 0).map_err(Stop::Admission)?;
                if !layout
                    .supports_equality_field_admitted(
                        field.native_aspect_key(),
                        field.native_field_key(),
                        admit,
                    )
                    .map_err(lookup_stop)?
                {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        WorthQueryGraphReadAccessRequirementKind::OrderingSupport => {
            match requirement.ordering_posture() {
                Some(WorthQueryGraphReadOrderingPosture::BoundedProjectedCollection) => {
                    for field in requirement.ordering_field_authorities() {
                        admit(1, 0).map_err(Stop::Admission)?;
                        if !layout
                            .supports_projection_field_admitted(
                                field.native_aspect_key(),
                                field.native_field_key(),
                                admit,
                            )
                            .map_err(lookup_stop)?
                        {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                Some(WorthQueryGraphReadOrderingPosture::IndexedRelatedCollectionSeek) => {
                    let Some(contract) = continuation else {
                        return Ok(false);
                    };
                    layout
                        .supports_continuation_ordering_admitted(contract, admit)
                        .map_err(lookup_stop)
                }
                _ => Ok(false),
            }
        }
        WorthQueryGraphReadAccessRequirementKind::LiveMaintenanceSupport => {
            let Some(contract) = live else {
                return Ok(false);
            };
            if !layout
                .supports_equality_field_admitted(
                    contract.scope_identity().aspect_key(),
                    contract.scope_identity().field_key(),
                    admit,
                )
                .map_err(lookup_stop)?
            {
                return Ok(false);
            }
            layout
                .supports_equality_field_admitted(
                    contract.target_identity().aspect_key(),
                    contract.target_identity().field_key(),
                    admit,
                )
                .map_err(lookup_stop)
        }
        WorthQueryGraphReadAccessRequirementKind::DomainOperationCapabilityRegistration => {
            Ok(false)
        }
        WorthQueryGraphReadAccessRequirementKind::TraversalWorkset
        | WorthQueryGraphReadAccessRequirementKind::VisitedSet
        | WorthQueryGraphReadAccessRequirementKind::DedupSet
        | WorthQueryGraphReadAccessRequirementKind::ProofSupport
        | WorthQueryGraphReadAccessRequirementKind::ResultBuffer
        | WorthQueryGraphReadAccessRequirementKind::MaterializationLifecycle => Ok(true),
    }
}

pub(in crate::domain_computation::primary_graph) fn primary_graph_support_inventory_admitted<
    Admission,
>(
    layout: &WorthQueryPrimaryGraphLayout,
    continuation: Option<&WorthQueryInstalledApplicationContinuationContract>,
    live: Option<&WorthQueryInstalledApplicationLiveContract>,
    requirements: &WorthQueryGraphReadAccessRequirementSet,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Admission>,
) -> Result<WorthQueryGraphIndexInventory, Stop<Admission>> {
    admit(1, 0).map_err(Stop::Admission)?;
    let all = requirements.rows();
    let reference_bytes = all
        .len()
        .checked_mul(size_of::<&WorthQueryGraphReadAccessRequirementRow>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Stop::AccountingOverflow)?;
    admit(reference_bytes, reference_bytes).map_err(Stop::Admission)?;
    let mut supported = Vec::new();
    supported
        .try_reserve_exact(all.len())
        .map_err(|_| Stop::AllocationUnavailable)?;
    for requirement in all {
        admit(1, 0).map_err(Stop::Admission)?;
        if runtime_supports_admitted(layout, continuation, live, requirement, admit)? {
            supported.push(requirement);
        }
    }
    let row_bytes = supported
        .len()
        .checked_mul(size_of::<WorthQueryGraphIndexSupportRow>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Stop::AccountingOverflow)?;
    admit(row_bytes, row_bytes).map_err(Stop::Admission)?;
    let mut rows = Vec::new();
    rows.try_reserve_exact(supported.len())
        .map_err(|_| Stop::AllocationUnavailable)?;
    for requirement in supported {
        rows.push(
            WorthQueryGraphIndexSupportRow::for_supported_requirement_admitted(requirement, admit)?,
        );
    }
    WorthQueryGraphIndexInventory::from_rows_admitted(rows, admit)
}
