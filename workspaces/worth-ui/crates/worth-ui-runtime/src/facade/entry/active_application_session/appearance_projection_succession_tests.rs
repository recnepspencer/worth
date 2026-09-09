pub(super) fn retain_across_source_generation(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    predecessor: worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection {
    let worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
        successor: Some(old_receipt),
        ..
    } = predecessor.fragments()[0].identity()
    else {
        panic!("previous frame carried an accepted physical node");
    };
    let instance = old_receipt.mounted_instance();
    let incarnation = session
        .mounted
        .current_mounted_identity_basis(instance)
        .unwrap()
        .mount_incarnation();
    let previous_generation = session.active_generation_identity();
    let previous_graph = session.graph().snapshot().clone();
    let source = successor_source(session, role, true);
    replace_source(session, host, source, 4, None);
    assert_ne!(session.active_generation_identity(), previous_generation);
    assert_eq!(
        session
            .mounted
            .current_mounted_identity_basis(instance)
            .expect("unchanged authored node must retain its mounted instance")
            .mount_incarnation(),
        incarnation
    );
    assert!(session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .is_none());

    let observation = successor_source(session, role, true);
    let admitted = {
        let mut turn = session.begin_observation_turn().unwrap();
        turn.admit_source(observation).unwrap();
        turn.seal().unwrap()
    };
    session.classify_observations(admitted).unwrap();
    assert!(session.has_appearance_owner_snapshot_for_test());
    super::test_support::change_appearance_color(session, 1, "#708090");
    let mut frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("new-generation appearance must prepare"));
    assert_eq!(
        frame.begin_appearance_lifecycle(
            session.session_identity(),
            &session.active_generation_identity(),
            crate::graph::UiGraphAuthority::new(&previous_graph),
        ),
        Err(crate::mounting::UiMountedProjectionDenial::AppearanceSelectionFrameMismatch)
    );
    frame.verify_unpublished_appearance_denial_and_retry();
    host.push_native_display_settled_without_effects();
    let outcome = session.present_prepared_mounted_frame_internal(
        frame,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
        5,
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
    let fragment = &output.fragments()[0];
    assert!(
        matches!(fragment.identity(), worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
        predecessor: Some(receipt), successor: Some(successor),
    } if receipt == old_receipt && successor != old_receipt)
    );
    assert_eq!(
        fragment.work().posture(),
        worth_ui_host_contract::UiMountedAppearanceWorkPosture::Delta
    );
    assert_eq!(fragment.work().changes().len(), 1);
    assert!(matches!(
        fragment.work().changes()[0],
        worth_ui_host_contract::UiMountedAppearanceMechanicChange::Replace { .. }
    ));
    super::test_support::assert_unpublished_surface(output, [112, 128, 144, 255]);
    output.clone()
}

pub(super) fn replace_source(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    submission: crate::runtime::WorthUiWatchedCandidateSubmission,
    now: u64,
    retiring: Option<worth_ui_host_contract::UiMountedNodeReceiptIdentity>,
) {
    let mut prepared = session
        .prepare_replacement(submission)
        .expect("source with an unrelated sibling must prepare");
    let catalog = session
        .admit_native_replacement_allocation_catalog(&mut prepared)
        .expect("successor allocation must come from host measurement");
    let lowered = session.lower_prepared_replacement(*prepared).unwrap();
    let pending = session.stage_prepared_replacement(lowered).unwrap();
    let boundary = session
        .execute_framework_turn(|_| {})
        .unwrap_or_else(|_| panic!("replacement boundary must prepare"))
        .into_completion()
        .into_execution()
        .unwrap_or_else(|_| panic!("replacement boundary must execute"))
        .into_activation_boundary();
    let prepared = session
        .prepare_mounted_replacement(
            pending,
            catalog,
            boundary,
            None,
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
        )
        .unwrap();
    let crate::facade::entry::WorthUiMountedReplacementPreparationOutcome::Prepared(replacement) =
        prepared
    else {
        panic!("changed authored source must not be a semantic no-op");
    };
    if let Some(retiring) = retiring {
        replacement
            .frame()
            .verify_unpublished_appearance_retirement_denial_and_retry(retiring);
    }
    host.push_native_display_settled_without_effects();
    assert!(matches!(
        replacement.present(
            worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
            now,
        ),
        crate::facade::entry::WorthUiMountedApplicationReplacementOutcome::Published { .. }
    ));
}

