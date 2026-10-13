use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_relational::facade::authorization::{
    RelationalAuthorizationEntityAnchor, RelationalAuthorizationFieldComparison,
    RelationalAuthorizationFieldConstraint, RelationalAuthorizationFieldOperand,
    RelationalAuthorizationObservationPlan, RelationalAuthorizationPathPlan,
    RelationalAuthorizationPredicate, RelationalAuthorizationRelatedEntityConstraint,
};

use super::super::super::capability_registry::WorthQueryInstalledCapabilityPlan;
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
    let narrowing = RelationalAuthorizationPathPlan::new(
        [
            installed.delegation().parent.clone(),
            installed.delegation().resource.clone(),
        ],
        parent_predicates(installed, sample),
    )
    .with_entity_anchors([RelationalAuthorizationEntityAnchor::new(
        1,
        installed.grant_kind(),
        parent,
    )])
    .with_field_constraints(narrowing_constraints(installed))
    .with_related_entities(related_constraint(installed, request));
    let link = RelationalAuthorizationPathPlan::new(
        [
            installed.delegation().grantor.clone(),
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
    vec![
        RelationalAuthorizationPredicate::new(
            1,
            installed.grant_kind(),
            installed.delegation().active_status.0.clone(),
            installed.delegation().active_status.1.clone(),
        ),
        RelationalAuthorizationPredicate::compare(
            1,
            installed.grant_kind(),
            installed.delegation().not_before.clone(),
            RelationalAuthorizationFieldComparison::AtMost,
            sample.value().clone(),
        ),
        RelationalAuthorizationPredicate::compare(
            1,
            installed.grant_kind(),
            installed.delegation().not_after.clone(),
            RelationalAuthorizationFieldComparison::AtLeast,
            sample.value().clone(),
        ),
    ]
}

fn narrowing_constraints(
    installed: &WorthQueryInstalledCapabilityPlan,
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
            &bindings.grant_workflow,
            RelationalAuthorizationFieldComparison::Equal,
        ),
        constraint(
            installed,
            &bindings.not_before,
            RelationalAuthorizationFieldComparison::AtLeast,
        ),
        constraint(
            installed,
            &bindings.not_after,
            RelationalAuthorizationFieldComparison::AtMost,
        ),
        constraint(
            installed,
            &bindings.remaining,
            RelationalAuthorizationFieldComparison::StrictlyLess,
        ),
    ];
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
            bindings.grant_workflow.clone(),
        ),
        RelationalAuthorizationFieldComparison::Equal,
        RelationalAuthorizationFieldOperand::new(
            2,
            installed.scope_kind(),
            bindings.resource_workflow.clone(),
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
