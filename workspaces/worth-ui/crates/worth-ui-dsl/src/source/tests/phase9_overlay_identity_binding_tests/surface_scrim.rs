#[test]
fn authored_surface_scrim_preserves_explicit_adjacency_and_distinct_meaning() {
    let source = r#"
        surface dashboard {}
        appearance role scrim applies_to backdrop {
            background use token(scrim.background)
            opacity use token(scrim.opacity)
        }
        backdrop dashboard.scrim {
            scope surface_singleton
            extent surface_viewport dashboard
            presence always
            motion none
            place immediately_above_surface_content
            appearance { role scrim }
        }
    "#;
    let adjacent = super::compile_file(source).unwrap();
    let unconstrained = super::compile_file(
        &source.replace("immediately_above_surface_content", "above_surface_content"),
    )
    .unwrap();
    let adjacent = adjacent.backdrop_declarations().next().unwrap();
    let unconstrained = unconstrained.backdrop_declarations().next().unwrap();
    assert_eq!(
        adjacent.declaration().placement(),
        crate::UiBackdropPlacement::ImmediatelyAboveSurfaceContent
    );
    assert_ne!(adjacent.declaration(), unconstrained.declaration());
    assert_ne!(
        adjacent.declaration().canonical_bytes(),
        unconstrained.declaration().canonical_bytes()
    );
}
