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
    let maximum_results = std::num::NonZeroUsize::new(query.read_graph().maximum_result_count())
        .expect("the installed TemporalIntent query has a nonzero result cardinality");
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
                    primary_graph::WorthQueryProductQueryControls::new(maximum_results, &request),
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let batch = world
        .application
        .application_query_batch_from_installed_reads(&[world
            .application
            .plan_installed_query_batch_reads(&query, 1, maximum_results)
            .unwrap()])
        .unwrap();
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
}
