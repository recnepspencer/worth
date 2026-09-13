use super::*;

#[test]
fn declared_region_names_do_not_alias_an_unnamed_typed_backdrop_extent() {
    use WorthUiArtifactInputBodyAtom::{Identifier as Id, LeftBrace as Open, RightBrace as Close};
    let role = rust_backdrop_role();
    let backdrop = typed_region_backdrop(&role);
    let UiBackdropExtentBasis::PresentedMosaicRegion {
        region: reserved, ..
    } = backdrop.extent()
    else {
        panic!("fixture carries a typed region extent");
    };
    let atoms: Vec<_> = (0..8)
        .flat_map(|index| {
            [
                Id("region".into()),
                Id(format!("workspace.region{index}")),
                Open,
                Close,
            ]
        })
        .collect();
    let package = compile_rust(
        WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
            .with_surface("workspace.surface")
            .with_component_body_atoms("workspace.component", atoms)
            .with_appearance_role(role)
            .with_backdrop(backdrop.clone()),
    )
    .unwrap();
    assert_eq!(
        package
            .backdrop_declarations()
            .next()
            .unwrap()
            .declaration(),
        &backdrop
    );
    for index in 0..8 {
        let named = package
            .overlay_declaration_bindings()
            .region_named("workspace.surface", &format!("workspace.region{index}"))
            .unwrap();
        assert_ne!(
            named, reserved,
            "a new region name cannot claim the typed extent's identity"
        );
    }
}

#[test]
fn file_and_rust_region_bindings_do_not_require_a_backdrop() {
    let file = compile_file(
        r#"
        surface alpha.surface {}
        surface beta.surface {}
        component workspace.component {
            region workspace.outer { region workspace.inner {} }
        }
        component workspace.unmounted { region workspace.unused {} }
    "#,
    )
    .unwrap();
    use WorthUiArtifactInputBodyAtom::{Identifier as Id, LeftBrace as Open, RightBrace as Close};
    let rust = compile_rust(
        WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
            .with_surface("alpha.surface")
            .with_surface("beta.surface")
            .with_component_body_atoms(
                "workspace.component",
                vec![
                    Id("region".into()),
                    Id("workspace.outer".into()),
                    Open,
                    Id("region".into()),
                    Id("workspace.inner".into()),
                    Open,
                    Close,
                    Close,
                ],
            )
            .with_component_body_atoms(
                "workspace.unmounted",
                vec![
                    Id("region".into()),
                    Id("workspace.unused".into()),
                    Open,
                    Close,
                ],
            ),
    )
    .unwrap();
    let mut identities = std::collections::BTreeSet::new();
    for surface in ["alpha.surface", "beta.surface"] {
        for region in ["workspace.outer", "workspace.inner", "workspace.unused"] {
            let file_binding = file
                .overlay_declaration_bindings()
                .region_named(surface, region)
                .expect("actual region declarations have compiler identities without Backdrops");
            assert_eq!(
                Some(file_binding),
                rust.overlay_declaration_bindings()
                    .region_named(surface, region)
            );
            assert!(
                identities.insert(file_binding),
                "surface-qualified declaration identities are distinct"
            );
        }
        assert!(file
            .overlay_declaration_bindings()
            .region_named(surface, "workspace.missing")
            .is_none());
    }
    assert!(file.backdrop_declarations().next().is_none());
    assert!(rust.backdrop_declarations().next().is_none());
}
