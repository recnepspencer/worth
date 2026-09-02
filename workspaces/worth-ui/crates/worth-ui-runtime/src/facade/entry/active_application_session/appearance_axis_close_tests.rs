use std::collections::BTreeSet;

#[path = "appearance_axis_close_test_support.rs"]
mod appearance_axis_close_test_support;

use worth_ui_dsl::{
    UiAppearanceAspect, UiAppearanceAxisDomain, UiAppearanceAxisPredicate, UiAppearanceCell,
    UiAppearancePartitionAuthoring, UiAppearanceRoleDeclaration, UiAppearanceRoleIdentity,
    UiAppearanceStateAxis, UiLogicalLength, UiThemeColor, UiThemeCornerRadii, UiThemeOpacity,
    UiThemeOutline, UiThemeSolidStroke, UiThemeValue, UI_APPEARANCE_DECISION_CELL_CAPACITY,
};

#[test]
fn active_session_close_seals_all_six_owner_exports_on_one_basis() {
    let role = six_axis_role();
    let axes = role
        .partitions()
        .iter()
        .flat_map(|(_, partition)| partition.axes().iter().map(|axis| axis.axis()))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        axes,
        [
            UiAppearanceStateAxis::Operability,
            UiAppearanceStateAxis::Focus,
            UiAppearanceStateAxis::Validation,
            UiAppearanceStateAxis::Selection,
            UiAppearanceStateAxis::Hover,
            UiAppearanceStateAxis::Pressed,
        ]
        .into_iter()
        .collect::<BTreeSet<_>>()
    );
    let cell_count = role
        .partitions()
        .iter()
        .map(|(_, partition)| partition.cells().len())
        .sum::<usize>();
    assert_eq!(cell_count, 26);
    assert!(cell_count <= UI_APPEARANCE_DECISION_CELL_CAPACITY);

    let capability_snapshot = appearance_axis_close_test_support::six_axis_builder(&role)
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("six-axis appearance capability snapshot should prepare");
    let launch_candidate = appearance_axis_close_test_support::candidate_submission(
        "appearance-six-axis-close",
        &role,
        capability_snapshot.capabilities(),
    );
    let prepared = appearance_axis_close_test_support::six_axis_builder(&role)
        .with_candidate_submission(launch_candidate)
        .freeze()
        .expect("six-axis appearance source application should prepare");
    assert_eq!(
        prepared.service_policy_plan().focus(),
        Some(crate::declaration::UiFocusPolicy::workbench())
    );
    assert_eq!(
        prepared.service_policy_plan().selection(),
        Some(crate::declaration::UiSelectionPolicy::single())
    );
    let mut session =
        crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host(
            prepared,
        )
        .launch()
        .expect("six-axis appearance source application should launch");
    let candidate = appearance_axis_close_test_support::candidate_submission(
        "appearance-six-axis-observation",
        &role,
        session.capabilities(),
    );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let admitted = turn.seal().unwrap();
    let snapshot = admitted
        .appearance_owner_snapshot_for_test()
        .expect("an all-axis role must carry the close snapshot");

    assert_eq!(snapshot.turn(), admitted.turn());
    assert_eq!(snapshot.session(), admitted.session());
    assert_eq!(snapshot.source_basis(), admitted.source_basis());
    assert_eq!(snapshot.generation(), &session.active_generation_identity());
    for axis in [
        UiAppearanceStateAxis::Operability,
        UiAppearanceStateAxis::Focus,
        UiAppearanceStateAxis::Validation,
        UiAppearanceStateAxis::Selection,
        UiAppearanceStateAxis::Hover,
        UiAppearanceStateAxis::Pressed,
    ] {
        assert!(snapshot.demand().contains(axis));
    }
    assert!(snapshot.operability().is_some());
    assert!(snapshot.focus().is_some());
    assert!(snapshot.validation().is_some());
    assert!(snapshot.selection().is_some());
    assert!(snapshot.pointer_presence().is_some());
    assert!(snapshot.pressed().is_some());
    let _ = session.shutdown();
}

fn six_axis_role() -> UiAppearanceRoleDeclaration {
    UiAppearanceRoleDeclaration::authoring(
        UiAppearanceRoleIdentity::new("test.six-axis-owner-close").unwrap(),
    )
    .cover(
        UiAppearanceAspect::Background,
        axis_partition(
            UiAppearanceStateAxis::Operability,
            UiThemeValue::Color(UiThemeColor::from_channels([20, 40, 60, 255])),
        ),
    )
    .unwrap()
    .cover(
        UiAppearanceAspect::Foreground,
        axis_partition(
            UiAppearanceStateAxis::Focus,
            UiThemeValue::Color(UiThemeColor::from_channels([220, 230, 240, 255])),
        ),
    )
    .unwrap()
    .cover(
        UiAppearanceAspect::Border,
        axis_partition(
            UiAppearanceStateAxis::Validation,
            UiThemeValue::SolidStroke(
                UiThemeSolidStroke::new(
                    UiThemeColor::from_channels([180, 20, 30, 255]),
                    UiLogicalLength::new(1_000),
                )
                .unwrap(),
            ),
        ),
    )
    .unwrap()
    .cover(
        UiAppearanceAspect::Radius,
        axis_partition(
            UiAppearanceStateAxis::Selection,
            UiThemeValue::CornerRadii(
                UiThemeCornerRadii::new(
                    UiLogicalLength::new(1_000),
                    UiLogicalLength::new(2_000),
                    UiLogicalLength::new(3_000),
                    UiLogicalLength::new(4_000),
                )
                .unwrap(),
            ),
        ),
    )
    .unwrap()
    .cover(
        UiAppearanceAspect::Opacity,
        axis_partition(
            UiAppearanceStateAxis::Hover,
            UiThemeValue::Opacity(UiThemeOpacity::from_units(50_000)),
        ),
    )
    .unwrap()
    .cover(
        UiAppearanceAspect::Outline,
        axis_partition(
            UiAppearanceStateAxis::Pressed,
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
    .unwrap()
    .build()
    .unwrap()
}

fn axis_partition(
    axis: UiAppearanceStateAxis,
    value: UiThemeValue,
) -> UiAppearancePartitionAuthoring {
    UiAppearancePartitionAuthoring::new([UiAppearanceAxisDomain::complete(axis)])
        .with_cell(UiAppearanceCell::when([UiAppearanceAxisPredicate::any(axis)]).literal(value))
}
