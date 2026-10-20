//! Internal topology validity for one declared capability contract.

use std::collections::BTreeSet;

use crate::application_capability::{
    ApplicationCapabilityElevationRule, ApplicationCapabilityFieldDimension,
    ApplicationCapabilityRelationDimension, ErasedApplicationCapabilityContract,
};

const MAXIMUM_DELEGATION_ACTIVATION_CONTEXT_RELATIONS: usize = 16;

pub(super) fn topology_is_valid(contract: &ErasedApplicationCapabilityContract) -> bool {
    let grant = contract.grant_entity();
    let target = contract.target();
    let constraints = contract.constraints();
    let currentness = constraints.currentness();
    let delegation = contract.delegation();
    target.action().field().entity() == grant
        && target.purpose().field().entity() == grant
        && target.resource().from() == grant
        && currentness.active_status().field().entity() == grant
        && currentness.workflow().grant().entity() == grant
        && currentness.workflow().resource().entity() == target.resource_path().scope_entity()
        && currentness.workflow().grant().value_type()
            == currentness.workflow().resource().value_type()
        && currentness.validity().not_before().entity() == grant
        && currentness.validity().not_after().entity() == grant
        && currentness.validity().not_before().value_type()
            == currentness.validity().not_after().value_type()
        && currentness.validity().not_before().scalar_family()
            == currentness.validity().timeline().scalar_family()
        && currentness.validity().not_after().scalar_family()
            == currentness.validity().timeline().scalar_family()
        && delegation.limit().entity() == grant
        && delegation.parent().from() == grant
        && delegation.parent().to() == grant
        && delegation.grantor().to() == grant
        && delegation.grantee().to() == grant
        && delegation_activation_topology_is_valid(contract)
        && capability_revocation_topology_is_valid(contract)
        && field_dimension_belongs_to(target.field(), grant)
        && field_dimension_belongs_to(constraints.magnitude(), grant)
        && elevation_topology_is_valid(contract)
}

fn capability_revocation_topology_is_valid(contract: &ErasedApplicationCapabilityContract) -> bool {
    let Some(revocation) = contract.delegation().revocation() else {
        return true;
    };
    let active = contract.constraints().currentness().active_status();
    revocation.identity().entity() == contract.grant_entity()
        && revocation.revoked_status().field() == active.field()
        && revocation.revoked_status().value() != active.value()
}

fn delegation_activation_topology_is_valid(contract: &ErasedApplicationCapabilityContract) -> bool {
    let Some(activation) = contract.delegation().activation() else {
        return true;
    };
    let context_relations = activation.context_relations();
    activation.identity().entity() == contract.grant_entity()
        && context_relations.len() <= MAXIMUM_DELEGATION_ACTIVATION_CONTEXT_RELATIONS
        && context_relations.iter().all(|relation| {
            relation.from() == contract.grant_entity()
                && !activation_context_overlaps_governed_relation(contract, relation)
        })
}

fn activation_context_overlaps_governed_relation(
    contract: &ErasedApplicationCapabilityContract,
    candidate: &crate::application_capability::ApplicationCapabilityRelationBinding,
) -> bool {
    let target = contract.target();
    let delegation = contract.delegation();
    candidate == target.resource()
        || matches!(
            target.relation(),
            ApplicationCapabilityRelationDimension::Bound(relation) if candidate == relation
        )
        || [
            delegation.parent(),
            delegation.grantor(),
            delegation.grantee(),
        ]
        .contains(&candidate)
}

