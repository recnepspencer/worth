use super::harness::ProcessWorld;

#[test]
fn runtime_and_observer_protocol_families_reject_cross_decoding() {
    let world = ProcessWorld::start("candidate-publication", 0, 1);
    let runtime = world.recover("protocol");
    let observer = world.observe("protocol");
    super::harness::assert_protocol_families_are_distinct(&runtime, &observer);
}
