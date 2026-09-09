use worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity;

pub(super) fn deregister_one_surface(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    predecessor: worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection,
) {
    let old_surface = predecessor.fragments()[0]
        .work()
        .successor()
        .semantic_surface();
    let UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
        successor: Some(receipt),
        ..
    } = predecessor.fragments()[0].identity()
    else {
        panic!("the original surface must have accepted appearance");
    };
    let graph_node = session
        .mounted
        .current_mounted_identity_basis(receipt.mounted_instance())
        .unwrap()
        .graph_node_identity();
    let survivor_surface = session.create_semantic_surface().unwrap();
    session
        .register_host_surface(
            survivor_surface,
            crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
            crate::facade::mounted::UiSurfaceBindingProfile::new(
                1_000,
                crate::facade::mounted::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                1,
            )
            .unwrap(),
        )
        .unwrap();
    let mounted_node = session.mounted_graph_node(graph_node).unwrap();
    let survivor = session
        .mount_instance(mounted_node, survivor_surface)
        .unwrap();
    crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
        session,
        survivor_surface,
    );
    super::test_support::change_appearance_color(session, 2, "#8090A0");
    publish(session, host, 6);
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    assert_eq!(output.fragments().len(), 2);
    let survivor_receipt = output
        .fragments()
        .iter()
        .find_map(|fragment| match fragment.identity() {
            UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                successor: Some(receipt),
                ..
            } if receipt.mounted_instance() == survivor => Some(receipt),
            _ => None,
        })
        .expect("the second surface must have its own accepted physical predecessor");
    let removed_binding = session
        .inspect_mounted_identity()
        .surface_bindings()
        .iter()
        .find(|binding| binding.semantic_surface_identity() == old_surface)
        .unwrap()
        .binding_generation();
    // Identity-only progression revokes the current projection while retaining
    // accepted appearance. Deregistration must filter that retained predecessor.
    session.advance_mounted_identity_frame().unwrap();
    session.deregister_host_surface(removed_binding).unwrap();
    super::test_support::change_appearance_color(session, 3, "#90A0B0");
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("surviving surface projection must prepare"));
    // The assertion concerns unpublished mounting. Identity-only progression
    // does not authorize changing the surviving host's physical predecessor.
    let output = frame.lower_unpublished_appearance_for_test();
    assert_eq!(output.fragments().len(), 1);
    let fragment = &output.fragments()[0];
    assert_eq!(
        fragment.work().successor().semantic_surface(),
        survivor_surface
    );
    assert!(
        matches!(fragment.identity(), UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
        predecessor: Some(previous), successor: Some(current),
    } if previous == survivor_receipt && current.mounted_instance() == survivor)
    );
    assert_eq!(
        fragment.work().posture(),
        worth_ui_host_contract::UiMountedAppearanceWorkPosture::Delta
    );
    super::test_support::assert_unpublished_surface(&output, [144, 160, 176, 255]);
}

fn publish(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    now: u64,
) {
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("surface transition must prepare"));
    for _ in frame.surfaces() {
        host.push_native_display_settled_without_effects();
    }
    let outcome = session.present_prepared_mounted_frame_internal(
        frame,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
        now,
    );
    match outcome {
        crate::mounting::UiMountedFrameOutcome::Published(_) => {}
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => {
            panic!("surface frame {now} rejected: {:?}", rejected.rejections())
        }
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(denial) => panic!(
            "surface frame {now} admission denied: {:?}",
            denial.denial()
        ),
        crate::mounting::UiMountedFrameOutcome::Unchanged(_) => {
            panic!("surface frame {now} unexpectedly unchanged")
        }
        crate::mounting::UiMountedFrameOutcome::Reconciled(_) => {
            panic!("surface frame {now} unexpectedly reconciled")
        }
        _ => panic!("surface frame {now} did not complete publication"),
    }
}
