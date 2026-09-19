use super::*;

#[test]
fn compiler_expands_one_total_canonical_cartesian_partition() {
    let domains = [
        UiAppearanceAxisDomain::complete(UiAppearanceStateAxis::Hover),
        UiAppearanceAxisDomain::complete(UiAppearanceStateAxis::Pressed),
    ];
    let result = result();
    let partition = UiAppearanceDecisionPartition::compile(
        domains,
        [UiAppearanceDecisionRule::new(
            [
                UiAppearanceAxisPredicate::any(UiAppearanceStateAxis::Hover),
                UiAppearanceAxisPredicate::any(UiAppearanceStateAxis::Pressed),
            ],
            result,
        )],
    )
    .unwrap();
    assert_eq!(partition.cells().len(), 6);
    assert_eq!(partition.axes()[0].axis(), UiAppearanceStateAxis::Hover);
}

#[test]
fn compiler_rejects_overlap_holes_and_products_above_512() {
    let hover = UiAppearanceAxisDomain::complete(UiAppearanceStateAxis::Hover);
    let result = result();
    assert_eq!(
        UiAppearanceDecisionPartition::compile(
            [hover.clone()],
            [
                UiAppearanceDecisionRule::new(
                    [UiAppearanceAxisPredicate::any(UiAppearanceStateAxis::Hover)],
                    result.clone(),
                ),
                UiAppearanceDecisionRule::new(
                    [UiAppearanceAxisPredicate::exact(
                        UiAppearanceAxisClass::Hovered
                    )],
                    result.clone(),
                )
            ]
        ),
        Err(UiAppearanceDecisionPartitionDenial::AmbiguousCell)
    );
    assert!(matches!(
        UiAppearanceDecisionPartition::compile(
            [hover],
            [UiAppearanceDecisionRule::new(
                [UiAppearanceAxisPredicate::exact(
                    UiAppearanceAxisClass::Hovered
                )],
                result.clone()
            )]
        ),
        Err(UiAppearanceDecisionPartitionDenial::MissingCell { .. })
    ));
    let all = [
        UiAppearanceStateAxis::Operability,
        UiAppearanceStateAxis::Focus,
        UiAppearanceStateAxis::Validation,
        UiAppearanceStateAxis::Selection,
        UiAppearanceStateAxis::Hover,
        UiAppearanceStateAxis::Pressed,
    ]
    .map(UiAppearanceAxisDomain::complete);
    assert_eq!(
        UiAppearanceDecisionPartition::compile(all, std::iter::empty()),
        Err(UiAppearanceDecisionPartitionDenial::CellCapacityExceeded)
    );
    assert_eq!(admit_cell_count([8, 8, 8]), Ok(512));
    assert_eq!(
        admit_cell_count([513]),
        Err(UiAppearanceDecisionPartitionDenial::CellCapacityExceeded)
    );
}

#[test]
fn real_axis_product_admits_144_cells_and_denies_720_before_rule_evaluation() {
    let supported = [
        UiAppearanceStateAxis::Operability,
        UiAppearanceStateAxis::Focus,
        UiAppearanceStateAxis::Validation,
    ]
    .map(UiAppearanceAxisDomain::complete);
    let partition = UiAppearanceDecisionPartition::compile(
        supported,
        [UiAppearanceDecisionRule::new(
            [
                UiAppearanceAxisPredicate::any(UiAppearanceStateAxis::Operability),
                UiAppearanceAxisPredicate::any(UiAppearanceStateAxis::Focus),
                UiAppearanceAxisPredicate::any(UiAppearanceStateAxis::Validation),
            ],
            result(),
        )],
    )
    .unwrap();
    assert_eq!(partition.cells().len(), 6 * 4 * 6);

    let oversized = [
        UiAppearanceStateAxis::Operability,
        UiAppearanceStateAxis::Focus,
        UiAppearanceStateAxis::Validation,
        UiAppearanceStateAxis::Selection,
    ]
    .map(UiAppearanceAxisDomain::complete);
    assert_eq!(
        UiAppearanceDecisionPartition::compile(oversized, std::iter::empty()),
        Err(UiAppearanceDecisionPartitionDenial::CellCapacityExceeded)
    );
}

#[test]
fn axis_declaration_permutations_have_identical_normalized_meaning() {
    let result = result();
    let rules = || {
        [UiAppearanceDecisionRule::new(
            [
                UiAppearanceAxisPredicate::any(UiAppearanceStateAxis::Pressed),
                UiAppearanceAxisPredicate::any(UiAppearanceStateAxis::Hover),
            ],
            result.clone(),
        )]
    };
    let first = UiAppearanceDecisionPartition::compile(
        [
            UiAppearanceAxisDomain::complete(UiAppearanceStateAxis::Hover),
            UiAppearanceAxisDomain::complete(UiAppearanceStateAxis::Pressed),
        ],
        rules(),
    )
    .unwrap();
    let second = UiAppearanceDecisionPartition::compile(
        [
            UiAppearanceAxisDomain::complete(UiAppearanceStateAxis::Pressed),
            UiAppearanceAxisDomain::complete(UiAppearanceStateAxis::Hover),
        ],
        rules(),
    )
    .unwrap();
    assert_eq!(first, second);
}

#[test]
fn all_six_axis_versions_and_exact_class_sets_are_locked() {
    use UiAppearanceAxisClass::*;
    let expected: &[(UiAppearanceStateAxis, &[UiAppearanceAxisClass])] = &[
        (
            UiAppearanceStateAxis::Operability,
            &[
                OperabilityReady,
                OperabilityPending,
                OperabilityOccupied,
                OperabilityDenied,
                OperabilityUnsupported,
                OperabilityStale,
            ],
        ),
        (
            UiAppearanceStateAxis::Focus,
            &[
                FocusUnfocused,
                FocusFocused,
                FocusVisible,
                FocusedWindowInactive,
            ],
        ),
        (
            UiAppearanceStateAxis::Validation,
            &[
                ValidationUnspecified,
                ValidationValid,
                ValidationAdvisory,
                ValidationInvalid,
                ValidationPending,
                ValidationStale,
            ],
        ),
        (
            UiAppearanceStateAxis::Selection,
            &[
                SelectionUnselected,
                SelectionSelected,
                SelectionAnchor,
                SelectionCursor,
                SelectedAnchorCursor,
            ],
        ),
        (UiAppearanceStateAxis::Hover, &[HoverOutside, Hovered]),
        (
            UiAppearanceStateAxis::Pressed,
            &[PressedIdle, PressedArmedInside, PressedCapturedOutside],
        ),
    ];
    for (axis, classes) in expected {
        let domain = UiAppearanceAxisDomain::complete(*axis);
        assert_eq!(domain.version().axis(), *axis);
        assert_eq!(domain.version().revision(), 1);
        assert_eq!(domain.classes(), *classes);
        assert!(classes.iter().all(|class| class.axis() == *axis));
    }
}

fn result() -> UiAppearanceDecisionResult {
    UiAppearanceDecisionResult::theme_slot(
        super::super::UiThemeSlotIdentity::new("test.slot").unwrap(),
        super::super::UiThemeValueKind::Color,
    )
}
