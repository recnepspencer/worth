//! Installed paths for exact active elevation and temporal posture.

use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityElevationDefinition, ApplicationCapabilityValueBinding,
};
use worth_relational::facade::authorization::RelationalAuthorizationPathPlan;
use worth_relational::facade::identity::KindId;
use worth_runtime_bridge::facade::BridgeAuthorizationRuleEffect;

use super::ElevationRelations;
use crate::domain_computation::authorization::capability_binding_lowering::predicate;
use crate::domain_computation::authorization::capability_lowering::accumulator::WorthQueryCapabilityRuleLoweringAccumulator;
use crate::domain_computation::authorization::capability_lowering::clause_identity;
use crate::domain_computation::authorization::capability_registry::{
    WorthQueryCapabilityPathTemplate, WorthQueryCapabilityRequestGuard,
};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::graph_layout::WorthQueryPrimaryGraphLayout;

pub(super) struct ActiveElevationPathIndices {
    pub(super) active: usize,
    pub(super) not_before: usize,
    pub(super) not_after: usize,
    pub(super) expired: usize,
    pub(super) self_approval: usize,
}

pub(super) fn compile(
    layout: &WorthQueryPrimaryGraphLayout,
    capability: &[u8; 32],
    elevation: &ApplicationCapabilityElevationDefinition,
    elevation_kind: KindId,
    relations: &ElevationRelations,
    lowering: &mut WorthQueryCapabilityRuleLoweringAccumulator,
) -> Result<ActiveElevationPathIndices, WorthQueryOperationAuthorizationDenial> {
    let active = push_path(
        active_path(
            layout,
            capability,
            elevation,
            elevation_kind,
            lowering.path_count(),
            relations,
        )?,
        BridgeAuthorizationRuleEffect::Required,
        lowering,
    );
    let not_before = push_temporal_path(capability, relations, lowering)?;
    let not_after = push_temporal_path(capability, relations, lowering)?;
    let expired = push_path(
        expired_path(
            layout,
            capability,
            elevation_kind,
            lowering.path_count(),
            relations,
            elevation.states().expired(),
        )?,
        BridgeAuthorizationRuleEffect::Prohibited,
        lowering,
    );
    let self_approval = push_path(
        self_approval_path(capability, lowering.path_count(), relations),
        BridgeAuthorizationRuleEffect::Prohibited,
        lowering,
    );
    Ok(ActiveElevationPathIndices {
        active,
        not_before,
        not_after,
        expired,
        self_approval,
    })
}

fn push_temporal_path(
    capability: &[u8; 32],
    relations: &ElevationRelations,
    lowering: &mut WorthQueryCapabilityRuleLoweringAccumulator,
) -> Result<usize, WorthQueryOperationAuthorizationDenial> {
    Ok(push_path(
        temporal_path(capability, lowering.path_count(), relations),
        BridgeAuthorizationRuleEffect::Required,
        lowering,
    ))
}

fn active_path(
    layout: &WorthQueryPrimaryGraphLayout,
    capability: &[u8; 32],
    elevation: &ApplicationCapabilityElevationDefinition,
    elevation_kind: KindId,
    path_index: usize,
    relations: &ElevationRelations,
) -> Result<WorthQueryCapabilityPathTemplate, WorthQueryOperationAuthorizationDenial> {
    Ok(WorthQueryCapabilityPathTemplate {
        plan: RelationalAuthorizationPathPlan::new(
            [
                relations.requester.clone(),
                relations.approver_reverse.clone(),
                relations.approver_forward.clone(),
                relations.grant.clone(),
                relations.resource.clone(),
            ],
            [predicate(
                layout,
                1,
                elevation_kind,
                elevation.states().approved(),
            )?],
        ),
        identity: clause_identity(capability, path_index),
        guard: WorthQueryCapabilityRequestGuard::Unconditional,
        grant_ordinal: Some(4),
        elevation_ordinals: vec![1, 3],
        elevation_resource_ordinal: elevation.resource_relation().map(|_| 1),
        context_anchors: Vec::new(),
    })
}

fn temporal_path(
    capability: &[u8; 32],
    path_index: usize,
    relations: &ElevationRelations,
) -> WorthQueryCapabilityPathTemplate {
    WorthQueryCapabilityPathTemplate {
        plan: RelationalAuthorizationPathPlan::new(
            [
                relations.requester.clone(),
                relations.grant.clone(),
                relations.resource.clone(),
            ],
            [],
        ),
        identity: clause_identity(capability, path_index),
        guard: WorthQueryCapabilityRequestGuard::Unconditional,
        grant_ordinal: Some(2),
        elevation_ordinals: vec![1],
        elevation_resource_ordinal: Some(1),
        context_anchors: Vec::new(),
    }
}

fn expired_path(
    layout: &WorthQueryPrimaryGraphLayout,
    capability: &[u8; 32],
    elevation_kind: KindId,
    path_index: usize,
    relations: &ElevationRelations,
    expired: &ApplicationCapabilityValueBinding,
) -> Result<WorthQueryCapabilityPathTemplate, WorthQueryOperationAuthorizationDenial> {
    let mut path = temporal_path(capability, path_index, relations);
    path.plan = path
        .plan
        .with_predicates([predicate(layout, 1, elevation_kind, expired)?]);
    Ok(path)
}

fn self_approval_path(
    capability: &[u8; 32],
    path_index: usize,
    relations: &ElevationRelations,
) -> WorthQueryCapabilityPathTemplate {
    WorthQueryCapabilityPathTemplate {
        plan: RelationalAuthorizationPathPlan::new(
            [
                relations.approver_forward.clone(),
                relations.grant.clone(),
                relations.resource.clone(),
            ],
            [],
        ),
        identity: clause_identity(capability, path_index),
        guard: WorthQueryCapabilityRequestGuard::Unconditional,
        grant_ordinal: Some(2),
        elevation_ordinals: vec![1],
        elevation_resource_ordinal: None,
        context_anchors: Vec::new(),
    }
}

fn push_path(
    path: WorthQueryCapabilityPathTemplate,
    effect: BridgeAuthorizationRuleEffect,
    lowering: &mut WorthQueryCapabilityRuleLoweringAccumulator,
) -> usize {
    lowering.add_elevation(effect, path)
}
