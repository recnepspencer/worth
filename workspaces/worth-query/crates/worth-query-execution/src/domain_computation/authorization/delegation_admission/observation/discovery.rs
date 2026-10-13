use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_relational::facade::authorization::{
    RelationalAuthorizationEntityAnchor, RelationalAuthorizationObservationPlan,
    RelationalAuthorizationPathPlan, RelationalAuthorizationRelatedEntityConstraint,
    RelationalAuthorizationTraversal,
};

use super::super::super::capability_registry::WorthQueryInstalledCapabilityPlan;
use super::super::super::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

use super::ObservedDelegationParent;

pub(super) fn observe_parent(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: worth_relational::facade::snapshots::SnapshotHandle,
    installed: &WorthQueryInstalledCapabilityPlan,
    grant: worth_relational::facade::identity::EntityId,
    expected_grantee: worth_relational::facade::identity::EntityId,
) -> Result<ObservedDelegationParent, WorthQueryOperationAuthorizationDenial> {
    let parent_path = RelationalAuthorizationPathPlan::new(
        [
            installed.delegation().parent.clone(),
            reverse(&installed.delegation().parent),
        ],
        [],
    )
    .with_entity_anchors([RelationalAuthorizationEntityAnchor::new(
        2,
        installed.grant_kind(),
        grant,
    )]);
    let grantee_path = RelationalAuthorizationPathPlan::new([], []).with_related_entities([
        RelationalAuthorizationRelatedEntityConstraint::new(
            0,
            installed.delegation().grantee_from_grant.clone(),
            expected_grantee,
        ),
    ]);
    let plan = RelationalAuthorizationObservationPlan::try_new(
        snapshot,
        grant,
        grant,
        installed.grant_kind(),
        installed.grant_kind(),
        [parent_path, grantee_path],
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
    let [parent_path, grantee_path] = evidence.paths() else {
        return Err(invalid_policy(installed.contract().name()));
    };
    if !grantee_path.matched()
        || !grantee_path.exhaustive()
        || !parent_path.exhaustive()
        || evidence.counters().maximum_frontier_width > 1
    {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::DelegationRejected,
            ),
            installed.contract().name(),
        ));
    }
    let parent = parent_path
        .matched()
        .then(|| {
            parent_path
                .witness()
                .and_then(|witness| witness.entity_at(1))
        })
        .flatten();
    if parent_path.matched() != parent.is_some() {
        return Err(invalid_policy(installed.contract().name()));
    }
    Ok(ObservedDelegationParent { evidence, parent })
}

fn reverse(traversal: &RelationalAuthorizationTraversal) -> RelationalAuthorizationTraversal {
    RelationalAuthorizationTraversal::new(
        traversal.relation_kind(),
        traversal.from_kind(),
        traversal.to_kind(),
        worth_relational::facade::authorization::RelationalAuthorizationTraversalDirection::Reverse,
    )
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
