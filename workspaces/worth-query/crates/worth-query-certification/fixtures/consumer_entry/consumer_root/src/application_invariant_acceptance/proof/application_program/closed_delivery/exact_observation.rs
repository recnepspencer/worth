//! Both closed owners run their complete native gate at one caller-held observation.
use super::*;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_entry::{WorthQueryApplicationReadObservation, WorthQueryOperationDeliveryPolicy},
};

pub(super) fn installed_policy(world: &World) -> WorthQueryOperationDeliveryPolicy {
    WorthQueryOperationDeliveryPolicy::currentness(
        NonZeroUsize::new(
            world
                .application
                .runtime()
                .output_demand_resource_profile()
                .limits()
                .source_currentness_work(),
        )
        .unwrap(),
    )
}

/// A stopped scope and a foreign-branch observation refuse; the same owners then
/// deliver at the one common observation where the application reads its facts.
pub(super) fn one_caller_observation(
    closed_fixed: &ClosedFixed,
    closed_tree: &ClosedTree,
    world: &World,
    request: &Request<'_, '_, '_>,
    foreign: &WorthQueryApplicationReadObservation,
) {
    let policy = installed_policy(world);
    let common = request.retain_current_read().unwrap();
    let cancellation = WorthQueryCancellationSource::new();
    let stopped_scope = WorthQueryRequestScope::new(None, cancellation.token());
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &stopped_scope,
    ))
    .unwrap();
    cancellation.cancel();
    let stopped = world
        .application
        .request(&principal, &stopped_scope)
        .on_branch(closed_tree.branch());
    let cancelled = |denied: Result<(), WorthQueryOperationDeliveryDenial>| {
        matches!(
            denied,
            Err(WorthQueryOperationDeliveryDenial::Execution(
                WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                )
            ))
        )
    };
    assert!(cancelled(closed_fixed.deliver_current_at(&stopped.at(&common), policy)));
    assert!(cancelled(closed_tree.deliver_current_at(&stopped.at(&common), policy)));
    // An observation retained on another branch is refused before installed meaning.
    let foreign_branch = |denied: Result<(), WorthQueryOperationDeliveryDenial>| {
        matches!(
            denied,
            Err(WorthQueryOperationDeliveryDenial::Currentness(
                WorthQueryOutputCurrentnessDenial::ForeignBranch
            ))
        )
    };
    assert!(foreign_branch(closed_fixed.deliver_current_at(&request.at(foreign), policy)));
    assert!(foreign_branch(closed_tree.deliver_current_at(&request.at(foreign), policy)));
    let pinned = request.at(&common);
    closed_fixed
        .deliver_current_at(&pinned, policy)
        .expect("original action and full lineage at the common observation");
    closed_tree
        .deliver_current_at(&pinned, policy)
        .expect("selected typed root and full tree at the common observation");
    assert_eq!(
        pinned
            .query(PlanarOutputRead {
                body_key: "anchor-a".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        length(3)
    );
}

/// After an actual consumed source edit, a caller observation of that edit
/// carries each owner's typed native supersession, not a weaker lineage verdict.
pub(super) fn superseded_at_caller_observation(
    closed_fixed: &ClosedFixed,
    closed_tree: &ClosedTree,
    world: &World,
    request: &Request<'_, '_, '_>,
) {
    let policy = installed_policy(world);
    let edited = request.retain_current_read().unwrap();
    let pinned = request.at(&edited);
    assert!(matches!(
        closed_fixed.deliver_current_at(&pinned, policy),
        Err(WorthQueryOperationDeliveryDenial::Stale(cause))
            if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded
    ));
    assert!(matches!(
        closed_tree.deliver_current_at(&pinned, policy),
        Err(WorthQueryOperationDeliveryDenial::Currentness(WorthQueryOutputCurrentnessDenial::Output(cause)))
            if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded
    ));
}
