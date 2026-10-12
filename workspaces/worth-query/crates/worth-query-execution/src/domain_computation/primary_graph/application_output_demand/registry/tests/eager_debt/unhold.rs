//! Cancellation and publication use genuine prepared tickets and settlement identity.
use super::*;
pub(super) fn run(context: &world::Context, seed: u64, divergences: &mut Vec<String>) {
    let mut world = World::new(context, seed & !3);
    let (lineage, identity) = crate::domain_computation::primary_graph::output_lineage
        ::registry_fixture::recorded_settlement();
    let cancel = seed & 1 == 0;
    {
        let mut state = world.registry.state.lock().unwrap();
        state
            .records
            .get_mut(&world.keys[2])
            .unwrap()
            .required_interests = 1;
        world.owners[2].as_mut().unwrap().requires_output = true;
    }
    super::super::super::prerequisite_claims::custody_note_steps::unhold(
        world.owners[2].as_ref().unwrap(),
        cancel,
        &identity,
    );
    world.model.rows.get_mut(&2).unwrap().phase = Phase::Running;
    world.model.release_claims(2);
    observe::compare(
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
    world.registry.finish_scheduling(
        world.owners[2].as_ref().unwrap(),
        None,
        &mut Err(WorthQueryOutputDemandDenial::new(
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
    world.registry.release_prepared_token(&context.commit(1));
    world.model.settle();
    for _ in 0..4 {
        world.admission(1_000_000).unwrap();
    }
    observe::compare(
        &world,
        seed,
        102,
        "unhold quiescent value",
        divergences,
        true,
    );
    drop(lineage);
}
