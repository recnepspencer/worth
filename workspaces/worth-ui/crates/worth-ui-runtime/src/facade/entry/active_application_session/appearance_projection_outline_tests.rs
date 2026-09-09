use super::support;

#[test]
fn outline_damage_uses_the_bound_surfaces_qualified_device_scale() {
    for scale in [1_000, 1_250, 1_500, 2_000, 1_100] {
        run_outline_case(scale);
    }
}

fn run_outline_case(scale: u32) {
    let role = outline_role();
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    let profile = worth_ui_host_native::staged_appearance_capability_report();
    host.set_capabilities(profile.clone());
    let observer = host.clone();
    let mut session = support::single_aspect_appearance_component_builder(
        &role,
        worth_ui_dsl::UiAppearanceAspect::Outline,
    )
    .register_appearance_theme_bundle(super::test_support::theme_bundle())
    .unwrap()
    .with_rust_authored_declaration_fixture(support::appearance_fixture(&role))
    .freeze()
    .map(|application| {
        crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
            application,
            host,
        )
    })
    .expect("outline source must prepare")
    .launch()
    .expect("outline owner must launch");
    let (surface, graph_node) = super::mounting_fixture::mount(&mut session, scale);
    let observation = support::two_node_appearance_candidate_submission(
        &session,
        "outline-geometry-current",
        &role,
        support::APPEARANCE_NODE_A,
    );
    let admitted = {
        let mut turn = session.begin_observation_turn().unwrap();
        turn.admit_source(observation).unwrap();
        turn.seal().unwrap()
    };
    session.classify_observations(admitted).unwrap();
    session.advance_mounted_identity_frame().unwrap();
    let frame = session
        .prepare_mounted_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            |_| {},
        )
        .unwrap_or_else(|_| panic!("outline mounting must prepare"));
    assert_eq!(frame.manifest().surfaces()[0].device_scale_milli(), scale);
    if scale == 1_100 {
        assert_eq!(
            frame.qualified_outline_fringe_for_test(profile.appearance_profile(), surface),
            Err(
                crate::mounting::UiMountedAppearanceLoweringDenial::HostGeometryScale(
                    worth_ui_host_contract::UiHostAppearanceScaleDenial::UnsupportedScale(1_100)
                )
            )
        );
    } else {
        assert_eq!(
            frame
                .qualified_outline_fringe_for_test(profile.appearance_profile(), surface)
                .unwrap()
                .subpixels(),
            expected_fringe(scale)
        );
    }
    if scale != 1_100 {
        observer.push_native_display_settled_without_effects();
    }
    let outcome = session.present_prepared_mounted_frame_internal(
        frame,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
        1,
    );
    if scale == 1_100 {
        assert_eq!(
            match outcome {
                crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejection) => {
                    rejection.denial()
                }
                _ => panic!("unsupported outline geometry must deny before publication"),
            },
            crate::mounting::UiMountedPresentationAdmissionDenial::AppearanceOutputUnavailable,
        );
    } else {
        assert!(matches!(
            outcome,
            crate::mounting::UiMountedFrameOutcome::Published(_)
        ));
        let query = worth_ui_inspection::UiAppearanceInspectionQuery::new(
            session.appearance_inspection_world(surface),
            graph_node.digest(),
            worth_ui_dsl::UiAppearanceAspect::Outline,
        );
        let worth_ui_inspection::UiAppearanceInspectionOutcome::Found(explanation) =
            session.why_appearance(query)
        else {
            panic!("the literal outline must have mounted inspection evidence");
        };
        assert!(!explanation.denied_before_effects());
        assert_eq!(
            explanation.value_source(),
            &worth_ui_inspection::UiAppearanceInspectionValueSource::Literal
        );
        assert_eq!(explanation.cost().theme_slots_compared(), 0);
        let output = session
            .mounted
            .current_unpublished_appearance()
            .unwrap()
            .unwrap();
        assert_outline(output, expected_fringe(scale));
        let frame = session
            .prepare_mounted_reconstruction_frame_with_application_presentation(
                crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
                &[],
                |_| {},
            )
            .unwrap_or_else(|_| panic!("outline reconstruction must prepare"));
        frame.verify_outline_geometry_denial_preserves_predecessor(
            profile.appearance_profile().unwrap(),
        );
        let output =
            frame.lower_unpublished_appearance_with_profile_for_test(profile.appearance_profile());
        assert_outline(&output, expected_fringe(scale));
        assert_eq!(
            output.fragments()[0].work().posture(),
            worth_ui_host_contract::UiMountedAppearanceWorkPosture::Reconstruction
        );
    }
    let _ = session.shutdown();
}