fn successor_source(
    session: &crate::facade::WorthUiActiveApplicationSession,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    attached: bool,
) -> crate::runtime::WorthUiWatchedCandidateSubmission {
    use crate::runtime::tests::appearance_component_session_test_support as support;
    let sibling = worth_ui_dsl::WorthUiSemanticArtifactDeclaration::new(
        worth_ui_dsl::UiDslSemanticKey::new(support::APPEARANCE_NODE_B),
        worth_ui_dsl::UiDslSemanticFamily::Control,
    )
    .with_structural_token(worth_ui_dsl::UiDslStructuralToken::new(
        "control:appearance-consumer",
    ))
    .with_component_reference(
        worth_ui_dsl::UiDslComponentReference::new(support::APPEARANCE_NODE_B).unwrap(),
    )
    .unwrap();
    let module = worth_ui_dsl::WorthUiRustAuthoredArtifactInputModule::new("appearance/consumer")
        .with_appearance_role(role.clone())
        .with_component(support::APPEARANCE_NODE_A);
    let module = if attached {
        module
            .with_component_appearance_role(
                support::APPEARANCE_NODE_A,
                worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
                    role.role().clone(),
                    role.revision(),
                ),
            )
            .unwrap()
    } else {
        module
    };
    let module = module
        .with_semantic_declaration(sibling)
        .with_token(support::LEGACY_STATIC_PAINT_TOKEN, "#112233");
    crate::runtime::tests::source_ingress_boundary_test_support::lower_rust_submission(
        crate::runtime::WorthUiSourceProvider::rust_authored("appearance-consumer-current")
            .with_rust_authored_input(
                worth_ui_dsl::WorthUiRustAuthoredArtifactInput::from_modules([module]),
            ),
        [crate::runtime::WorthUiWatcherEvent::provider_revision(
            "appearance-consumer-current",
        )],
        session.capabilities(),
    )
}

pub(super) fn detach_role(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    predecessor: worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) {
    let previous = &predecessor.fragments()[0];
    let worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
        successor: Some(receipt),
        ..
    } = previous.identity()
    else {
        panic!("attached role must have produced appearance");
    };
    let instance = receipt.mounted_instance();
    let identity = session
        .mounted
        .current_mounted_identity_basis(instance)
        .unwrap();
    let source = successor_source(session, role, false);
    replace_source(session, host, source, 6, Some(receipt));
    assert_eq!(
        session
            .mounted
            .current_mounted_identity_basis(instance)
            .unwrap()
            .mount_incarnation(),
        identity.mount_incarnation()
    );
    // Observe the replacement itself before any follow-up observation or turn.
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    assert_eq!(output.fragments().len(), 1);
    let removal = &output.fragments()[0];
    assert_eq!(
        removal.identity(),
        worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            predecessor: Some(receipt),
            successor: None,
        }
    );
    assert!(removal.work().successor().mechanics().is_empty());
    assert_eq!(
        removal.work().changes(),
        &[
            worth_ui_host_contract::UiMountedAppearanceMechanicChange::Remove(
                worth_ui_host_contract::UiMountedAppearanceMechanicIdentity::Surface(instance),
            )
        ]
    );
    let worth_ui_host_contract::UiMountedAppearanceMechanic::Surface(surface) =
        &previous.work().successor().mechanics()[0]
    else {
        panic!("the previous role must own a surface");
    };
    let bounds = surface.visual_bounds();
    assert_eq!(removal.work().damage().len(), 1);
    let damage = &removal.work().damage()[0];
    assert_eq!(
        (damage.x(), damage.y(), damage.width(), damage.height()),
        (bounds.x(), bounds.y(), bounds.width(), bounds.height())
    );
    worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
    host.push_native_display_settled_without_effects();
    super::test_support::publish_without_selected_appearance(session, 7);
    assert!(session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .is_none());
}

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
    let source = successor_source(session, role, true);
    replace_source(session, host, source, 4, None);
    assert_ne!(session.active_generation_identity(), previous_generation);
    assert!(!session.has_appearance_owner_snapshot_for_test());
    let frame = session
        .prepare_mounted_reconstruction_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            &[],
            |_| {},
        )
        .unwrap_or_else(|_| panic!("unpublished reconstruction frame must prepare"));
    frame.verify_unpublished_reconstruction_needs_current_projection();
    drop(frame);

    // A real observation close queues the successor's canonical initial
    // selection. No theme mutation is used to force the refresh.
    let observation = successor_source(session, role, true);
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
        } if receipt == old_receipt && successor != old_receipt));
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
