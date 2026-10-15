use crate::application_capability::{
    ApplicationCapabilityDisclosureRule, ApplicationCapabilityFieldBinding,
    ApplicationCapabilityFieldDimension, ApplicationCapabilityGraphRule,
    ApplicationCapabilityPathContextAnchor, ApplicationCapabilityScopeGuard,
    ErasedApplicationCapabilityContract,
};

use super::super::member_closure::ClosureIndex;
use super::super::{
    ApplicationAuthorizationPathEffect, ApplicationAuthorizationTraversalDirection,
};
use super::declared_dimensions::DeclaredCapabilityDimensions;

pub(super) fn composition_is_closed(
    closure: &ClosureIndex<'_>,
    dimensions: &DeclaredCapabilityDimensions<'_>,
    contract: &ErasedApplicationCapabilityContract,
) -> bool {
    let composition = contract.composition();
    if composition
        .actors()
        .recorded_actor()
        .requirement()
        .is_some_and(|requirement| !requirement.is_valid())
    {
        return false;
    }
    graph_rule_is_closed(
        closure,
        dimensions,
        contract,
        composition.decision().allow().graph(),
        ApplicationAuthorizationPathEffect::Allow,
    ) && optional_graph_rule_is_closed(
        closure,
        dimensions,
        contract,
        composition.decision().deny().graph(),
    ) && optional_graph_rule_is_closed(
        closure,
        dimensions,
        contract,
        composition.decision().conflict().graph(),
    ) && optional_graph_rule_is_closed(
        closure,
        dimensions,
        contract,
        composition.actors().separation_of_duty().graph(),
    ) && optional_graph_rule_is_closed(
        closure,
        dimensions,
        contract,
        composition.actors().distinct_actor().graph(),
    ) && disclosure_rule_is_closed(contract, composition.propagation().disclosure())
}

fn optional_graph_rule_is_closed(
    closure: &ClosureIndex<'_>,
    dimensions: &DeclaredCapabilityDimensions<'_>,
    contract: &ErasedApplicationCapabilityContract,
    rule: Option<&ApplicationCapabilityGraphRule>,
) -> bool {
    rule.is_none_or(|rule| {
        graph_rule_is_closed(
            closure,
            dimensions,
            contract,
            rule,
            ApplicationAuthorizationPathEffect::Deny,
        )
    })
}

fn graph_rule_is_closed(
    closure: &ClosureIndex<'_>,
    dimensions: &DeclaredCapabilityDimensions<'_>,
    contract: &ErasedApplicationCapabilityContract,
    rule: &ApplicationCapabilityGraphRule,
    expected_effect: ApplicationAuthorizationPathEffect,
) -> bool {
    !rule.requirements().is_empty()
        && rule.requirements().iter().all(|requirement| {
            !requirement.clauses().is_empty()
                && requirement.clauses().iter().all(|clause| {
                    clause.path().effect() == expected_effect
                        && closure.authorization_path_is_closed(
                            contract.target().resource().to(),
                            clause.path(),
                        )
                        && guard_is_owned(contract, clause.guard())
                        && clause.context_anchors().iter().all(|anchor| {
                            anchor_is_closed(dimensions, contract, clause.path(), anchor)
                        })
                })
        })
}

fn anchor_is_closed(
    dimensions: &DeclaredCapabilityDimensions<'_>,
    contract: &ErasedApplicationCapabilityContract,
    path: &super::super::ApplicationAuthorizationPath,
    anchor: &ApplicationCapabilityPathContextAnchor,
) -> bool {
    let slot = anchor.slot();
    slot.context() == contract.constraints().context()
        && slot.context_identity().as_str() == contract.constraints().context_type()
        && dimensions.entity_slot_exists(slot)
        && matching_traversal_count(path, anchor) == 1
}

fn matching_traversal_count(
    path: &super::super::ApplicationAuthorizationPath,
    anchor: &ApplicationCapabilityPathContextAnchor,
) -> usize {
    path.traversals()
        .iter()
        .filter(|traversal| {
            traversal.relation() == anchor.relation().relation()
                && traversal.from() == anchor.relation().from()
                && traversal.to() == anchor.relation().to()
                && traversal.direction() == anchor.direction()
                && reached_entity(traversal, anchor.direction()) == anchor.slot().entity()
        })
        .count()
}

fn reached_entity(
    traversal: &super::super::ApplicationAuthorizationTraversal,
    direction: ApplicationAuthorizationTraversalDirection,
) -> &str {
    match direction {
        ApplicationAuthorizationTraversalDirection::Forward => traversal.to(),
        ApplicationAuthorizationTraversalDirection::Reverse => traversal.from(),
    }
}

fn disclosure_rule_is_closed(
    contract: &ErasedApplicationCapabilityContract,
    rule: &ApplicationCapabilityDisclosureRule,
) -> bool {
    match (contract.target().field(), rule) {
        (
            ApplicationCapabilityFieldDimension::NotApplicable,
            ApplicationCapabilityDisclosureRule::NotApplicable,
        ) => true,
        (
            ApplicationCapabilityFieldDimension::Bound(disclosed_field),
            ApplicationCapabilityDisclosureRule::Permit(guards),
        ) => {
            !guards.is_empty()
                && guards.iter().all(|guard| {
                    guard_is_owned(contract, guard)
                        && guard
                            .requirements()
                            .iter()
                            .any(|requirement| requirement.field() == disclosed_field)
                })
        }
        _ => false,
    }
}

fn guard_is_owned(
    contract: &ErasedApplicationCapabilityContract,
    guard: &ApplicationCapabilityScopeGuard,
) -> bool {
    guard
        .requirements()
        .iter()
        .all(|requirement| requirement_is_owned(contract, requirement))
}

fn requirement_is_owned(
    contract: &ErasedApplicationCapabilityContract,
    requirement: &crate::application_capability::ApplicationCapabilityAcceptedValues,
) -> bool {
    if requirement.values().is_empty() {
        return false;
    }
    let target = contract.target();
    let constraints = contract.constraints();
    let currentness = constraints.currentness();
    let delegation = contract.delegation();
    requirement_matches_fixed(requirement, target.action())
        || requirement_matches_fixed(requirement, target.purpose())
        || field_dimension_owns(target.field(), requirement.field())
        || field_dimension_owns(constraints.magnitude(), requirement.field())
        || requirement_matches_fixed(requirement, currentness.active_status())
        || requirement.field() == currentness.workflow().grant()
        || requirement.field() == currentness.validity().not_before()
        || requirement.field() == currentness.validity().not_after()
        || requirement.field() == delegation.limit()
}

fn requirement_matches_fixed(
    requirement: &crate::application_capability::ApplicationCapabilityAcceptedValues,
    fixed: &crate::application_capability::ApplicationCapabilityValueBinding,
) -> bool {
    requirement.field() == fixed.field()
        && requirement.values().len() == 1
        && requirement.values().first() == Some(fixed.value())
}

fn field_dimension_owns(
    dimension: &ApplicationCapabilityFieldDimension,
    field: &ApplicationCapabilityFieldBinding,
) -> bool {
    matches!(dimension, ApplicationCapabilityFieldDimension::Bound(bound) if bound == field)
}