fn expected_fringe(scale: u32) -> u32 {
    match scale {
        1_000 => 1_000,
        1_250 => 800,
        1_500 => 667,
        2_000 => 500,
        _ => unreachable!("only qualified scales reach outline lowering"),
    }
}

fn assert_outline(
    output: &worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection,
    fringe: u32,
) {
    assert_eq!(output.fragments().len(), 1);
    let work = output.fragments()[0].work();
    let [worth_ui_host_contract::UiMountedAppearanceMechanic::Outline(outline)] =
        work.successor().mechanics()
    else {
        panic!("the authored outline must reach actual frame lowering");
    };
    assert_eq!(outline.anti_alias_fringe().subpixels(), fringe);
    let allocation = outline.allocation();
    let visual = outline.visual_bounds();
    let expansion = 750 + i32::try_from(fringe).unwrap();
    assert_eq!(visual.x(), allocation.x() - expansion);
    assert_eq!(visual.y(), allocation.y() - expansion);
    assert_eq!(visual.width(), allocation.width() + 2 * expansion as u32);
    assert_eq!(visual.height(), allocation.height() + 2 * expansion as u32);
    if work.posture() == worth_ui_host_contract::UiMountedAppearanceWorkPosture::Initial {
        assert_eq!(work.damage().len(), 1);
        let damage = &work.damage()[0];
        assert_eq!(
            (damage.x(), damage.y(), damage.width(), damage.height()),
            (visual.x(), visual.y(), visual.width(), visual.height())
        );
    }
    let transcript =
        worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
    let [worth_ui_host_headless::UiHeadlessAppearanceMechanic::Outline(translated)] =
        transcript.fragments()[0].work().successor().mechanics()
    else {
        panic!("headless translation must preserve the outline");
    };
    let clip = translated.clip();
    // Independent coverage samples in the four straight stroke interiors.
    // Width500 + offset250 puts each sample500 outside the allocation.
    // Own-allocation clipping makes all four false even with correct damage.
    let left = i64::from(allocation.x());
    let top = i64::from(allocation.y());
    let right = left + i64::from(allocation.width());
    let bottom = top + i64::from(allocation.height());
    for (x, y) in [
        (left - 500, (top + bottom) / 2),
        (right + 500, (top + bottom) / 2),
        ((left + right) / 2, top - 500),
        ((left + right) / 2, bottom + 500),
    ] {
        assert!(x >= i64::from(clip.x()) && x < i64::from(clip.x()) + i64::from(clip.width()));
        assert!(y >= i64::from(clip.y()) && y < i64::from(clip.y()) + i64::from(clip.height()));
    }
}

fn outline_role() -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    use worth_ui_dsl::*;
    let axis = UiAppearanceStateAxis::Validation;
    let contract =
        UiAppearanceAspectContract::component([UiAppearanceAspect::Outline], []).unwrap();
    let partition = UiAppearancePartitionAuthoring::new([UiAppearanceAxisDomain::complete(axis)])
        .with_cell(
            UiAppearanceCell::when([UiAppearanceAxisPredicate::any(axis)]).literal(
                UiThemeValue::SolidOutline(
                    UiThemeOutline::new(
                        UiThemeSolidStroke::new(
                            UiThemeColor::from_channels([10, 100, 200, 255]),
                            UiLogicalLength::new(500),
                        )
                        .unwrap(),
                        UiLogicalLength::new(250),
                    )
                    .unwrap(),
                ),
            ),
        )
        .compile(UiAppearanceAspect::Outline)
        .unwrap();
    UiAppearanceRoleDeclaration::admit(
        UiAppearanceRoleIdentity::new("test.qualified-outline").unwrap(),
        UiAppearanceRoleRevision::new(1).unwrap(),
        UiAppearanceRoleApplicability::AnyComponent,
        &contract,
        [(UiAppearanceAspect::Outline, partition)],
    )
    .unwrap()
}
