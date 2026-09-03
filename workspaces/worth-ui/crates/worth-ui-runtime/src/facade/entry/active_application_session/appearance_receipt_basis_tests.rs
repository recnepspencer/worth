use super::support;

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
        receipt_basis.successor_node_receipt()
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
