use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_relational::facade::authorization::{
    RelationalAuthorizationEntityAnchor, RelationalAuthorizationFieldComparison,
    RelationalAuthorizationFieldConstraint, RelationalAuthorizationFieldOperand,
    RelationalAuthorizationObservationPlan, RelationalAuthorizationPathPlan,
    RelationalAuthorizationPredicate, RelationalAuthorizationRelatedEntityConstraint,
};

use super::super::super::capability_registry::{
    WorthQueryCapabilityLineageBindings, WorthQueryInstalledCapabilityPlan,
};
use super::super::super::retained_capability_request::WorthQueryRetainedCapabilityRequest;
use super::super::super::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryRuntimeTimeSample,
};

use super::ObservedDelegationTransition;

#[allow(clippy::too_many_arguments)]
pub(super) fn observe_transition(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: worth_relational::facade::snapshots::SnapshotHandle,
    installed: &WorthQueryInstalledCapabilityPlan,
    request: &WorthQueryRetainedCapabilityRequest,
    sample: &WorthQueryRuntimeTimeSample,
    child: worth_relational::facade::identity::EntityId,
    parent: worth_relational::facade::identity::EntityId,
) -> Result<ObservedDelegationTransition, WorthQueryOperationAuthorizationDenial> {
    let lineage = installed
        .delegation()
        .delegable_lineage(installed.contract().name())?;
    let narrowing = RelationalAuthorizationPathPlan::new(
        [
            lineage.parent.clone(),
            installed.delegation().resource.clone(),
        ],
        parent_predicates(installed, sample),
    )
    .with_entity_anchors([RelationalAuthorizationEntityAnchor::new(
        1,
        installed.grant_kind(),
        parent,
    )])
    .with_field_constraints(narrowing_constraints(installed, lineage))
    .with_related_entities(related_constraint(installed, request));
    let link = RelationalAuthorizationPathPlan::new(
        [
            lineage.grantor.clone(),
            installed.delegation().grantee.clone(),
            installed.delegation().resource.clone(),
        ],
        [],
    )
    .with_entity_anchors([RelationalAuthorizationEntityAnchor::new(
        2,
        installed.grant_kind(),
        parent,
    )]);
    let plan = RelationalAuthorizationObservationPlan::try_new(
        snapshot,
        child,
        request.resource(),
        installed.grant_kind(),
        installed.scope_kind(),
        [narrowing, link],
        [],
    )
    .map_err(|_| invalid_policy(installed.contract().name()))?;
    let evidence = relational
        .observe_authorization(plan, execution)
        .map_err(|_| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::RelationalObservationRejected,
                ),
                installed.contract().name(),
            )
        })?;
    let [narrowing, link] = evidence.paths() else {
        return Err(invalid_policy(installed.contract().name()));
    };
    let grantor = link.witness().and_then(|witness| witness.entity_at(1));
    if !narrowing.matched()
        || !link.matched()
        || !narrowing.exhaustive()
        || !link.exhaustive()
        || evidence.counters().maximum_frontier_width > 1
        || grantor.is_none()
    {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::DelegationRejected,
            ),
            installed.contract().name(),
        ));
    }
    Ok(ObservedDelegationTransition {
        evidence,
        grantor: grantor.expect("checked above"),
    })
}

fn parent_predicates(
    installed: &WorthQueryInstalledCapabilityPlan,
    sample: &WorthQueryRuntimeTimeSample,
) -> Vec<RelationalAuthorizationPredicate> {
    let delegation = installed.delegation();
    // An open bound admits every instant on its side: it adds no predicate.
    let bounds = [
        (
            &delegation.not_before,
            RelationalAuthorizationFieldComparison::AtMost,
        ),
        (
            &delegation.not_after,
            RelationalAuthorizationFieldComparison::AtLeast,
        ),
    ];
    std::iter::once(RelationalAuthorizationPredicate::new(
        1,
        installed.grant_kind(),
        delegation.active_status.0.clone(),
        delegation.active_status.1.clone(),
    ))
    .chain(bounds.into_iter().filter_map(|(bound, comparison)| {
        bound.as_ref().map(|bound| {
            RelationalAuthorizationPredicate::compare(
                1,
                installed.grant_kind(),
                bound.clone(),
                comparison,
                sample.value().clone(),
            )
        })
    }))
    .collect()
}

fn narrowing_constraints(
    installed: &WorthQueryInstalledCapabilityPlan,
    lineage: &WorthQueryCapabilityLineageBindings,
) -> Vec<RelationalAuthorizationFieldConstraint> {
    let bindings = &installed.delegation();
    let mut constraints = vec![
        constraint(
            installed,
            &bindings.action,
            RelationalAuthorizationFieldComparison::Equal,
        ),
        constraint(
            installed,
            &bindings.purpose,
            RelationalAuthorizationFieldComparison::Equal,
        ),
        constraint(
            installed,
            &lineage.grant_workflow,
            RelationalAuthorizationFieldComparison::Equal,
        ),
        constraint(
            installed,
            &lineage.remaining,
            RelationalAuthorizationFieldComparison::StrictlyLess,
        ),
    ];
    // A child sits inside its parent's window on every closed side; an open
    // side is unbounded for parent and child alike, so it constrains nothing.
    if let Some(not_before) = &bindings.not_before {
        constraints.push(constraint(
            installed,
            not_before,
            RelationalAuthorizationFieldComparison::AtLeast,
        ));
    }
    if let Some(not_after) = &bindings.not_after {
        constraints.push(constraint(
            installed,
            not_after,
            RelationalAuthorizationFieldComparison::AtMost,
        ));
    }
    if let Some(disclosure) = &bindings.disclosure {
        constraints.push(constraint(
            installed,
            disclosure,
            RelationalAuthorizationFieldComparison::Equal,
        ));
    }
    if let Some(amount) = &bindings.magnitude {
        constraints.push(constraint(
            installed,
            amount,
            RelationalAuthorizationFieldComparison::AtMost,
        ));
    }
    constraints.push(RelationalAuthorizationFieldConstraint::new(
        RelationalAuthorizationFieldOperand::new(
            1,
            installed.grant_kind(),
            lineage.grant_workflow.clone(),
        ),
        RelationalAuthorizationFieldComparison::Equal,
        RelationalAuthorizationFieldOperand::new(
            2,
            installed.scope_kind(),
            lineage.resource_workflow.clone(),
        ),
    ));
    constraints
}

fn constraint(
    installed: &WorthQueryInstalledCapabilityPlan,
    field: &worth_foundational::facade::AspectFieldLocator,
    comparison: RelationalAuthorizationFieldComparison,
) -> RelationalAuthorizationFieldConstraint {
    RelationalAuthorizationFieldConstraint::new(
        RelationalAuthorizationFieldOperand::new(0, installed.grant_kind(), field.clone()),
        comparison,
        RelationalAuthorizationFieldOperand::new(1, installed.grant_kind(), field.clone()),
    )
}

fn related_constraint(
    installed: &WorthQueryInstalledCapabilityPlan,
    request: &WorthQueryRetainedCapabilityRequest,
) -> Vec<RelationalAuthorizationRelatedEntityConstraint> {
    match (&installed.delegation().related, request.related()) {
        (Some(traversal), Some(entity)) => {
            vec![RelationalAuthorizationRelatedEntityConstraint::new(
                1,
                traversal.clone(),
                entity,
            )]
        }
        (None, None) => Vec::new(),
        (Some(_), None) | (None, Some(_)) => Vec::new(),
    }
}

fn invalid_policy(subject: &str) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::InvalidInstalledPolicy,
        ),
        subject,
    )
}

fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
