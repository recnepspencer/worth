//! Each request entry preserves its own ceiling despite unrelated noted debt.
use super::*;

#[test]
fn registry_eager_completion_notes_without_sweeping_then_admission_prunes() {
    let context = world::Context::new();
    let mut world = World::new(&context, 1);
    let contacts = super::super::super::post_commit::sweep_contacts();
    world.registry.complete_prepared_source(&context.completed);
    assert_eq!(super::super::super::post_commit::sweep_contacts(), contacts);
    assert!(
        world.registry.state.lock().unwrap().source_custody[&context.commit(1)]
            .cleanup
            .queued
    );
    world.drop_owner(1);
    world.drop_owner(2);
    world.admission().unwrap();
    assert!(!world
        .registry
        .state
        .lock()
        .unwrap()
        .source_custody
        .contains_key(&context.commit(1)));
    let mut divergences = Vec::new();
    observe::compare(
        &world,
        1,
        1,
        "committed completion drained",
        &mut divergences,
        false,
    );
    assert!(divergences.is_empty(), "{divergences:?}");
}
