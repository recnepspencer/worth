//! Cancellation and publication use genuine prepared tickets and settlement identity.
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::*;
pub fn run(
    context: &crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::world::Context,
    seed: u64,
    divergences: &mut Vec<String>,
) {
    let mut world = World::new(context, seed & !3);
    let (lineage, identity) = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    let cancel = seed & 1 == 0;
    {
        let mut state = (*world.registry.state_for_test()).lock().unwrap();
        (*(*state.records_mut_for_test())
            .get_mut(&world.keys[2])
            .unwrap()
            .required_interests_mut_for_test()) = 1;
        (*world.owners[2]
            .as_mut()
            .unwrap()
            .requires_output_mut_for_test()) = true;
    }
    worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_output_demand::registry::prerequisite_claims::custody_note_steps::unhold(
        world.owners[2].as_ref().unwrap(),
        cancel,
        &identity,
    );
    world.model.rows.get_mut(&2).unwrap().phase = Phase::Running;
    world.model.release_claims(2);
    crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::observe::compare(
        &world,
        seed,
        101,
        if cancel {
            "prepared prerequisite cancel"
        } else {
            "prepared prerequisite publish"
        },
        divergences,
        false,
    );
    world.registry.finish_scheduling_for_test(
        world.owners[2].as_ref().unwrap(),
        None,
        &mut Err(WorthQueryOutputDemandDenial::new_for_test(
            WorthQueryOutputDemandDenialKind::Superseded,
            "unhold fixture finished",
        )),
    );
    world.model.rows.get_mut(&2).unwrap().phase = Phase::Superseded;
    for id in 0..world.owners.len() {
        world.drop_owner(id);
    }
    if let Some(entry) = world.model.custody.get_mut(&1) {
        entry.tokens = 0;
    }
    world
        .registry
        .release_prepared_token_for_test(&context.commit(1));
    world.model.settle();
    for _ in 0..4 {
        world.admission().unwrap();
    }
    crate::domain_computation::primary_graph::application_output_demand::registry::tests::eager_debt::observe::compare(
        &world,
        seed,
        102,
        "unhold quiescent value",
        divergences,
        true,
    );
    drop(lineage);
}
