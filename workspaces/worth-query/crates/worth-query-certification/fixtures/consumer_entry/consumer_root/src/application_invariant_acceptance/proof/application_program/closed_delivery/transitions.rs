//! Wrong and stopped requests cannot discard native or already-closed custody.
use super::*;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_entry::{WorthQueryOperationCloseDenial, WorthQueryProgramOutputCloseDenial},
};

pub(super) fn close_fixed_after_foreign_refusal(
    fixed: Fixed,
    wrong: &Request<'_, '_, '_>,
    request: &Request<'_, '_, '_>,
) -> ClosedFixed {
    let fixed = match fixed.close_for_delivery(wrong) {
        Err((original, WorthQueryOperationCloseDenial::FreshRequestMismatch)) => original,
        _ => panic!("wrong native branch refuses without losing original Fixed custody"),
    };
    fixed
        .close_for_delivery(request)
        .unwrap_or_else(|(_, stop)| panic!("original Fixed close: {stop:?}"))
}

pub(super) fn close_tree_after_cancelled_refusal(tree: Tree, world: &World) -> ClosedTree {
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token());
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &stopped,
    ))
    .unwrap();
    cancellation.cancel();
    let refused = match tree.close_for_delivery(&world.application.request(&principal, &stopped)) {
        Err(original) => original,
        Ok(_) => panic!("stopped fresh authority cannot discharge original tree custody"),
    };
    assert!(matches!(
        refused.denial(),
        WorthQueryProgramOutputCloseDenial::Execution(WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled
        ))
    ));
    // The close-only refusal and its exact native prefix escape the stopped scope.
    let fresh_scope = authentication::request_scope();
    let fresh_principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &fresh_scope,
    ))
    .unwrap();
    let fresh = world
        .application
        .request(&fresh_principal, &fresh_scope)
        .on_branch(refused.branch());
    refused
        .retry_close(&fresh)
        .unwrap_or_else(|stop| panic!("same native tree close retry: {:?}", stop.denial()))
}

pub(super) fn cancelled_delivery_retains_closed(
    closed: &ClosedTree,
    world: &World,
    request: &Request<'_, '_, '_>,
) {
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token());
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &stopped,
    ))
    .unwrap();
    cancellation.cancel();
    let denied = closed.read_current(
        &world
            .application
            .request(&principal, &stopped)
            .on_branch(closed.branch()),
        PlanarOutputRead {
            body_key: "anchor-a".into(),
        },
    );
    assert!(matches!(
        denied,
        Err(WorthQueryOperationDeliveryDenial::Execution(
            WorthQueryAdvancementDenial::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled
            )
        ))
    ));
    assert_eq!(
        closed
            .read_current(
                request,
                PlanarOutputRead {
                    body_key: "anchor-a".into()
                }
            )
            .unwrap()
            .rows()[0]
            .value,
        length(3)
    );
}
