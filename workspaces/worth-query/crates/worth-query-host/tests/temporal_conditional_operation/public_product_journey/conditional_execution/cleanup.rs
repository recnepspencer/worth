//! The public journey releases every conditional resource after its final delivery.
use super::*;
pub(super) fn assert_closed(
    mut world: CourtroomWorld,
    probe: &primary_graph::WorthQueryConditionalRuntimeLifecycleProbe,
) {
    world.application.close_conditional_runtime().unwrap();
    assert_conditional_resources_empty(world.application.inspect_conditional_runtime());
    drop(world);
    let live = probe.live_inventory();
    assert!(
        live.is_empty(),
        "live conditional inventory after cleanup: {live:?}"
    );
}
