use worth_ui_host_contract::*;

pub(super) fn project(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    expected: &[(UiMountedInstanceIdentity, u8)],
    pointers: &[(
        UiSemanticSurfaceIdentity,
        Option<(UiMountedInstanceIdentity, UiPointerAffordanceFamily)>,
    )],
) -> crate::mounting::UiPreparedMountedFrame {
    let frame = super::prepare(session);
    let cost = frame.appearance_selection_cost_report();
    assert_eq!(cost.selected_instance_count(), expected.len());
    assert_eq!(cost.materialized_context_count(), expected.len());
    assert_eq!(cost.index_entries_touched(), expected.len());
    let output = frame.lower_unpublished_appearance_for_test();
    assert_eq!(output.fragments().len(), expected.len() + pointers.len());
    let mut nodes_seen = Vec::new();
    let mut pointers_seen = 0;
    for fragment in output.fragments() {
        match fragment.identity() {
            UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                successor: Some(receipt),
                ..
            } => {
                nodes_seen.push(receipt.mounted_instance());
                let red = expected
                    .iter()
                    .find(|(instance, _)| *instance == receipt.mounted_instance())
                    .expect("unrelated neighborhood received work")
                    .1;
                let [UiMountedAppearanceMechanic::Surface(mechanic)] =
                    fragment.work().successor().mechanics()
                else {
                    panic!("one background mechanic");
                };
                assert_eq!(
                    mechanic.paint(),
                    &UiMountedSurfacePaint::Fill(UiMountedAppearanceColor::from_straight_srgba([
                        red, 0, 0, 255
                    ]))
                );
            }
            UiUnpublishedAppearanceFragmentIdentity::SurfacePointer { surface, pointer } => {
                pointers_seen += 1;
                assert_eq!(pointer, UiHostPointerIdentity::new(1));
                let expected = pointers
                    .iter()
                    .find(|(expected, _)| *expected == surface)
                    .expect("unrelated pointer surface received work")
                    .1;
                match expected {
                    Some((target, family)) => {
                        let [UiMountedAppearanceMechanic::Pointer(mechanic)] =
                            fragment.work().successor().mechanics()
                        else {
                            panic!("one independent pointer mechanic");
                        };
                        assert_eq!(mechanic.surface(), surface);
                        assert_eq!(mechanic.pointer(), pointer);
                        assert_eq!(mechanic.target(), target);
                        assert_eq!(mechanic.family(), family);
                    }
                    None => assert!(fragment.work().successor().mechanics().is_empty()),
                }
            }
            _ => panic!("unexpected fragment family"),
        }
    }
    nodes_seen.sort_unstable();
    let mut expected_nodes = expected
        .iter()
        .map(|(target, _)| *target)
        .collect::<Vec<_>>();
    expected_nodes.sort_unstable();
    assert_eq!(nodes_seen, expected_nodes);
    assert_eq!(pointers_seen, pointers.len());
    frame
}
