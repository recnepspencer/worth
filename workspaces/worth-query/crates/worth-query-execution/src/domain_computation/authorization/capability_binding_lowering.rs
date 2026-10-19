use worth_foundational::facade::AspectFieldLocator;
use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityFieldBinding, ApplicationCapabilityGraphClause,
        ApplicationCapabilityGraphRule, ApplicationCapabilityRelationBinding,
        ApplicationCapabilityRelationDimension, ApplicationCapabilityValueBinding,
        ErasedApplicationCapabilityContract,
    },
    application_schema::ApplicationAuthorizationTraversalDirection,
};
use worth_relational::facade::{
    authorization::{
        RelationalAuthorizationFieldOperand, RelationalAuthorizationPredicate,
        RelationalAuthorizationTraversal, RelationalAuthorizationTraversalDirection,
    },
    identity::KindId,
};

use super::capability_registry::{
    field_binding, WorthQueryCapabilityContextAnchor, WorthQueryCapabilityRequestBindings,
};
use super::{authorization_denial, WorthQueryOperationAuthorizationDenial};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;

pub(super) fn capability_principal(
    contract: &ErasedApplicationCapabilityContract,
) -> Result<&str, WorthQueryOperationAuthorizationDenial> {
    let allow = contract.composition().decision().allow().graph();
    let principal = first_clause(allow)
        .map(|clause| clause.path().principal_entity())
        .ok_or_else(|| {
            authorization_denial(
                contract.name(),
                "capability allow composition has no clauses",
            )
        })?;
    for graph in capability_graph_rules(contract) {
        for requirement in graph.requirements() {
            for clause in requirement.clauses() {
                if clause.path().principal_entity() != principal
                    || clause.path().scope_entity()
                        != contract.target().resource_path().scope_entity()
                {
                    return Err(authorization_denial(
                        contract.name(),
                        "capability composition changes principal or scope",
                    ));
                }
            }
        }
    }
    Ok(principal)
}

fn first_clause(
    graph: &ApplicationCapabilityGraphRule,
) -> Option<&ApplicationCapabilityGraphClause> {
    graph
        .requirements()
        .first()
        .and_then(|requirement| requirement.clauses().first())
}

fn capability_graph_rules(
    contract: &ErasedApplicationCapabilityContract,
) -> impl Iterator<Item = &ApplicationCapabilityGraphRule> {
    let composition = contract.composition();
    std::iter::once(composition.decision().allow().graph()).chain(
        [
            composition.decision().deny().graph(),
            composition.decision().conflict().graph(),
            composition.actors().separation_of_duty().graph(),
            composition.actors().distinct_actor().graph(),
        ]
        .into_iter()
        .flatten(),
    )
}

pub(super) fn request_bindings(
    contract: &ErasedApplicationCapabilityContract,
    layout: &WorthQueryPrimaryGraphLayout,
) -> Result<WorthQueryCapabilityRequestBindings, WorthQueryOperationAuthorizationDenial> {
    let target = contract.target();
    let constraints = contract.constraints();
    let currentness = constraints.currentness();
    Ok(WorthQueryCapabilityRequestBindings {
        action: target.action().value().clone(),
        purpose: target.purpose().value().clone(),
        resource_entity: target.resource_path().scope_entity().to_string(),
        related_relation: match target.relation() {
            ApplicationCapabilityRelationDimension::NotApplicable => None,
            ApplicationCapabilityRelationDimension::Bound(binding) => Some(relation(
                layout,
                binding,
                RelationalAuthorizationTraversalDirection::Forward,
            )?),
        },
        field: field_binding(target.field())
            .map(|binding| field_locator(layout, binding))
            .transpose()?,
        magnitude: field_binding(constraints.magnitude())
            .map(|binding| field_locator(layout, binding))
            .transpose()?,
        cardinality: constraints.cardinality(),
        timeline: currentness.validity().timeline(),
        not_before: field_locator(layout, currentness.validity().not_before())?,
        not_after: field_locator(layout, currentness.validity().not_after())?,
        context: constraints.context().to_string(),
        context_type: constraints.context_type().to_string(),
    })
}

