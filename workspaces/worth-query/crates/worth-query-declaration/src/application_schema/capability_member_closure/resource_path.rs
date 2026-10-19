//! Cold admission of a resource extent; a supplied parent identity is not authority.
use super::{DeclaredCapabilityDimensions, ErasedApplicationCapabilityContract};
use crate::application_capability::{
    ApplicationCapabilityDelegationRule, ApplicationCapabilityElevationRule,
};
use crate::application_schema::{
    ApplicationAuthorizationTraversalDirection as Direction, ApplicationSchemaMember,
};

pub(super) fn is_closed(
    members: &[ApplicationSchemaMember],
    dimensions: &DeclaredCapabilityDimensions<'_>,
    contract: &ErasedApplicationCapabilityContract,
) -> bool {
    let path = contract.target().resource_path();
    if path.is_direct() {
        return true;
    }
    let Some(slot) = path.root_slot() else {
        return false;
    };
    if path.traversals().is_empty()
        || slot.entity() != path.head().to()
        || slot.context() != contract.constraints().context()
        || slot.context_identity().as_str() != contract.constraints().context_type()
        || !dimensions.entity_slot_exists(slot)
        || !matches!(
            contract.composition().propagation().delegation(),
            ApplicationCapabilityDelegationRule::Forbidden
        )
        || !matches!(
            contract.elevation(),
            ApplicationCapabilityElevationRule::NotApplicable
        )
        || contract.delegation().activation().is_some()
        || contract.delegation().revocation().is_some()
    {
        return false;
    }
    let mut current = path.head().to();
    for step in path.traversals() {
        let (from, to) = match step.direction() {
            Direction::Forward => (step.from(), step.to()),
            Direction::Reverse => (step.to(), step.from()),
        };
        if current != from
            || !members.iter().any(|member| {
                matches!(member,
                    ApplicationSchemaMember::Relation { relation, from, to, .. }
                        if relation == step.relation() && from == step.from() && to == step.to()
                )
            })
        {
            return false;
        }
        current = to;
    }
    current == path.scope_entity()
}
