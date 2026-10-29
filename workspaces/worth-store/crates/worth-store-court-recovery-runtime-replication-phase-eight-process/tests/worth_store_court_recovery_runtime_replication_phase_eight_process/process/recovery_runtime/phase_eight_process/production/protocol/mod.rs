use {crate::phase_eight_process::production::harness::ProcessWorld};

#[test]
fn runtime_and_observer_protocol_families_reject_cross_decoding() {
    let world = crate::phase_eight_process::production::harness::ProcessWorld::start("candidate-publication", 0, 1);
    let runtime = world.recover("protocol");
    let observer = world.observe("protocol");
    crate::phase_eight_process::production::harness::assert_protocol_families_are_distinct(&runtime, &observer);
}
