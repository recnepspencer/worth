//! pre kernel cancellation keeps its real execution cause.
use super::*;

#[test]
fn pre_kernel_cancellation_keeps_its_real_execution_cause() {
    use super::super::{product_query_support, schema::*};
    use worth_query_host::facade::{
        admission::authenticated_principal::*,
        declaration::application_query::ApplicationQueryParameterSet, primary_graph,
    };
    let world = CourtroomWorld::publish("ready");
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(std::time::Instant::now() + std::time::Duration::from_secs(60)),
        cancellation.token(),
    );
    let principal = product_query_support::principal(&world, &request);
    let query = world
        .application
        .installed_schema()
        .certification_query(TemporalIntentQuery::reference())
        .unwrap();
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let scope = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    IntentIdentityField::reference(),
                    "intent-1".to_owned(),
                    &request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let access = primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let plan = world
        .application
        .with_host_advancement(|phase| {
            selected
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    product_query_support::controls(&request),
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let batch =
        primary_graph::WorthQueryApplicationQueryBatchAdmission::new(limits(None, 1_048_576));
    cancellation.cancel();
    let result = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot_in_batch(&phase, plan, &batch)
        })
        .expect("installed host call opens");
    assert!(
        matches!(result, Err(primary_graph::WorthQueryApplicationBatchReadDenial::Execution(error)) if error.kind() == primary_graph::WorthQueryApplicationOneShotDenialKind::Cancelled)
    );
    assert_eq!(batch.observe().retained_bytes(), 0);
    assert_eq!(
        world
            .application
            .result_buffer_observer()
            .observe()
            .active_buffers(),
        0
    );
}
