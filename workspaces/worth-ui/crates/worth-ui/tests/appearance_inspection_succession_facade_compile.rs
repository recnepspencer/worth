use worth_ui::facade::appearance::UiAppearanceInspectionGenerationSuccessionDenial;

#[test]
fn appearance_inspection_succession_denial_is_nameable_through_public_facade() {
    let denial = UiAppearanceInspectionGenerationSuccessionDenial::StaleInspectionGeneration;
    assert!(matches!(
        denial,
        UiAppearanceInspectionGenerationSuccessionDenial::StaleInspectionGeneration
    ));
}
