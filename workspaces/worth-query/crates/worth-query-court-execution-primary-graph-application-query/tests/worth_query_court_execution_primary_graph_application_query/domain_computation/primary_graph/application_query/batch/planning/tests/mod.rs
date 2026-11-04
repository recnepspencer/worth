use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryBatchResourceDenial;
#[test]
fn installed_schedule_preserves_exact_slots_and_plan_association() {
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::current_output_source::TestAccountSourceBinding;
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
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
    let runtime = (*read.runtime_for_test());
    let schema = (*read.schema_for_test()).clone();
    let query = (*read.query_for_test()).clone();
    let roots = (*read.roots_for_test());
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
        .take_planned_read_for_test(runtime, &schema, &query, roots)
        .unwrap()
        .unwrap();
    assert!(item.belongs_to(&batch));
    assert!(!item.belongs_to(&foreign));
    assert!(matches!(
        batch.take_planned_read_for_test(runtime, &schema, &query, roots),
        Err(WorthQueryApplicationQueryBatchResourceDenial::UnplannedRead)
    ));
}
