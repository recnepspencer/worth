use super::fixture::TextWorld;

#[test]
fn settled_text_restoration_prepares_only_the_added_occurrence_at_32_and_512() {
    let small = restore(32);
    let large = restore(512);
    assert_eq!(small.changed_mounted_instances(), 1);
    assert_eq!(large.changed_mounted_instances(), 1);
    assert!(large.index_entries_touched() <= small.index_entries_touched() + 64,
        "a source lookup and persistent index updates may grow with tree depth, not scan 480 additional occurrences: small={small:?}, large={large:?}");
}

fn restore(count: usize) -> crate::mounting::UiMountCostReport {
    let mut world = TextWorld::launch_scaled(count);
    world.execute(&[0, 1], 1, true);
    world.assert_pending(None);
    world.assert_text(0, "AB");
    world.add_occurrence(0, [1210.0, 650.0, 50.0, 25.0]);
    // Geometry admission is setup. Measure the actual publication that must
    // restore text from settled source, before any later interaction frame.
    world.host.push_native_display_presented();
    world.host.push_native_display_settled_without_effects();
    let work = world.execute_scripted(&[0, 1], 2, true);
    world.assert_text(0, "AB");
    world.assert_text(1, "AB");
    world.assert_pending(None);
    let _ = world.session.shutdown();
    assert_eq!(world.host.pending_presentation_count(), 0);
    work
}
