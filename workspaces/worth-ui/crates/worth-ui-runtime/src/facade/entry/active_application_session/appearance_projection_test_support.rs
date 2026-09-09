use super::support;

pub(super) fn change_appearance_color(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    expected_revision: u64,
    color: &str,
) {
    let token = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
    let value = crate::capability::ThemeTokenValue::color(
        crate::capability::ThemeColorValue::hex(color).unwrap(),
    );
    let change = super::super::super::UiNativeThemeTokenValueChange::successor(
        token,
        expected_revision,
        value,
    )
    .unwrap();
    session.admit_application_theme_values(&[change]).unwrap();
}

pub(super) fn assert_unpublished_surface(
    projection: &worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection,
    expected_color: [u8; 4],
) {
    assert_unpublished_surface_with_pointer(projection, expected_color, None);
}

pub(super) fn assert_unpublished_surface_with_pointer(
    projection: &worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection,
    expected_color: [u8; 4],
    pointer: Option<(
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        worth_ui_host_contract::UiMountedInstanceIdentity,
        bool,
    )>,
) {
    let transcript =
        worth_ui_host_headless::translate_unpublished_appearance_for_certification(projection)
            .expect("headless consumes the production mounting output");
    assert_eq!(transcript.frame(), projection.frame());
    assert_eq!(transcript.presentation(), projection.presentation());
    assert_eq!(
        projection.fragments().len(),
        1 + usize::from(pointer.is_some())
    );
    assert_eq!(transcript.fragments().len(), projection.fragments().len());
    if let Some((surface, target, present)) = pointer {
        use worth_ui_host_contract::*;
        let fragment = &projection.fragments()[1];
        assert_eq!(
            fragment.identity(),
            UiUnpublishedAppearanceFragmentIdentity::SurfacePointer {
                surface,
                pointer: UiHostPointerIdentity::new(1),
            }
        );
        assert_eq!(transcript.fragments()[1].identity(), fragment.identity());
        if present {
            let [UiMountedAppearanceMechanic::Pointer(mechanic)] =
                fragment.work().successor().mechanics()
            else {
                panic!("one independent pointer mechanic");
            };
            assert_eq!(
                *mechanic,
                UiMountedPointerAffordanceMechanic::complete_from_runtime_mounting(
                    UiHostPointerIdentity::new(1),
                    surface,
                    target,
                    UiPointerAffordanceFamily::Default
                )
            );
        } else {
            assert!(fragment.work().successor().mechanics().is_empty());
            assert_eq!(
                fragment.work().changes(),
                &[UiMountedAppearanceMechanicChange::Remove(
                    UiMountedAppearanceMechanicIdentity::Pointer {
                        pointer: UiHostPointerIdentity::new(1),
                        surface,
                        target
                    }
                )]
            );
        }
    }
    let fragment = &projection.fragments()[0];
    let observed = &transcript.fragments()[0];
    assert_eq!(observed.identity(), fragment.identity());
    assert_eq!(observed.surface_binding(), fragment.surface_binding());
    assert_eq!(
        observed.presentation_affinity(),
        fragment.presentation_affinity()
    );
    assert_eq!(observed.work().posture(), fragment.work().posture());
    assert_eq!(observed.work().damage(), fragment.work().damage());
    assert_eq!(fragment.work().successor().mechanics().len(), 1);
    let worth_ui_host_contract::UiMountedAppearanceMechanic::Surface(mechanic) =
        &fragment.work().successor().mechanics()[0]
    else {
        panic!("the authored background must produce a surface mechanic");
    };
    assert_eq!(mechanic.surface_paint_order(), 65_536);
    assert_eq!(
        mechanic.paint(),
        &worth_ui_host_contract::UiMountedSurfacePaint::Fill(
            worth_ui_host_contract::UiMountedAppearanceColor::from_straight_srgba(expected_color),
        )
    );
    assert_eq!(
        observed.work().successor().mechanics(),
        &[worth_ui_host_headless::UiHeadlessAppearanceMechanic::Surface(mechanic.clone())]
    );
}

pub(super) fn theme_session(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    let host_observer = host.clone();
    host.set_capabilities(worth_ui_host_native::staged_appearance_capability_report());
    let session = support::legacy_static_paint_appearance_component_builder(role)
        .register_appearance_theme_bundle(theme_bundle())
        .unwrap()
        .with_rust_authored_declaration_fixture(support::appearance_fixture(role))
        .freeze()
        .map(|application| {
            host.push_native_display_presented();
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                application,
                host,
            )
        })
        .expect("appearance capability fixture should prepare")
        .launch()
        .expect("appearance capability fixture should launch");
    (session, host_observer)
}

