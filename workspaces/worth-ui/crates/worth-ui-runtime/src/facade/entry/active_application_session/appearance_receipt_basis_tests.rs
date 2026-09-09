use super::support;

#[test]
fn new_instance_has_no_presented_owner_while_existing_instance_keeps_its_receipt() {
    use crate::mounting::UiMountedAppearanceReceiptBasisDenial as Denial;
    let role = super::single_aspect_role(
        "test.new-instance-basis",
        worth_ui_dsl::UiAppearanceAspect::Background,
        support::APPEARANCE_TOKEN,
    );
    let mut fixture = super::mounted_fixture(&role, &[], false);
    let identity = fixture.session.inspect_mounted_identity();
    let existing = identity
        .mounted_instances()
        .iter()
        .find(|row| row.graph_node_identity() == fixture.graph_node)
        .unwrap();
    let handle = fixture
        .session
        .mounted_graph_node(fixture.graph_node)
        .unwrap();
    let instance = fixture
        .session
        .mount_instance(handle, fixture.surface)
        .unwrap();
    crate::facade::entry::mounted_occurrence_geometry_test_support::refresh_nonoverlapping_surface_geometry(
        &mut fixture.session,
        fixture.surface,
    );
    let mounted_identity = fixture
        .session
        .mounted
        .current_mounted_identity_basis(instance)
        .unwrap();
    let frame = fixture
        .session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| {
            panic!("new mounted instance should prepare through the ordinary path")
        });
    let incarnation = mounted_identity.mount_incarnation();
    let candidate = frame.presented_receipt_basis();
    let basis = fixture
        .session
        .mounted
        .seal_appearance_receipt_basis(instance, incarnation, candidate)
        .unwrap();
    assert!(basis.owner_node_receipt().is_none());
    let existing_basis = fixture
        .session
        .mounted
        .seal_appearance_receipt_basis(existing.identity(), existing.mount_incarnation(), candidate)
        .unwrap();
    assert!(existing_basis.owner_node_receipt().is_some());
    let input = crate::runtime::appearance::UiAppearanceCoherentBasisInput {
        mounted_identity,
        mounted_instance: instance,
        receipt_basis: basis,
        theme: fixture
            .session
            .presentation
            .active_appearance_theme_binding(fixture.surface)
            .cloned()
            .unwrap(),
        presentation: None,
        selection: None,
        operability_route: None,
    };
    let consumer =
        crate::runtime::appearance::UiAppearanceStateConsumer::from_role(fixture.graph_node, &role);
    let snapshot = fixture
        .session
        .appearance_owner_snapshot_for_test()
        .unwrap();
    let themes = fixture
        .session
        .presentation
        .appearance_theme_state()
        .unwrap();
    assert!(
        crate::runtime::appearance::UiAppearanceCoherentBasis::admit_prepared(
            &frame,
            snapshot,
            &consumer,
            &fixture.session.mounted,
            themes,
            input.clone(),
        )
        .is_ok()
    );
    assert_eq!(
        crate::runtime::appearance::UiAppearanceCoherentBasis::admit_current(
            snapshot,
            &consumer,
            &fixture.session.mounted,
            themes,
            input,
        ),
        Err(crate::runtime::appearance::UiAppearanceCoherentBasisDenial::MountedTargetNotCurrent)
    );
    assert_eq!(
        fixture.session.mounted.seal_appearance_receipt_basis(
            instance,
            worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap(),
            candidate,
        ),
        Err(Denial::CurrentIncarnationMismatch)
    );
    assert_eq!(
        fixture.session.mounted.seal_appearance_receipt_basis(
            worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap(),
            incarnation,
            candidate,
        ),
        Err(Denial::CurrentInstanceUnavailable)
    );
    drop(frame);
    fixture.session.unmount_instance(instance).unwrap();
    assert_eq!(
        fixture
            .session
            .mounted
            .validate_appearance_receipt_basis(basis),
        Err(Denial::CurrentInstanceUnavailable)
    );
    let _ = fixture.session.shutdown();
}

