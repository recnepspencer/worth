use crate::{
    UiAppearanceAspect, UiAppearanceAxisClass, UiAppearanceAxisDomain, UiAppearanceAxisPredicate,
    UiAppearanceCell, UiAppearanceDecisionPartitionDenial, UiAppearancePartitionAuthoring,
    UiAppearanceRole, UiAppearanceRoleIdentity, UiDslComponentReference, UiThemeSlotIdentity,
    UiThemeValueKind,
};

fn slot(name: &str) -> UiThemeSlotIdentity {
    UiThemeSlotIdentity::new(name).expect("test slot identity is valid")
}

fn domain(axis: crate::UiAppearanceStateAxis) -> UiAppearanceAxisDomain {
    UiAppearanceAxisDomain::complete(axis)
}

fn color_cell(
    name: Option<&str>,
    predicates: impl IntoIterator<Item = UiAppearanceAxisPredicate>,
    value: &str,
) -> UiAppearanceCell {
    UiAppearanceCell::new(
        name,
        predicates,
        crate::UiAppearanceCellValue::theme_slot(slot(value), UiThemeValueKind::Color),
    )
}

#[test]
fn otherwise_compiles_to_a_total_finite_partition() {
    let partition = UiAppearancePartitionAuthoring::new([
        domain(crate::UiAppearanceStateAxis::Pressed),
        domain(crate::UiAppearanceStateAxis::Hover),
    ])
    .with_cell(color_cell(
        Some("outside"),
        [UiAppearanceAxisPredicate::exact(
            UiAppearanceAxisClass::HoverOutside,
        )],
        "button.outside",
    ))
    .otherwise_same_as("outside")
    .compile(UiAppearanceAspect::Background)
    .expect("otherwise should close the finite complement");

    assert_eq!(partition.cells().len(), 6);
}

#[test]
fn declaration_permutations_lower_to_identical_role_bytes() {
    let first = UiAppearanceRole::new(UiAppearanceRoleIdentity::new("button.primary").unwrap())
        .applies_to_component(UiDslComponentReference::new("button").unwrap())
        .cover(
            UiAppearanceAspect::Background,
            UiAppearancePartitionAuthoring::new([
                domain(crate::UiAppearanceStateAxis::Pressed),
                domain(crate::UiAppearanceStateAxis::Hover),
            ])
            .with_cell(
                UiAppearanceCell::named("outside")
                    .when([
                        UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::HoverOutside),
                        UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::PressedIdle),
                    ])
                    .uses_slot(slot("button.outside"), UiThemeValueKind::Color),
            )
            .with_cell(
                UiAppearanceCell::named("hovered")
                    .when([
                        UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::Hovered),
                        UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::PressedIdle),
                    ])
                    .uses_slot(slot("button.hovered"), UiThemeValueKind::Color),
            )
            .otherwise_same_as("outside"),
        )
        .unwrap()
        .build()
        .unwrap();
    let second = UiAppearanceRole::new(UiAppearanceRoleIdentity::new("button.primary").unwrap())
        .applies_to_component(UiDslComponentReference::new("button").unwrap())
        .cover(
            UiAppearanceAspect::Background,
            UiAppearancePartitionAuthoring::new([
                domain(crate::UiAppearanceStateAxis::Hover),
                domain(crate::UiAppearanceStateAxis::Pressed),
            ])
            .with_cell(
                UiAppearanceCell::named("hovered")
                    .when([
                        UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::PressedIdle),
                        UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::Hovered),
                    ])
                    .uses_slot(slot("button.hovered"), UiThemeValueKind::Color),
            )
            .with_cell(
                UiAppearanceCell::named("outside")
                    .when([
                        UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::PressedIdle),
                        UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::HoverOutside),
                    ])
                    .uses_slot(slot("button.outside"), UiThemeValueKind::Color),
            )
            .otherwise_same_as("outside"),
        )
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(first.canonical_bytes(), second.canonical_bytes());
}

#[test]
fn overlap_is_not_resolved_by_source_order() {
    let denial = UiAppearancePartitionAuthoring::new([domain(crate::UiAppearanceStateAxis::Hover)])
        .with_cell(color_cell(
            None,
            [UiAppearanceAxisPredicate::exact(
                UiAppearanceAxisClass::HoverOutside,
            )],
            "first",
        ))
        .with_cell(color_cell(
            None,
            [UiAppearanceAxisPredicate::exact(
                UiAppearanceAxisClass::HoverOutside,
            )],
            "second",
        ))
        .compile(UiAppearanceAspect::Background);

    assert_eq!(
        denial,
        Err(UiAppearanceDecisionPartitionDenial::OverlappingCell)
    );
}

#[test]
fn missing_and_cyclic_named_cells_are_typed() {
    let missing = UiAppearancePartitionAuthoring::new([])
        .with_otherwise(crate::UiAppearanceCellValue::same_as("unknown"))
        .compile(UiAppearanceAspect::Background);
    assert_eq!(
        missing,
        Err(UiAppearanceDecisionPartitionDenial::MissingNamedCell)
    );

    let cyclic = UiAppearancePartitionAuthoring::new([])
        .with_cell(UiAppearanceCell::named("first").same_as("second"))
        .with_cell(UiAppearanceCell::named("second").same_as("first"))
        .compile(UiAppearanceAspect::Background);
    assert_eq!(
        cyclic,
        Err(UiAppearanceDecisionPartitionDenial::CyclicCellReference)
    );
}

#[test]
fn repeated_otherwise_is_not_a_last_writer_fallback() {
    let denial = UiAppearancePartitionAuthoring::new([])
        .with_otherwise(crate::UiAppearanceCellValue::same_as("first"))
        .with_otherwise(crate::UiAppearanceCellValue::same_as("second"))
        .compile(UiAppearanceAspect::Background);

    assert_eq!(
        denial,
        Err(UiAppearanceDecisionPartitionDenial::DuplicateOtherwise)
    );
}

#[test]
fn wrong_kind_and_capacity_are_denied_before_lowering() {
    let wrong_kind = UiAppearancePartitionAuthoring::new([])
        .with_cell(
            UiAppearanceCell::when([]).uses_slot(slot("wrong.kind"), UiThemeValueKind::Opacity),
        )
        .compile(UiAppearanceAspect::Background);
    assert_eq!(
        wrong_kind,
        Err(UiAppearanceDecisionPartitionDenial::ResultValueKindMismatch)
    );

    let too_many_cells = UiAppearancePartitionAuthoring::new([
        domain(crate::UiAppearanceStateAxis::Operability),
        domain(crate::UiAppearanceStateAxis::Focus),
        domain(crate::UiAppearanceStateAxis::Validation),
        domain(crate::UiAppearanceStateAxis::Selection),
    ])
    .compile(UiAppearanceAspect::Background);
    assert_eq!(
        too_many_cells,
        Err(UiAppearanceDecisionPartitionDenial::CellCapacityExceeded)
    );
}