pub(super) fn lower_context_anchors(
    contract: &ErasedApplicationCapabilityContract,
    layout: &WorthQueryPrimaryGraphLayout,
    clause: &ApplicationCapabilityGraphClause,
) -> Result<Vec<WorthQueryCapabilityContextAnchor>, WorthQueryOperationAuthorizationDenial> {
    let mut lowered = Vec::with_capacity(clause.context_anchors().len());
    for anchor in clause.context_anchors() {
        if anchor.slot().context() != contract.constraints().context()
            || anchor.slot().context_identity() != contract.constraints().context_identity()
        {
            return Err(authorization_denial(
                anchor.slot().slot(),
                "context anchor belongs to another capability context",
            ));
        }
        let matching = clause
            .path()
            .traversals()
            .iter()
            .enumerate()
            .filter(|(_, traversal)| {
                traversal.relation() == anchor.relation().relation()
                    && traversal.direction() == anchor.direction()
            })
            .collect::<Vec<_>>();
        let [(index, traversal)] = matching.as_slice() else {
            return Err(authorization_denial(
                anchor.relation().relation(),
                "context anchor relation is missing or ambiguous",
            ));
        };
        let entity = match traversal.direction() {
            ApplicationAuthorizationTraversalDirection::Forward => traversal.to(),
            ApplicationAuthorizationTraversalDirection::Reverse => traversal.from(),
        };
        if entity != anchor.slot().entity() {
            return Err(authorization_denial(
                anchor.slot().slot(),
                "context anchor entity does not match its path",
            ));
        }
        lowered.push(WorthQueryCapabilityContextAnchor {
            ordinal: index + 1,
            kind: kind(layout, entity)?,
            context: anchor.slot().context().to_string(),
            context_type: anchor.slot().context_identity().as_str().to_string(),
            slot: anchor.slot().slot().to_string(),
            slot_type: anchor.slot().slot_identity().as_str().to_string(),
            entity: entity.to_string(),
        });
    }
    Ok(lowered)
}

pub(super) fn relation(
    layout: &WorthQueryPrimaryGraphLayout,
    binding: &ApplicationCapabilityRelationBinding,
    direction: RelationalAuthorizationTraversalDirection,
) -> Result<RelationalAuthorizationTraversal, WorthQueryOperationAuthorizationDenial> {
    let relation = layout.relation(binding.relation()).ok_or_else(|| {
        authorization_denial(binding.relation(), "capability relation is not installed")
    })?;
    let from = kind(layout, binding.from())?;
    let to = kind(layout, binding.to())?;
    if relation.from != from || relation.to != to {
        return Err(authorization_denial(
            binding.relation(),
            "capability relation endpoints changed",
        ));
    }
    Ok(RelationalAuthorizationTraversal::new(
        relation.kind,
        from,
        to,
        direction,
    ))
}

pub(super) fn predicate(
    layout: &WorthQueryPrimaryGraphLayout,
    ordinal: usize,
    entity_kind: KindId,
    binding: &ApplicationCapabilityValueBinding,
) -> Result<RelationalAuthorizationPredicate, WorthQueryOperationAuthorizationDenial> {
    Ok(RelationalAuthorizationPredicate::new(
        ordinal,
        entity_kind,
        field_locator(layout, binding.field())?,
        binding.value().clone(),
    ))
}

pub(super) fn operand(
    layout: &WorthQueryPrimaryGraphLayout,
    ordinal: usize,
    entity_kind: KindId,
    binding: &ApplicationCapabilityFieldBinding,
) -> Result<RelationalAuthorizationFieldOperand, WorthQueryOperationAuthorizationDenial> {
    Ok(RelationalAuthorizationFieldOperand::new(
        ordinal,
        entity_kind,
        field_locator(layout, binding)?,
    ))
}

pub(super) fn field_locator(
    layout: &WorthQueryPrimaryGraphLayout,
    binding: &ApplicationCapabilityFieldBinding,
) -> Result<AspectFieldLocator, WorthQueryOperationAuthorizationDenial> {
    layout
        .field_locator(binding.entity(), binding.aspect(), binding.field())
        .cloned()
        .ok_or_else(|| authorization_denial(binding.field(), "capability field is not installed"))
}

pub(super) fn kind(
    layout: &WorthQueryPrimaryGraphLayout,
    entity: &str,
) -> Result<KindId, WorthQueryOperationAuthorizationDenial> {
    layout
        .entity_kind(entity)
        .ok_or_else(|| authorization_denial(entity, "capability entity is not installed"))
}

pub(super) fn clause_identity(capability: &[u8; 32], path_index: usize) -> [u8; 32] {
    let mut identity = *capability;
    identity[0] ^= 0xc7;
    identity[24..].copy_from_slice(&(path_index as u64).to_be_bytes());
    identity
}