pub(super) fn publish_initial_appearance(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    graph_node: crate::graph::UiGraphNodeIdentity,
    now: u64,
) {
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("first appearance frame should prepare"));
    let identity = session.inspect_mounted_identity();
    let instance = identity
        .mounted_instances()
        .iter()
        .find(|row| row.graph_node_identity() == graph_node)
        .unwrap();
    let receipt_basis = session
        .mounted
        .seal_appearance_receipt_basis(
            instance.identity(),
            instance.mount_incarnation(),
            frame.presented_receipt_basis(),
        )
        .unwrap();
    assert_eq!(receipt_basis.owner_node_receipt(), None);
    let consumer =
        crate::runtime::appearance::UiAppearanceStateConsumer::from_role(graph_node, role);
    let input = crate::runtime::appearance::UiAppearanceCoherentBasisInput {
        mounted_identity: instance.basis().clone(),
        mounted_instance: instance.identity(),
        receipt_basis,
        theme: session
            .presentation
            .active_appearance_theme_binding(instance.basis().semantic_surface_identity())
            .cloned()
            .unwrap(),
        presentation: None,
        selection: None,
        operability_route: None,
    };
    let snapshot = session.appearance_owner_snapshot_for_test().unwrap();
    let themes = session.presentation.appearance_theme_state().unwrap();
    assert_eq!(
        crate::runtime::appearance::UiAppearanceCoherentBasis::admit_current(
            snapshot,
            &consumer,
            &session.mounted,
            themes,
            input.clone(),
        ),
        Err(crate::runtime::appearance::UiAppearanceCoherentBasisDenial::MountedTargetNotCurrent)
    );
    let basis = crate::runtime::appearance::UiAppearanceCoherentBasis::admit_prepared(
        &frame,
        snapshot,
        &consumer,
        &session.mounted,
        themes,
        input.clone(),
    )
    .unwrap();
    let vector =
        crate::runtime::appearance::UiAppearanceStateVector::seal(snapshot, &basis).unwrap();
    let expected = [
        (
            worth_ui_dsl::UiAppearanceStateAxis::Validation,
            worth_ui_dsl::UiAppearanceAxisClass::ValidationUnspecified,
        ),
        (
            worth_ui_dsl::UiAppearanceStateAxis::Hover,
            worth_ui_dsl::UiAppearanceAxisClass::HoverOutside,
        ),
        (
            worth_ui_dsl::UiAppearanceStateAxis::Pressed,
            worth_ui_dsl::UiAppearanceAxisClass::PressedIdle,
        ),
    ];
    assert_eq!(vector.classes().count(), 1);
    for (axis, class) in expected {
        if consumer.consumes(axis) {
            assert_eq!(vector.class(axis), Some(class));
        }
    }
    let outcome = session.present_prepared_mounted_frame_internal(
        frame,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
        now,
    );
    assert!(matches!(
        outcome,
        crate::mounting::UiMountedFrameOutcome::Published(_)
    ));
    assert_eq!(
        session
            .mounted
            .validate_appearance_receipt_basis(receipt_basis),
        Err(crate::mounting::UiMountedAppearanceReceiptBasisDenial::OwnerBasisChanged)
    );
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("successor candidate prepares"));
    let current_theme = session
        .presentation
        .active_appearance_theme_binding(basis.surface())
        .cloned()
        .unwrap();
    assert_eq!(
        crate::runtime::appearance::UiAppearanceCoherentBasis::admit_prepared(
            &frame,
            session.appearance_owner_snapshot_for_test().unwrap(),
            &consumer,
            &session.mounted,
            session.presentation.appearance_theme_state().unwrap(),
            crate::runtime::appearance::UiAppearanceCoherentBasisInput {
                theme: current_theme,
                ..input
            },
        ),
        Err(crate::runtime::appearance::UiAppearanceCoherentBasisDenial::MountedTargetNotCurrent)
    );
}

pub(super) fn publish_without_selected_appearance(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    now: u64,
) {
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("the next application frame should prepare"));
    let outcome = session.present_prepared_mounted_frame_internal(
        frame,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
        now,
    );
    match outcome {
        crate::mounting::UiMountedFrameOutcome::Published(_) => {}
        crate::mounting::UiMountedFrameOutcome::RetentionDenied(rejection) => {
            panic!("retention denial: {:?}", rejection.denial())
        }
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejection) => {
            panic!("admission denial: {:?}", rejection.denial())
        }
        crate::mounting::UiMountedFrameOutcome::CompletionDenied(denial) => {
            panic!("completion denial: {denial:?}")
        }
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_) => {
            panic!("host rejected before effects")
        }
        crate::mounting::UiMountedFrameOutcome::Unchanged(_) => panic!("unchanged"),
        crate::mounting::UiMountedFrameOutcome::Reconciled(_) => panic!("reconciled"),
        crate::mounting::UiMountedFrameOutcome::InFlight(_) => panic!("in flight"),
        crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(_) => {
            panic!("indeterminate")
        }
        crate::mounting::UiMountedFrameOutcome::Superseded(_) => panic!("superseded"),
    }
    assert!(
        session
            .mounted
            .current_unpublished_appearance()
            .unwrap()
            .is_none(),
        "a new frame without selected appearance retains no old output batch"
    );
}

pub(super) fn theme_bundle() -> crate::capability::FrozenAppearanceThemeCapabilities {
    let token = crate::capability::ThemeTokenId::new(support::APPEARANCE_TOKEN).unwrap();
    let catalog = crate::capability::UiThemeSlotCatalog::admit(
        1,
        [crate::capability::UiThemeSlotDeclaration::new(
            token.clone(),
            crate::capability::ThemeTokenFamily::surface(),
            worth_ui_dsl::UiThemeValueKind::Color,
            crate::capability::ThemeTokenSource::application(),
            crate::capability::UiThemeSlotDisclosure::Public,
            crate::capability::UiThemeSlotSuccessorCompatibility::ExactMeaning,
            None,
        )],
    )
    .unwrap();
    let definition = crate::capability::UiThemeDefinition::admit(
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.production").unwrap(),
        1,
        &catalog,
        [(
            token,
            worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                17, 34, 51, 255,
            ])),
        )],
    )
    .unwrap();
    crate::capability::FrozenAppearanceThemeCapabilities::admit(
        catalog,
        crate::capability::UiThemeDefinitionIdentity::new("theme.appearance.production").unwrap(),
        vec![definition],
    )
    .unwrap()
}
