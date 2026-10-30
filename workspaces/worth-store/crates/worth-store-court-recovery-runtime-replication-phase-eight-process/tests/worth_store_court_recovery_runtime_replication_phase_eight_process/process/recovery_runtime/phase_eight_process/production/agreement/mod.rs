use crate::phase_eight_process::production::harness::ProcessWorld;

#[test]
fn production_and_observer_agree_across_real_process_boundary() {
    let world = crate::phase_eight_process::production::harness::ProcessWorld::start(
        "candidate-publication",
        0,
        1,
    );
    let dead_observer = world.observe("dead");
    world
        .writer
        .history
        .compare_report(&dead_observer.report)
        .expect("parent history must match dead-byte observer evidence");
    let runtime = world.recover("production");
    let observer = world.observe("live");
    let history = world.parent_history();
    crate::phase_eight_process::production::harness::compare_runtime_and_observer(
        &runtime, &observer, &history,
    );
    assert_ne!(world.writer.process_id, dead_observer.process_id);
    assert_ne!(world.writer.process_id, runtime.process_id);
    assert_ne!(runtime.process_id, observer.process_id);
}
