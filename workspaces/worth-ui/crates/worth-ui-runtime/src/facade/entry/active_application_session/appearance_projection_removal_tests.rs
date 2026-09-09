use worth_ui_host_contract::{
    UiMountedAppearanceMechanic, UiMountedAppearanceMechanicChange, UiMountedAppearanceWorkPosture,
    UiUnpublishedAppearanceFragmentIdentity,
};

pub(super) fn remove_last_nodes_without_a_theme_change(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    predecessor: worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection,
) {
    let previous = &predecessor.fragments()[0];
    let UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
        successor: Some(receipt),
        ..
    } = previous.identity()
    else {
        panic!("the real theme delta must carry its physical successor");
    };
    let instances = session
        .current_mounted_projection_rc_for_test()
        .unwrap()
        .mounted_instances()
        .collect::<Vec<_>>();
    assert!(instances.contains(&receipt.mounted_instance()));
    for instance in instances {
        session.unmount_instance(instance).unwrap();
    }
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("the removal-only frame must prepare"));
    assert!(frame.appearance_invalidation_batch().is_none());
    assert_eq!(
        frame
            .appearance_selection_cost_report()
            .selected_instance_count(),
        0
    );
    frame.verify_unpublished_appearance_retirement_denial_and_retry(receipt);
    host.push_native_display_settled_without_effects();
    let outcome = session.present_prepared_mounted_frame_internal(
        frame,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
        6,
    );
    assert!(matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    assert_eq!(output.fragments().len(), 1);
    let removal = &output.fragments()[0];
    assert_eq!(
        removal.identity(),
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            predecessor: Some(receipt),
            successor: None,
        }
    );
    assert_eq!(removal.presentation_affinity().receipt_affinity(), None);
    assert_eq!(
        removal.work().posture(),
        UiMountedAppearanceWorkPosture::Delta
    );
    assert!(removal.work().successor().mechanics().is_empty());
    assert_eq!(
        removal.work().changes().len(),
        previous.work().successor().mechanics().len()
    );
    for mechanic in previous.work().successor().mechanics() {
        let UiMountedAppearanceMechanic::Surface(surface) = mechanic else {
            panic!("fixture owns one real surface fill");
        };
        assert!(removal
            .work()
            .changes()
            .contains(&UiMountedAppearanceMechanicChange::Remove(
                worth_ui_host_contract::UiMountedAppearanceMechanicIdentity::Surface(
                    receipt.mounted_instance()
                ),
            )));
        let bounds = surface.visual_bounds();
        assert!(removal.work().damage().iter().any(|damage| {
            (damage.x(), damage.y(), damage.width(), damage.height())
                == (bounds.x(), bounds.y(), bounds.width(), bounds.height())
        }));
    }
    worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
    host.push_native_display_settled_without_effects();
    super::test_support::publish_without_selected_appearance(session, 7);
    assert!(session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .is_none());
}
