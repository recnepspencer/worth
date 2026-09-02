use super::{
    test_support::portal, UiPortalIdentity, UiPortalLifecyclePosture, UiPortalOverlayBindingDenial,
    UiPortalOverlayBindingOwner, UiPortalOverlayBindingRow, UiPortalStackSnapshot,
};

fn generation(
) -> crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity {
    crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .freeze()
        .expect("Portal binding fixture freezes without a host")
        .generation_identity()
        .clone()
}

fn declaration(value: u64) -> worth_ui_dsl::UiPortalDeclarationId {
    worth_ui_dsl::UiPortalDeclarationId::new(value).expect("test declaration identity is nonzero")
}

fn snapshot(
    revision: u64,
    rows: impl IntoIterator<Item = (UiPortalIdentity, Option<UiPortalIdentity>, u64)>,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) -> UiPortalStackSnapshot {
    UiPortalStackSnapshot::for_test(
        revision,
        rows.into_iter().map(|(portal, parent, ordinal)| {
            (
                portal,
                parent,
                surface,
                ordinal,
                UiPortalLifecyclePosture::Visible,
            )
        }),
    )
}

fn binding(
    declaration: worth_ui_dsl::UiPortalDeclarationId,
    portal: UiPortalIdentity,
) -> UiPortalOverlayBindingRow {
    UiPortalOverlayBindingRow::new(declaration, portal)
}

#[test]
fn shared_declaration_exports_nested_and_sibling_instances_in_minted_order() {
    let surface = super::test_support::semantic_surface();
    let declaration = declaration(1);
    let parent = portal(11, 21);
    let child = portal(12, 22);
    let sibling = portal(13, 23);
    let snapshot = snapshot(
        37,
        [
            (sibling, None, 30),
            (child, Some(parent), 20),
            (parent, None, 10),
        ],
        surface,
    );
    let mut owner = UiPortalOverlayBindingOwner::new(generation(), surface);

    owner.bind(declaration, sibling).unwrap();
    owner.bind(declaration, parent).unwrap();
    owner.bind(declaration, child).unwrap();

    let export = owner
        .export(&snapshot)
        .expect("every shared-declaration Portal row is present");
    assert_eq!(export.portal_revision(), 37);
    assert_eq!(
        export
            .rows()
            .iter()
            .map(|row| (row.declaration(), row.portal()))
            .collect::<Vec<_>>(),
        [
            (declaration, parent),
            (declaration, child),
            (declaration, sibling)
        ]
    );
}

#[test]
fn duplicate_and_conflicting_candidates_leave_the_owner_unchanged() {
    let surface = super::test_support::semantic_surface();
    let first_declaration = declaration(2);
    let second_declaration = declaration(3);
    let first_portal = portal(31, 41);
    let second_portal = portal(32, 42);
    let snapshot = snapshot(
        41,
        [(first_portal, None, 1), (second_portal, None, 2)],
        surface,
    );
    let mut owner = UiPortalOverlayBindingOwner::new(generation(), surface);
    owner.bind(first_declaration, first_portal).unwrap();
    owner.bind(second_declaration, second_portal).unwrap();
    let before = owner.export(&snapshot).unwrap();

    assert_eq!(
        owner.bind(first_declaration, first_portal),
        Err(UiPortalOverlayBindingDenial::DuplicateBinding)
    );
    assert_eq!(owner.export(&snapshot).unwrap(), before);
    assert_eq!(
        owner.bind(second_declaration, first_portal),
        Err(UiPortalOverlayBindingDenial::PortalDeclarationConflict)
    );
    assert_eq!(owner.export(&snapshot).unwrap(), before);

    assert_eq!(
        owner.replace([
            binding(first_declaration, first_portal),
            binding(first_declaration, first_portal),
        ]),
        Err(UiPortalOverlayBindingDenial::DuplicateBinding)
    );
    assert_eq!(owner.export(&snapshot).unwrap(), before);
    assert_eq!(
        owner.replace([
            binding(first_declaration, first_portal),
            binding(second_declaration, first_portal),
        ]),
        Err(UiPortalOverlayBindingDenial::PortalDeclarationConflict)
    );
    assert_eq!(owner.export(&snapshot).unwrap(), before);
}

#[test]
fn replace_swaps_a_fully_validated_table_after_candidate_success() {
    let surface = super::test_support::semantic_surface();
    let first_declaration = declaration(4);
    let second_declaration = declaration(5);
    let first_portal = portal(51, 61);
    let second_portal = portal(52, 62);
    let snapshot = snapshot(
        53,
        [(first_portal, None, 5), (second_portal, None, 6)],
        surface,
    );
    let mut owner = UiPortalOverlayBindingOwner::new(generation(), surface);
    owner.bind(first_declaration, first_portal).unwrap();

    owner
        .replace([
            binding(second_declaration, second_portal),
            binding(first_declaration, first_portal),
        ])
        .expect("a complete replacement table is swapped atomically");

    let export = owner.export(&snapshot).unwrap();
    assert_eq!(
        export
            .rows()
            .iter()
            .map(|row| (row.declaration(), row.portal()))
            .collect::<Vec<_>>(),
        [
            (first_declaration, first_portal),
            (second_declaration, second_portal)
        ]
    );
}

#[test]
fn foreign_surface_and_missing_portal_denials_precede_export() {
    let surface = super::test_support::semantic_surface();
    let foreign_surface = super::test_support::semantic_surface();
    let declaration = declaration(6);
    let portal = portal(71, 81);
    let good_snapshot = snapshot(61, [(portal, None, 1)], surface);
    let mut owner = UiPortalOverlayBindingOwner::new(generation(), surface);
    owner.bind(declaration, portal).unwrap();
    let before = owner.export(&good_snapshot).unwrap();

    assert_eq!(
        owner.export(&snapshot(62, [(portal, None, 1)], foreign_surface)),
        Err(UiPortalOverlayBindingDenial::ForeignSurface)
    );
    assert_eq!(owner.export(&good_snapshot).unwrap(), before);
    assert_eq!(
        owner.export(&snapshot(63, [], surface)),
        Err(UiPortalOverlayBindingDenial::MissingPortal)
    );
    assert_eq!(owner.export(&good_snapshot).unwrap(), before);
}
