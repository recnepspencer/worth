//! Exact request anchors and trusted-time predicates for installed policy paths.

use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_relational::facade::authorization::{
    RelationalAuthorizationEntityAnchor, RelationalAuthorizationExactAdjacencyConstraint,
    RelationalAuthorizationTraversal, RelationalAuthorizationTraversalDirection,
};

use super::super::capability_registry::{
    WorthQueryCapabilityContextAnchor, WorthQueryCapabilityPathTemplate,
    WorthQueryInstalledCapabilityPlan,
};
use super::super::operation_progression::WorthQueryCapabilityContextKey;
use super::super::retained_capability_request::WorthQueryRetainedCapabilityRequest;
use super::super::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryRuntimeTimeSample,
};

pub(super) fn prepare_exact_policy_paths(
    installed: &WorthQueryInstalledCapabilityPlan,
    request: &WorthQueryRetainedCapabilityRequest,
    sample: &WorthQueryRuntimeTimeSample,
    exact_grant: worth_relational::facade::identity::EntityId,
) -> Result<
    Vec<worth_relational::facade::authorization::RelationalAuthorizationPathPlan>,
    WorthQueryOperationAuthorizationDenial,
> {
    let grant_path_index = installed.grant_witness().path_index();
    installed
        .paths()
        .iter()
        .enumerate()
        .map(|(index, template)| {
            let plan = if index == grant_path_index {
                super::grant_selection::prepare_grant_path(installed, request, sample)?
            } else if is_temporal_path(installed, index) {
                super::elevation::prepare_temporal_path(installed, template, sample, index)?
            } else {
                template.plan.clone()
            };
            let plan =
                plan.with_entity_anchors(exact_anchors(installed, request, template, exact_grant)?);
            Ok(bind_exact_elevation_resource(
                installed, request, template, plan,
            ))
        })
        .collect()
}

fn bind_exact_elevation_resource(
    installed: &WorthQueryInstalledCapabilityPlan,
    request: &WorthQueryRetainedCapabilityRequest,
    template: &WorthQueryCapabilityPathTemplate,
    plan: worth_relational::facade::authorization::RelationalAuthorizationPathPlan,
) -> worth_relational::facade::authorization::RelationalAuthorizationPathPlan {
    let (Some(ordinal), Some(relation_kind), Some(elevation)) = (
        template.elevation_resource_ordinal,
        installed
            .elevation()
            .as_ref()
            .and_then(|bindings| bindings.lifecycle.resource_relation),
        installed.elevation().as_ref(),
    ) else {
        return plan;
    };
    let mut exact_adjacencies = plan.exact_adjacencies().to_vec();
    exact_adjacencies.push(RelationalAuthorizationExactAdjacencyConstraint::new(
        ordinal,
        RelationalAuthorizationTraversal::new(
            relation_kind,
            elevation.elevation_kind,
            installed.scope_kind(),
            RelationalAuthorizationTraversalDirection::Forward,
        ),
        [request.resource()],
    ));
    plan.with_exact_adjacencies(exact_adjacencies)
}

pub(super) fn prepare_upper_bound_policy_paths(
    installed: &WorthQueryInstalledCapabilityPlan,
    request: &WorthQueryRetainedCapabilityRequest,
    sample: &WorthQueryRuntimeTimeSample,
    exact_grant: worth_relational::facade::identity::EntityId,
) -> Result<
    Vec<worth_relational::facade::authorization::RelationalAuthorizationPathPlan>,
    WorthQueryOperationAuthorizationDenial,
> {
    let upper_bound = installed
        .upper_bound()
        .as_ref()
        .ok_or_else(|| invalid_policy(installed.contract().name()))?;
    let grant_path_index = installed.grant_witness().path_index();
    installed
        .paths()
        .iter()
        .take(upper_bound.path_count)
        .enumerate()
        .map(|(index, template)| {
            let plan = if index == grant_path_index {
                super::grant_selection::prepare_grant_path(installed, request, sample)?
            } else {
                template.plan.clone()
            };
            Ok(plan.with_entity_anchors(exact_anchors(installed, request, template, exact_grant)?))
        })
        .collect()
}

fn exact_anchors(
    installed: &WorthQueryInstalledCapabilityPlan,
    request: &WorthQueryRetainedCapabilityRequest,
    template: &WorthQueryCapabilityPathTemplate,
    exact_grant: worth_relational::facade::identity::EntityId,
) -> Result<Vec<RelationalAuthorizationEntityAnchor>, WorthQueryOperationAuthorizationDenial> {
    let mut anchors = context_anchors(request, template)?;
    if let Some(ordinal) = template.grant_ordinal {
        anchors.push(RelationalAuthorizationEntityAnchor::new(
            ordinal,
            installed.grant_kind(),
            exact_grant,
        ));
    }
    if !template.elevation_ordinals.is_empty() {
        let elevation = request
            .elevation()
            .ok_or_else(|| projection_denial(installed.contract().name()))?;
        let bindings = installed
            .elevation()
            .as_ref()
            .ok_or_else(|| invalid_policy(installed.contract().name()))?;
        anchors.extend(template.elevation_ordinals.iter().map(|ordinal| {
            RelationalAuthorizationEntityAnchor::new(*ordinal, bindings.elevation_kind, elevation)
        }));
    }
    Ok(anchors)
}

pub(super) fn context_anchors(
    request: &WorthQueryRetainedCapabilityRequest,
    template: &WorthQueryCapabilityPathTemplate,
) -> Result<Vec<RelationalAuthorizationEntityAnchor>, WorthQueryOperationAuthorizationDenial> {
    template
        .context_anchors
        .iter()
        .map(|anchor| {
            request
                .context()
                .get(&context_key(anchor))
                .copied()
                .map(|entity| {
                    RelationalAuthorizationEntityAnchor::new(anchor.ordinal, anchor.kind, entity)
                })
                .ok_or_else(|| projection_denial(&anchor.slot))
        })
        .collect()
}

fn is_temporal_path(installed: &WorthQueryInstalledCapabilityPlan, index: usize) -> bool {
    installed.elevation().as_ref().is_some_and(|bindings| {
        index == bindings.temporal.not_before_path_index
            || index == bindings.temporal.not_after_path_index
    })
}

pub(super) fn context_key(
    anchor: &WorthQueryCapabilityContextAnchor,
) -> WorthQueryCapabilityContextKey {
    WorthQueryCapabilityContextKey::from_anchor(anchor)
}

fn projection_denial(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityProjectionRejected,
        ),
        subject,
    )
}

fn invalid_policy(subject: &str) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::InvalidInstalledPolicy,
        ),
        subject,
    )
}
