use worth_ui::facade::appearance::UiAppearanceProjection;

// Gate 1 publishes only appearance declaration and finite-partition contracts.
// The unresolved projection type proves that resolved live appearance remains
// runtime-owned; it does not deny the facade module itself.
fn main() {
    let _default = UiAppearanceProjection::default();
}
