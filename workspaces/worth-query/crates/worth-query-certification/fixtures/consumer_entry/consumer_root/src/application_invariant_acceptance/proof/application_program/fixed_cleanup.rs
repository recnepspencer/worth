//! Independent fixed cleanup release does not dispose pending publication or output custody.
use super::*;
use crate::application_program::ConsumerFixedAction;
use worth_query_host::facade::application_entry::{
    WorthQueryOperationAdvance as Advance, WorthQueryOperationCloseDenial as Close,
    WorthQueryOperationProgress as Progress,
};
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts,
};
#[test]
fn fixed_initial_close_preserves_refused_owner_without_executing_source() {
    let world = installation::install(&crate::installed_schema());
    let adapter = authentication::admit(world.application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-b".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    let operation = request
        .mutate(input.clone())
        .expect_source(
            request
                .query(PlanarRead {
                    body_key: input.scope_key.clone(),
                })
                .execute()
                .unwrap()
                .observed_sources()[0]
                .clone(),
        )
        .idempotency_owned(10_197)
        .unwrap_or_else(|(_, _, d)| panic!("capture: {d:?}"))
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, d)| panic!("construction: {d:?}"));
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let stop=worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    stop.cancel();
    let stopped =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            None,
            stop.token(),
        );
    let (operation, denial) = operation
        .close(&world.application.request(&principal, &stopped))
        .err()
        .expect("stopped close returns Initial owner");
    assert!(matches!(denial, Close::Execution(_)));
    operation
        .close(&request)
        .unwrap_or_else(|(_, d)| panic!("Initial close: {d:?}"));
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    assert_eq!(planar_source_adjustment_contacts(&input), (0, 0));
}
#[test]
fn fixed_live_partial_releases_actual_managed_cleanup_without_business_retry() {
    let world = installation::install(&crate::installed_schema());
    let adapter = authentication::admit(world.application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-b".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    let operation = request
        .mutate(input.clone())
        .expect_source(
            request
                .query(PlanarRead {
                    body_key: input.scope_key.clone(),
                })
                .execute()
                .unwrap()
                .observed_sources()[0]
                .clone(),
        )
        .idempotency_owned(10_198)
        .unwrap_or_else(|(_, _, d)| panic!("capture: {d:?}"))
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, d)| panic!("construction: {d:?}"));
    world
        .application
        .with_host_advancement(|phase| {
            world.application.fail_next_durable_append_for_test(
                &phase
                    .checked_execution_request_for(world.application.product_runtime())
                    .unwrap(),
            )
        })
        .unwrap();
    let _cleanup = world
        .application
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    let operation = match operation.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourceUnpublished,
        } => operation,
        Advance::Terminal { operation, denial } => {
            panic!(
                "actual lower fault posture: {denial:?}, native={:?}, managed={:?}",
                operation.native_outcome(),
                operation.managed_cleanup_failure()
            )
        }
        Advance::Refused { denial, .. } => panic!("actual lower fault admission: {denial:?}"),
        _ => panic!("real durable refusal must retain the original partial"),
    };
    let (operation, denial) = operation
        .close(&request)
        .err()
        .expect("pending partial cannot close");
    assert!(matches!(denial, Close::NativeObligationRetained));
    // Promotion must retain the already absorbed managed failure, not replace its collection.
    let mut retained = Some(operation);
    let mut operation = (0..32)
        .find_map(|_| match retained.take().unwrap().advance(&request) {
            Advance::Progress {
                operation,
                progress: Progress::SourcePromoted,
            } => Some(operation),
            Advance::Progress { operation, .. } => {
                retained = Some(operation);
                None
            }
            Advance::Terminal { operation, denial } => panic!(
                "native recovery before cleanup: {denial:?}, native={:?}, managed={:?}",
                operation.native_outcome(),
                operation.managed_cleanup_failure()
            ),
            Advance::Refused { denial, .. } => panic!("native recovery admission: {denial:?}"),
            Advance::Settled(_) => panic!("source promotion precedes output progression"),
        })
        .expect("the same original partial promotes without repeating its handler");
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let data = request
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .y;
    let stop=worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    stop.cancel();
    let stopped =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            None,
            stop.token(),
        );
    assert!(matches!(
        operation.release_cleanup(&world.application.request(&principal, &stopped)),
        Err(Close::Execution(_))
    ));
    let released = operation
        .release_cleanup(&request)
        .expect("owning thread lawfully releases the same actual cleanup");
    assert!(released
        .managed_cleanup_receipt()
        .expect("real failed cleanup was retained")
        .inspection()
        .resources_released());
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: input.scope_key.clone()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        data
    );
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    let mut operation = Some(operation);
    let settled = (0..32)
        .find_map(|_| match operation.take().unwrap().advance(&request) {
            Advance::Progress {
                operation: next, ..
            } => {
                operation = Some(next);
                None
            }
            Advance::Settled(settled) => Some(settled),
            Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
                panic!("original partial progression: {denial:?}")
            }
        })
        .expect("cleanup release leaves the original native partial usable");
    assert!(settled.handler_result().is_none());
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    settled
        .close(&request)
        .unwrap_or_else(|(_, d)| panic!("settled cleanup: {d:?}"));
}
