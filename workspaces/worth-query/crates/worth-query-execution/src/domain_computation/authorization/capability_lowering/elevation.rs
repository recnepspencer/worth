//! Installed active-elevation composition.

use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityElevationDefinition, ErasedApplicationCapabilityContract,
};
use worth_relational::facade::authorization::{
    RelationalAuthorizationTraversal, RelationalAuthorizationTraversalDirection,
};
use worth_relational::facade::identity::KindId;

use super::accumulator::WorthQueryCapabilityRuleLoweringAccumulator;
use crate::domain_computation::authorization::capability_binding_lowering::{
    field_locator, kind, relation,
};
use crate::domain_computation::authorization::capability_registry::{
    WorthQueryCapabilityElevationBindings, WorthQueryCapabilityElevationLifecycleBindings,
    WorthQueryCapabilityElevationTemporalBindings,
};
use crate::domain_computation::authorization::{
    authorization_denial, WorthQueryOperationAuthorizationDenial,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;

mod active_use;
mod approver_conflict;

pub(super) fn compile_elevation_rules(
    contract: &ErasedApplicationCapabilityContract,
    layout: &WorthQueryPrimaryGraphLayout,
    capability: &[u8; 32],
    principal_kind: KindId,
    grant_kind: KindId,
    scope_kind: KindId,
    lowering: &mut WorthQueryCapabilityRuleLoweringAccumulator,
) -> Result<Option<WorthQueryCapabilityElevationBindings>, WorthQueryOperationAuthorizationDenial> {
    let Some(elevation) = contract.elevation().definition() else {
        return Ok(None);
    };
    let elevation_kind = kind(layout, elevation.identity().entity())?;
    if elevation.validity().timeline() != contract.constraints().currentness().validity().timeline()
    {
        return Err(authorization_denial(
            contract.name(),
            "elevation and grant validity timelines differ",
        ));
    }
    let relations = lower_relations(layout, contract, elevation)?;
    relations.validate_endpoints(
        contract,
        principal_kind,
        elevation_kind,
        grant_kind,
        scope_kind,
    )?;

    let active = active_use::compile(
        layout,
        capability,
        elevation,
        elevation_kind,
        &relations,
        lowering,
    )?;
    let approver_conflict_requirements =
        approver_conflict::compile(contract, layout, capability, &relations, lowering)?;
    Ok(Some(WorthQueryCapabilityElevationBindings::new(
        elevation_kind,
        active.active,
        active.expired,
        active.self_approval,
        WorthQueryCapabilityElevationTemporalBindings::new(
            elevation.validity().timeline(),
            active.not_before,
            active.not_after,
            field_locator(layout, elevation.validity().not_before())?,
            field_locator(layout, elevation.validity().not_after())?,
        ),
        approver_conflict_requirements,
        lower_lifecycle(layout, elevation)?,
    )))
}

fn lower_lifecycle(
    layout: &WorthQueryPrimaryGraphLayout,
    elevation: &ApplicationCapabilityElevationDefinition,
) -> Result<WorthQueryCapabilityElevationLifecycleBindings, WorthQueryOperationAuthorizationDenial>
{
    let relation_kind = |binding| {
        relation(
            layout,
            binding,
            RelationalAuthorizationTraversalDirection::Forward,
        )
        .map(|relation| relation.relation_kind())
    };
    let review = elevation.review();
    Ok(WorthQueryCapabilityElevationLifecycleBindings {
        review_kind: kind(layout, review.identity().entity())?,
        identity: field_locator(layout, elevation.identity())?,
        reason: field_locator(layout, elevation.reason())?,
        status: field_locator(layout, elevation.status())?,
        closed_at: field_locator(layout, elevation.closed_at())?,
        review_identity: field_locator(layout, review.identity())?,
        review_type: field_locator(layout, review.kind().field())?,
        review_type_value: review.kind().value().clone(),
        review_status: field_locator(layout, review.status())?,
        reviewed_at: field_locator(layout, review.reviewed_at())?,
        requester_relation: relation_kind(elevation.requester())?,
        approver_relation: relation_kind(elevation.approver())?,
        grant_relation: relation_kind(elevation.grant())?,
        resource_relation: elevation
            .resource_relation()
            .map(relation_kind)
            .transpose()?,
        review_relation: relation_kind(review.relation())?,
        review_scope_relation: relation_kind(review.scope())?,
        reviewer_relation: relation_kind(review.reviewer())?,
        requested: elevation.states().requested().value().clone(),
        approved: elevation.states().approved().value().clone(),
        expired: elevation.states().expired().value().clone(),
        revoked: elevation.states().revoked().value().clone(),
        review_required: review.required().value().clone(),
        review_completed: review.completed().value().clone(),
        maximum_duration: elevation.maximum_duration(),
    })
}

struct ElevationRelations {
    requester: RelationalAuthorizationTraversal,
    approver_reverse: RelationalAuthorizationTraversal,
    approver_forward: RelationalAuthorizationTraversal,
    grant: RelationalAuthorizationTraversal,
    resource: RelationalAuthorizationTraversal,
}

impl ElevationRelations {
    fn validate_endpoints(
        &self,
        contract: &ErasedApplicationCapabilityContract,
        principal_kind: KindId,
        elevation_kind: KindId,
        grant_kind: KindId,
        scope_kind: KindId,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        if self.requester.from_kind() != principal_kind
            || self.requester.to_kind() != elevation_kind
            || self.approver_forward.from_kind() != principal_kind
            || self.approver_forward.to_kind() != elevation_kind
            || self.approver_reverse.from_kind() != principal_kind
            || self.approver_reverse.to_kind() != elevation_kind
            || self.grant.from_kind() != elevation_kind
            || self.grant.to_kind() != grant_kind
            || self.resource.from_kind() != grant_kind
            || self.resource.to_kind() != scope_kind
        {
            return Err(authorization_denial(
                contract.name(),
                "elevation relation endpoints changed",
            ));
        }
        Ok(())
    }
}

fn lower_relations(
    layout: &WorthQueryPrimaryGraphLayout,
    contract: &ErasedApplicationCapabilityContract,
    elevation: &ApplicationCapabilityElevationDefinition,
) -> Result<ElevationRelations, WorthQueryOperationAuthorizationDenial> {
    let lower = |binding, direction| relation(layout, binding, direction);
    Ok(ElevationRelations {
        requester: lower(
            elevation.requester(),
            RelationalAuthorizationTraversalDirection::Forward,
        )?,
        approver_reverse: lower(
            elevation.approver(),
            RelationalAuthorizationTraversalDirection::Reverse,
        )?,
        approver_forward: lower(
            elevation.approver(),
            RelationalAuthorizationTraversalDirection::Forward,
        )?,
        grant: lower(
            elevation.grant(),
            RelationalAuthorizationTraversalDirection::Forward,
        )?,
        resource: lower(
            contract.target().resource(),
            RelationalAuthorizationTraversalDirection::Forward,
        )?,
    })
}