#[test]
fn mounted_appearance_basis_separates_owner_and_successor_authority() {
    let role = super::single_aspect_role(
        "test.receipt-basis",
        worth_ui_dsl::UiAppearanceAspect::Background,
        support::APPEARANCE_TOKEN,
    );
    let mut fixture = super::mounted_fixture(&role, &[], false);
    super::update_theme(&mut fixture.session, "#405060");
    super::publish_frame(&mut fixture.session, 0);
    let (instance, incarnation, mounted_identity) = {
        let identity = fixture.session.inspect_mounted_identity();
        let row = identity
            .mounted_instances()
            .iter()
            .find(|row| row.graph_node_identity() == fixture.graph_node)
            .unwrap();
        (row.identity(), row.mount_incarnation(), row.basis().clone())
    };

    let frame = fixture
        .session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("receipt candidate prepares"));

    let candidate_basis = |instance| {
        let mut presented = crate::runtime::persistent_index::UiPersistentOrdSet::default();
        presented.insert(instance);
        crate::mounting::UiMountedNodeReceiptBasis::mint(
            worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
            presented,
        )
        .unwrap()
    };
    let successor = candidate_basis(instance);
    let receipt_basis = fixture
        .session
        .mounted
        .seal_appearance_receipt_basis(instance, incarnation, &successor)
        .unwrap();
    assert_ne!(
        receipt_basis.owner_node_receipt(),
        Some(receipt_basis.successor_node_receipt())
    );

    let target = crate::runtime::appearance::UiAppearanceTarget::new(
        fixture.session.session_identity(),
        fixture.surface,
        fixture.graph_node,
        instance,
        incarnation,
        receipt_basis.successor_node_receipt(),
    )
    .unwrap();
    let binding = crate::runtime::appearance::UiAppearanceNodeRoleBinding::from_current_graph(
        fixture.session.graph().snapshot(),
        fixture.session.capabilities(),
        &target,
    )
    .unwrap();
    let consumer = crate::runtime::appearance::UiAppearanceStateConsumer::from_role(
        fixture.graph_node,
        binding.role(),
    );
    let input = crate::runtime::appearance::UiAppearanceCoherentBasisInput {
        mounted_identity,
        mounted_instance: instance,
        receipt_basis,
        theme: fixture
            .session
            .presentation
            .active_appearance_theme_binding(fixture.surface)
            .cloned()
            .unwrap(),
        presentation: None,
        selection: None,
        operability_route: None,
    };
    let admitted = crate::runtime::appearance::UiAppearanceCoherentBasis::admit_prepared(
        &frame,
        fixture
            .session
            .appearance_owner_snapshot_for_test()
            .unwrap(),
        &consumer,
        &fixture.session.mounted,
        fixture
            .session
            .presentation
            .appearance_theme_state()
            .unwrap(),
        input.clone(),
    )
    .unwrap();
    assert_eq!(
        admitted.owner_node_receipt(),
        receipt_basis.owner_node_receipt()
    );
    assert_eq!(
        admitted.node_receipt(),
        receipt_basis.successor_node_receipt()
    );
    let foreign_instance =
        worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let foreign_successor = candidate_basis(foreign_instance);
    assert_eq!(
        fixture.session.mounted.seal_appearance_receipt_basis(
            instance,
            incarnation,
            &foreign_successor,
        ),
        Err(crate::mounting::UiMountedAppearanceReceiptBasisDenial::SuccessorNotPresented)
    );

    let foreign_fixture = super::mounted_fixture(&role, &[], false);
    let (foreign_instance, foreign_incarnation) = {
        let identity = foreign_fixture.session.inspect_mounted_identity();
        let row = identity
            .mounted_instances()
            .iter()
            .find(|row| row.graph_node_identity() == foreign_fixture.graph_node)
            .unwrap();
        (row.identity(), row.mount_incarnation())
    };
    let foreign_basis = foreign_fixture
        .session
        .mounted
        .seal_appearance_receipt_basis(
            foreign_instance,
            foreign_incarnation,
            &candidate_basis(foreign_instance),
        )
        .unwrap();
    let mut foreign_input = input;
    foreign_input.receipt_basis = foreign_basis;
    assert_eq!(
        crate::runtime::appearance::UiAppearanceCoherentBasis::admit_prepared(
            &frame,
            fixture
                .session
                .appearance_owner_snapshot_for_test()
                .unwrap(),
            &consumer,
            &fixture.session.mounted,
            fixture
                .session
                .presentation
                .appearance_theme_state()
                .unwrap(),
            foreign_input,
        ),
        Err(crate::runtime::appearance::UiAppearanceCoherentBasisDenial::MountedTargetNotCurrent)
    );

    super::publish_frame(&mut fixture.session, 1);
    assert_eq!(
        crate::runtime::appearance::UiAppearanceCoherentBasis::admit_prepared(
            &frame,
            fixture
                .session
                .appearance_owner_snapshot_for_test()
                .unwrap(),
            &consumer,
            &fixture.session.mounted,
            fixture
                .session
                .presentation
                .appearance_theme_state()
                .unwrap(),
            crate::runtime::appearance::UiAppearanceCoherentBasisInput {
                mounted_identity: fixture
                    .session
                    .mounted
                    .current_mounted_identity_basis(instance)
                    .unwrap(),
                mounted_instance: instance,
                receipt_basis,
                theme: fixture
                    .session
                    .presentation
                    .active_appearance_theme_binding(fixture.surface)
                    .cloned()
                    .unwrap(),
                presentation: None,
                selection: None,
                operability_route: None,
            },
        ),
        Err(crate::runtime::appearance::UiAppearanceCoherentBasisDenial::MountedTargetNotCurrent)
    );
    let _ = foreign_fixture.session.shutdown();
    let _ = fixture.session.shutdown();
}