fn elevation_topology_is_valid(contract: &ErasedApplicationCapabilityContract) -> bool {
    let ApplicationCapabilityElevationRule::Governed(elevation) = contract.elevation() else {
        return true;
    };
    let Some(principal) = capability_principal(contract) else {
        return false;
    };
    let elevation_entity = elevation.identity().entity();
    let review = elevation.review();
    let review_entity = review.identity().entity();
    let lifecycle = elevation.lifecycle();
    elevation.reason().entity() == elevation_entity
        && elevation.status().entity() == elevation_entity
        && elevation.validity().not_before().entity() == elevation_entity
        && elevation.validity().not_after().entity() == elevation_entity
        && elevation.validity().not_before().value_type()
            == elevation.validity().not_after().value_type()
        && elevation.validity().not_before().scalar_family()
            == elevation.validity().timeline().scalar_family()
        && elevation.validity().not_after().scalar_family()
            == elevation.validity().timeline().scalar_family()
        && elevation_duration_is_valid(elevation)
        && elevation
            .states()
            .values()
            .into_iter()
            .all(|state| state.field() == elevation.status())
        && elevation.requester().from() == principal
        && elevation.requester().to() == elevation_entity
        && elevation.approver().from() == principal
        && elevation.approver().to() == elevation_entity
        && elevation.grant().from() == elevation_entity
        && elevation.grant().to() == contract.grant_entity()
        && elevation.resource_relation().is_none_or(|relation| {
            relation.from() == elevation_entity
                && relation.to() == contract.target().resource().to()
        })
        && review.relation().from() == elevation_entity
        && review.relation().to() == review_entity
        && review_entity != elevation_entity
        && review.kind().field().entity() == review_entity
        && review.scope().from() == review_entity
        && review.scope().to() == contract.target().resource().to()
        && review.status().entity() == review_entity
        && review.reviewer().from() == principal
        && review.reviewer().to() == review_entity
        && review.required().field() == review.status()
        && review.completed().field() == review.status()
        && distinct_elevation_states(elevation)
        && review.required().value() != review.completed().value()
        && lifecycle.elevation_slot().context() == contract.constraints().context()
        && lifecycle.elevation_slot().context_identity().as_str()
            == contract.constraints().context_type()
        && lifecycle.elevation_slot().entity() == elevation_entity
        && lifecycle.review_slot().context() == contract.constraints().context()
        && lifecycle.review_slot().context_identity().as_str()
            == contract.constraints().context_type()
        && lifecycle.review_slot().entity() == review_entity
        && distinct_lifecycle_operations(contract)
}

fn distinct_elevation_states(
    elevation: &crate::application_capability::ApplicationCapabilityElevationDefinition,
) -> bool {
    elevation
        .states()
        .values()
        .into_iter()
        .map(|state| state.value())
        .collect::<BTreeSet<_>>()
        .len()
        == 4
}

fn elevation_duration_is_valid(
    elevation: &crate::application_capability::ApplicationCapabilityElevationDefinition,
) -> bool {
    let duration = elevation.maximum_duration();
    if duration.is_zero() {
        return false;
    }
    match elevation.validity().timeline() {
        crate::application_capability::ApplicationCapabilityValidityTimeline::UnixEpochSeconds => {
            duration.subsec_nanos() == 0
        }
        crate::application_capability::ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds => {
            duration.subsec_nanos().is_multiple_of(1_000_000)
        }
    }
}

fn distinct_lifecycle_operations(contract: &ErasedApplicationCapabilityContract) -> bool {
    let ApplicationCapabilityElevationRule::Governed(elevation) = contract.elevation() else {
        return true;
    };
    let transitions = elevation.lifecycle().transitions();
    transitions
        .iter()
        .map(|transition| {
            let operation = transition.operation();
            (operation.operation(), operation.input_type())
        })
        .collect::<BTreeSet<_>>()
        .len()
        == transitions.len()
        && transitions.into_iter().all(|transition| {
            let operation = transition.operation();
            operation.operation() != contract.operation()
                || operation.input_type() != contract.input_type()
        })
}

fn capability_principal(contract: &ErasedApplicationCapabilityContract) -> Option<&str> {
    contract
        .composition()
        .decision()
        .allow()
        .graph()
        .requirements()
        .first()
        .and_then(|requirement| requirement.clauses().first())
        .map(|clause| clause.path().principal_entity())
}

fn field_dimension_belongs_to(
    dimension: &ApplicationCapabilityFieldDimension,
    grant_entity: &str,
) -> bool {
    match dimension {
        ApplicationCapabilityFieldDimension::NotApplicable => true,
        ApplicationCapabilityFieldDimension::Bound(field) => field.entity() == grant_entity,
    }
}
