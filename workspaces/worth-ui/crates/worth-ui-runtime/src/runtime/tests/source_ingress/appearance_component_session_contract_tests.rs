use super::*;

#[test]
fn component_descriptor_rejects_the_actual_backdrop_contract() {
    let token = crate::capability::ThemeTokenId::new(APPEARANCE_TOKEN).unwrap();
    let result = static_paint_component_with_contract(
        ACTIVE_COMPONENT,
        token,
        worth_ui_dsl::UiAppearanceAspectContract::backdrop(),
    );
    assert_eq!(
        result,
        Err(
            crate::capability::ComponentAppearanceAspectContractDenial::BackdropContractOnComponent
        )
    );
}
