use worth_query_admission::facade::graph_read_access::{
    WorthQueryGraphIndexInventory, WorthQueryGraphIndexSupportRow,
    WorthQueryGraphReadAccessRequirementKind, WorthQueryGraphReadAccessRequirementRow,
    WorthQueryGraphReadAccessRequirementSet, WorthQueryGraphReadOrderingPosture,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationContinuationContract, WorthQueryInstalledApplicationLiveContract,
};

use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;

fn runtime_supports(
    layout: &WorthQueryPrimaryGraphLayout,
    continuation: Option<&WorthQueryInstalledApplicationContinuationContract>,
    live: Option<&WorthQueryInstalledApplicationLiveContract>,
    requirement: &WorthQueryGraphReadAccessRequirementRow,
) -> bool {
    match requirement.kind() {
        WorthQueryGraphReadAccessRequirementKind::AggregateSelection => requirement
            .relation_name()
            .and_then(|name| layout.relation(name))
            .is_some_and(|relation| relation.orbit && relation.from == relation.to),
        WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency
        | WorthQueryGraphReadAccessRequirementKind::ReverseAdjacency => {
            let Some(relation) = requirement.relation_name() else {
                return false;
            };
            layout.supports_relation(relation)
        }
        WorthQueryGraphReadAccessRequirementKind::PredicateSupport => {
            for field in requirement.predicate_field_authorities() {
                if !layout
                    .supports_equality_field(field.native_aspect_key(), field.native_field_key())
                {
                    return false;
                }
            }
            true
        }
        WorthQueryGraphReadAccessRequirementKind::OrderingSupport => {
            match requirement.ordering_posture() {
                Some(WorthQueryGraphReadOrderingPosture::BoundedProjectedCollection) => {
                    for field in requirement.ordering_field_authorities() {
                        if !layout.supports_projection_field(
                            field.native_aspect_key(),
                            field.native_field_key(),
                        ) {
                            return false;
                        }
                    }
                    true
                }
                Some(WorthQueryGraphReadOrderingPosture::IndexedRelatedCollectionSeek) => {
                    let Some(contract) = continuation else {
                        return false;
                    };
                    layout.supports_continuation_ordering(contract)
                }
                _ => false,
            }
        }
        WorthQueryGraphReadAccessRequirementKind::LiveMaintenanceSupport => {
            let Some(contract) = live else {
                return false;
            };
            if !layout.supports_equality_field(
                contract.scope_identity().aspect_key(),
                contract.scope_identity().field_key(),
            ) {
                return false;
            }
            layout.supports_equality_field(
                contract.target_identity().aspect_key(),
                contract.target_identity().field_key(),
            )
        }
        WorthQueryGraphReadAccessRequirementKind::DomainOperationCapabilityRegistration => false,
        WorthQueryGraphReadAccessRequirementKind::TraversalWorkset
        | WorthQueryGraphReadAccessRequirementKind::VisitedSet
        | WorthQueryGraphReadAccessRequirementKind::DedupSet
        | WorthQueryGraphReadAccessRequirementKind::ProofSupport
        | WorthQueryGraphReadAccessRequirementKind::ResultBuffer
        | WorthQueryGraphReadAccessRequirementKind::MaterializationLifecycle => true,
    }
}

pub(in crate::domain_computation::primary_graph) fn primary_graph_support_inventory(
    layout: &WorthQueryPrimaryGraphLayout,
    continuation: Option<&WorthQueryInstalledApplicationContinuationContract>,
    live: Option<&WorthQueryInstalledApplicationLiveContract>,
    requirements: &WorthQueryGraphReadAccessRequirementSet,
) -> Result<WorthQueryGraphIndexInventory, std::collections::TryReserveError> {
    let mut rows = Vec::new();
    rows.try_reserve_exact(requirements.rows().len())?;
    for requirement in requirements.rows() {
        if runtime_supports(layout, continuation, live, requirement) {
            rows.push(WorthQueryGraphIndexSupportRow::for_supported_requirement(
                requirement,
            ));
        }
    }
    Ok(WorthQueryGraphIndexInventory::from_rows(rows))
}
