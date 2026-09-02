use worth_ui::facade::appearance::{
    UiAppearanceAspect, UiAppearanceAspectContract, UiAppearanceAxisClass, UiAppearanceAxisDomain,
    UiAppearanceAxisPredicate, UiAppearanceCell, UiAppearanceDecisionPartition,
    UiAppearancePartitionAuthoring, UiAppearanceRoleApplicability, UiAppearanceRoleAuthoring,
    UiAppearanceRoleDeclaration, UiAppearanceRoleIdentity, UiAppearanceRoleRevision,
    UiAppearanceStateAxis, UiThemeSlotIdentity,
};

fn partition_authoring() -> UiAppearancePartitionAuthoring {
    UiAppearancePartitionAuthoring::new([UiAppearanceAxisDomain::complete(
        UiAppearanceStateAxis::Hover,
    )])
    .with_cell(
        UiAppearanceCell::named("outside")
            .when([UiAppearanceAxisPredicate::exact(
                UiAppearanceAxisClass::HoverOutside,
            )])
            .uses_color("button.primary.background")
            .expect("the facade accepts a valid theme slot"),
    )
    .otherwise_same_as("outside")
}

fn finite_partition() -> UiAppearanceDecisionPartition {
    partition_authoring()
        .compile(UiAppearanceAspect::Background)
        .expect("the otherwise clause closes the finite appearance partition")
}

fn main() {
    let identity = UiAppearanceRoleIdentity::new("button.primary")
        .expect("the facade accepts a valid role identity");
    let contract = UiAppearanceAspectContract::component([UiAppearanceAspect::Background], [])
        .expect("the facade accepts a valid finite appearance contract");
    let declaration = UiAppearanceRoleDeclaration::admit(
        identity.clone(),
        UiAppearanceRoleRevision::new(1).expect("one is a valid revision"),
        UiAppearanceRoleApplicability::AnyComponent,
        &contract,
        [(UiAppearanceAspect::Background, finite_partition())],
    )
    .expect("the admitted declaration preserves the contract and partition");
    let authored = UiAppearanceRoleAuthoring::new(identity)
        .cover(UiAppearanceAspect::Background, partition_authoring())
        .expect("the facade exposes authoring of a declaration")
        .build()
        .expect("the authored declaration is admitted");
    let _ = (declaration, authored, UiThemeSlotIdentity::new("proof").unwrap());
}
