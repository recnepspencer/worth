//! Each request entry preserves its own ceiling despite unrelated noted debt.
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::*;

#[test]
fn registry_eager_completion_notes_without_sweeping_then_admission_prunes() {
    let context = crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::world::Context::new();
    let mut world = World::new(&context, 1);
    let contacts = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::post_commit::sweep_contacts();
    world.registry.complete_prepared_source_for_test(&context.completed);
    assert_eq!(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::post_commit::sweep_contacts(), contacts);
    assert!(
        (*(*world.registry.state_for_test()).lock().unwrap().source_custody[&context.commit(1)]
            .cleanup_for_test())
            .queued
    );
    world.drop_owner(1);
    world.drop_owner(2);
    world.admission().unwrap();
    assert!(!(*world
        .registry
        .state_for_test())
        .lock()
        .unwrap()
        .source_custody
        .contains_key(&context.commit(1)));
    let mut divergences = Vec::new();
    crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::observe::compare(
        &world,
        1,
        1,
        "committed completion drained",
        &mut divergences,
        false,
    );
    assert!(divergences.is_empty(), "{divergences:?}");
}
