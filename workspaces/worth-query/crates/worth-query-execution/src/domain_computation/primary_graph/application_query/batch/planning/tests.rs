use super::*;
#[test]
fn installed_schedule_preserves_exact_slots_and_plan_association() {
    use crate::domain_computation::primary_graph::tests::fixture::{
        installed_authorization_world, TestAccountSourceBinding,
    };
    let world = installed_authorization_world(true);
    let binding = world
        .application
        .installed_schema()
        .installed_query_binding::<TestAccountSourceBinding>()
        .unwrap();
    let issue = || {
        world
            .application
            .plan_application_query_batch_reads(&binding, 1)
            .unwrap()
    };
    let read = issue();
    let runtime = read.runtime;
    let schema = read.schema.clone();
    let query = read.query.clone();
    let roots = read.roots;
    let batch = world
        .application
        .application_query_batch_from_installed_reads(&[read])
        .unwrap();
    let foreign = world
        .application
        .application_query_batch_from_installed_reads(&[issue()])
        .unwrap();
    batch.freeze_planned_reads().unwrap();
    assert!(matches!(
        world
            .application
            .extend_application_query_batch_once(&batch, &[issue()]),
        Err(WorthQueryApplicationQueryBatchResourceDenial::PlanningFrozen)
    ));
    let item = batch
        .take_planned_read(runtime, &schema, &query, roots)
        .unwrap()
        .unwrap();
    assert!(item.belongs_to(&batch));
    assert!(!item.belongs_to(&foreign));
    assert!(matches!(
        batch.take_planned_read(runtime, &schema, &query, roots),
        Err(WorthQueryApplicationQueryBatchResourceDenial::UnplannedRead)
    ));
}
