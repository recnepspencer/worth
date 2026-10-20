use super::*;

#[test]
fn absent_request_cannot_widen_a_finite_slot_or_consume_its_retry() {
    use crate::domain_computation::primary_graph::tests::fixture::{
        installed_authorization_world, TestAccountSourceBinding,
    };
    let world = installed_authorization_world(true);
    let binding = world
        .application
        .installed_schema()
        .installed_query_binding::<TestAccountSourceBinding>()
        .unwrap();
    let read = world
        .application
        .plan_application_query_batch_reads(&binding, 1)
        .unwrap();
    let runtime = read.runtime;
    let schema = read.schema.clone();
    let query = read.query.clone();
    let maximum = read.work.expect("fixture installs a finite query ceiling");
    let roots = read.roots;
    let batch = world
        .application
        .application_query_batch_from_installed_reads(&[read])
        .unwrap();
    assert!(
        matches!(batch.take_planned_read(runtime, &schema, &query, None, roots),
        Err(WorthQueryApplicationQueryBatchResourceDenial::WorkLimitWidening { maximum: actual }) if actual == maximum)
    );
    assert!(batch
        .take_planned_read(runtime, &schema, &query, Some(maximum), roots)
        .unwrap()
        .is_some());
}
