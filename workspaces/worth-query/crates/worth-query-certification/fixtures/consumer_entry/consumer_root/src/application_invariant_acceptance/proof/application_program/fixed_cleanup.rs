//! A stopped initial fixed close returns its owner without executing the source.
use super::*;
use crate::application_program::ConsumerFixedAction;
use worth_query_host::facade::application_entry::WorthQueryOperationCloseDenial as Close;
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
