pub(super) fn reconstruct_across_source_generation(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    predecessor: worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) {
    let worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
        successor: Some(old_receipt),
        ..
    } = predecessor.fragments()[0].identity()
    else {
        panic!("reconstruction needs a real physical predecessor");
    };
    let previous_generation = session.active_generation_identity();
    let source = super::succession_tests::successor_source(session, role, true);
    super::succession_tests::replace_source(session, host, source, 4, None);
    assert_ne!(session.active_generation_identity(), previous_generation);
    let snapshot = session
        .appearance_owner_snapshot
        .as_ref()
        .expect("accepted succession commits the owner snapshot used for successor pixels");
    assert_eq!(snapshot.generation(), &session.active_generation_identity());
    let accepted = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    super::test_support::assert_unpublished_surface(accepted, [64, 80, 96, 255]);
    let worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
        predecessor: Some(previous),
        successor: Some(accepted_receipt),
    } = accepted.fragments()[0].identity()
    else {
        panic!("accepted replacement must reissue its exact physical receipt");
    };
    assert_eq!(previous, old_receipt);
    assert_ne!(accepted_receipt, old_receipt);
    let frame = session
        .prepare_mounted_reconstruction_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            &[],
            |_| {},
        )
        .unwrap_or_else(|_| panic!("unpublished reconstruction frame must prepare"));
    let physical = frame.lower_unpublished_appearance_for_test();
    assert_eq!(physical.fragments().len(), 1);
    let fragment = &physical.fragments()[0];
    assert!(matches!(fragment.identity(),
        worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            predecessor: Some(receipt), successor: Some(successor),
        } if receipt == accepted_receipt && successor != accepted_receipt));
    assert_eq!(
        fragment.work().posture(),
        worth_ui_host_contract::UiMountedAppearanceWorkPosture::Reconstruction
    );
    assert!(
        fragment.work().changes().is_empty(),
        "same mounted occurrence reissues exact paint across graph identity succession"
    );
    drop(frame);

    // A new owner fact requires semantic refresh during reconstruction. The
    // unchanged role value still requires exact physical reuse after resolving.
    let instance = accepted_receipt.mounted_instance();
    let graph = session
        .mounted
        .current_mounted_identity_basis(instance)
        .unwrap()
        .graph_node_identity();
    let target = crate::runtime::intent::UiAdmittedValidationAppearanceTarget::admit(
        session,
        graph,
        instance,
        accepted_receipt,
    )
    .unwrap();
    session
        .intent_application_facts
        .publish_validation_appearance_fact(
            target,
            None,
            crate::runtime::intent::UiValidationAppearanceClass::Invalid,
        )
        .unwrap();
    let observation = super::succession_tests::successor_source(session, role, true);
    let admitted = {
        let mut turn = session.begin_observation_turn().unwrap();
        turn.admit_source(observation).unwrap();
        turn.seal().unwrap()
    };
    session.classify_observations(admitted).unwrap();
    let frame = session
        .prepare_mounted_reconstruction_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            &[],
            |_| {},
        )
        .unwrap_or_else(|_| panic!("current owners must refresh reconstruction"));
    assert_eq!(
        frame
            .appearance_selection_cost_report()
            .selected_instance_count(),
        1
    );
    frame.verify_unpublished_reconstruction_denial_and_retry();
    let output = frame.lower_unpublished_appearance_for_test();
    assert_eq!(output.fragments().len(), 1);
    let fragment = &output.fragments()[0];
    assert!(matches!(fragment.identity(),
        worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            predecessor: Some(receipt), successor: Some(successor),
        } if receipt == accepted_receipt && successor != accepted_receipt));
    assert_eq!(
        fragment.work().posture(),
        worth_ui_host_contract::UiMountedAppearanceWorkPosture::Reconstruction
    );
    assert!(
        fragment.work().changes().is_empty(),
        "fresh provenance preserves the unchanged mechanics"
    );
    super::test_support::assert_unpublished_surface(&output, [64, 80, 96, 255]);
}
