#[path = "declared_overlay_binding_tests/allocation_succession.rs"]
mod allocation_succession;
#[path = "declared_overlay_binding_tests/appearance_publication.rs"]
mod appearance_publication;
#[path = "declared_overlay_binding_tests/appearance_publication_lifecycle.rs"]
mod appearance_publication_lifecycle;
#[path = "declared_overlay_binding_tests/appearance_publication_support.rs"]
mod appearance_publication_support;
#[path = "declared_overlay_binding_tests/appearance_region_clip.rs"]
mod appearance_region_clip;
#[path = "declared_overlay_binding_tests/appearance_region_extent.rs"]
mod appearance_region_extent;
#[path = "declared_overlay_binding_tests/appearance_structural_portal.rs"]
mod appearance_structural_portal;
#[path = "declared_overlay_binding_tests/appearance_surface_scope.rs"]
mod appearance_surface_scope;
#[path = "declared_overlay_binding_tests/integrated_appearance_world.rs"]
mod integrated_appearance_world;
#[path = "declared_overlay_binding_test_support.rs"]
mod test_support;

use test_support::{authored_overlay_session, portal_target, presentation};

#[test]
fn prepared_authored_surface_binding_survives_allocation_and_exports_current_portal() {
    let mut session = authored_overlay_session();
    let material = session.application.authored_overlay_material();
    let surface_declaration = material
        .overlay_declaration_bindings()
        .surface_named("workspace.surface.overlay")
        .expect("prepared source should retain the issued surface declaration");
    let portal_declaration = material
        .overlay_declaration_bindings()
        .portal_named("overlay.menu")
        .expect("prepared source should retain the issued Portal declaration");
    let runtime_surface = session
        .create_declared_semantic_surface(surface_declaration)
        .expect("declared surface binding should use the prepared generation");
    let (graph_node, mounted) = portal_target(&mut session, runtime_surface);
    let routed_portal_declaration = match session
        .application
        .prepared_authority()
        .intent_catalog()
        .lookup(
            graph_node,
            crate::capability::UiSemanticInteractionFamily::Activate,
        ) {
        Some((crate::declaration::UiIntentCatalogResolvedRoute::Product { route, .. }, _)) => {
            route.portal_declaration()
        }
        _ => panic!("authored source should retain its prepared product route"),
    };
    assert_eq!(
        routed_portal_declaration,
        Some(portal_declaration),
        "the real prepared route must carry the issued Portal declaration"
    );
    let portal = crate::runtime::portal::UiPortalIdentity::for_owner(
        crate::runtime::portal::UiPortalOwnerIdentity::from_mounted_owner(graph_node, mounted),
    );
    let geometry =
        crate::runtime::interaction::UiPresentedInteractionGeometry::for_test(presentation());
    let request = crate::runtime::portal::UiPortalServiceRequest::open(
        portal,
        crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(
            session.session_identity().as_u64(),
            1,
        ),
        geometry,
        Some(
            crate::runtime::interaction::UiPresentedViewportGeometry::for_test(
                geometry.clip_bounds(),
                geometry.presentation(),
            ),
        ),
        runtime_surface,
    )
    .with_declared_portal(Some(portal_declaration));
    let stage = session
        .admit_authored_portal_open(portal_declaration, portal, runtime_surface)
        .expect("prepared Portal declaration should admit before Portal preparation");
    let transition = session
        .portal
        .as_ref()
        .expect("source Portal declaration installs the Portal owner")
        .prepare(request)
        .expect("admitted Portal request should prepare");
    let binding_commit = crate::runtime::portal::UiPortalOverlayBindingCommit::from_transition(
        &transition,
        Some(stage),
    );
    session
        .portal
        .as_mut()
        .expect("source Portal declaration installs the Portal owner")
        .commit_published(transition)
        .expect("prepared Portal transition should commit");
    session
        .commit_authored_overlay_binding(binding_commit)
        .expect("published Portal transition should commit its binding");

    let exports = session
        .authored_overlay_binding_exports()
        .expect("current Portal owner should export its binding");
    assert_eq!(exports.len(), 1);
    assert_eq!(exports[0].generation(), session.generation_identity());
    assert_eq!(exports[0].runtime_surface(), runtime_surface);
    assert_eq!(exports[0].rows().len(), 1);
    assert_eq!(exports[0].rows()[0].declaration(), portal_declaration);
    assert_eq!(exports[0].rows()[0].portal(), portal);

    let idempotent_request = crate::runtime::portal::UiPortalServiceRequest::open(
        portal,
        request.idempotency(),
        geometry,
        request.presented_viewport(),
        runtime_surface,
    )
    .with_declared_portal(Some(portal_declaration));
    let idempotent_stage = session
        .admit_authored_portal_open(portal_declaration, portal, runtime_surface)
        .expect("same declaration and Portal identity should be idempotent");
    let idempotent_transition = session
        .portal
        .as_ref()
        .expect("source Portal declaration installs the Portal owner")
        .prepare(idempotent_request)
        .expect("same Portal request should prepare idempotently");
    assert!(idempotent_transition.is_idempotent());
    let idempotent_commit = crate::runtime::portal::UiPortalOverlayBindingCommit::from_transition(
        &idempotent_transition,
        Some(idempotent_stage),
    );
    session
        .portal
        .as_mut()
        .expect("source Portal declaration installs the Portal owner")
        .commit_published(idempotent_transition)
        .expect("idempotent Portal transition should commit");
    session
        .commit_authored_overlay_binding(idempotent_commit.clone())
        .expect("idempotent Portal binding should remain current");
    assert_eq!(
        session
            .authored_overlay_binding_exports()
            .expect("idempotent binding remains exportable")[0]
            .rows()[0]
            .declaration(),
        portal_declaration
    );
    allocation_succession::assert_preserved(&mut session, idempotent_commit);
    let _ = session.shutdown();
}

#[test]
fn authored_portal_denies_unbound_surface_before_portal_preparation() {
    let session = authored_overlay_session();
    let material = session.application.authored_overlay_material();
    let surface_declaration = material
        .overlay_declaration_bindings()
        .surface_named("workspace.surface.overlay")
        .expect("prepared source should retain the issued surface declaration");
    let portal_declaration = material
        .overlay_declaration_bindings()
        .portal_named("overlay.menu")
        .expect("prepared source should retain the issued Portal declaration");
    let runtime_surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound()
        .expect("unbound denial surface identity capacity");
    let portal = crate::runtime::portal::UiPortalIdentity::for_owner(
        crate::runtime::portal::UiPortalOwnerIdentity::from_mounted_owner(
            session
                .graph()
                .node_identities()
                .next()
                .expect("source graph should retain a target"),
            worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound()
                .expect("unbound denial mounted identity capacity"),
        ),
    );
    let denial = session
        .admit_authored_portal_open(portal_declaration, portal, runtime_surface)
        .expect_err("an authored Portal cannot bypass its declared surface binding");
    assert_eq!(
        denial,
        crate::runtime::portal::UiPortalOverlayBindingLifecycleDenial::Mounted(
            crate::mounting::UiMountedIdentityDenial::UnknownMountedInstance,
        )
    );
    assert!(session
        .authored_overlay_bindings
        .validate_declared_surface(
            &session.active_generation_identity(),
            material,
            surface_declaration,
        )
        .is_ok());
    let _ = session.shutdown();
}

#[test]
fn retained_close_keeps_binding_until_terminal_portal_exit() {
    let mut session = authored_overlay_session();
    let material = session.application.authored_overlay_material();
    let surface_declaration = material
        .overlay_declaration_bindings()
        .surface_named("workspace.surface.overlay")
        .expect("prepared source should retain the issued surface declaration");
    let portal_declaration = material
        .overlay_declaration_bindings()
        .portal_named("overlay.menu")
        .expect("prepared source should retain the issued Portal declaration");
    let runtime_surface = session
        .create_declared_semantic_surface(surface_declaration)
        .expect("declared surface binding should use the prepared generation");
    let (graph_node, mounted) = portal_target(&mut session, runtime_surface);
    let portal = crate::runtime::portal::UiPortalIdentity::for_owner(
        crate::runtime::portal::UiPortalOwnerIdentity::from_mounted_owner(graph_node, mounted),
    );
    let geometry =
        crate::runtime::interaction::UiPresentedInteractionGeometry::for_test(presentation());
    let open = crate::runtime::portal::UiPortalServiceRequest::open(
        portal,
        crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(
            session.session_identity().as_u64(),
            10,
        ),
        geometry,
        Some(
            crate::runtime::interaction::UiPresentedViewportGeometry::for_test(
                geometry.clip_bounds(),
                geometry.presentation(),
            ),
        ),
        runtime_surface,
    )
    .with_declared_portal(Some(portal_declaration));
    let stage = session
        .admit_authored_portal_open(portal_declaration, portal, runtime_surface)
        .expect("authored Portal should admit before opening");
    let transition = session.portal.as_ref().unwrap().prepare(open).unwrap();
    let open_commit = crate::runtime::portal::UiPortalOverlayBindingCommit::from_transition(
        &transition,
        Some(stage),
    );
    session
        .portal
        .as_mut()
        .unwrap()
        .commit_published(transition)
        .unwrap();
    session
        .commit_authored_overlay_binding(open_commit)
        .unwrap();

    let close = crate::runtime::portal::UiPortalServiceRequest::close(
        portal,
        crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(
            session.session_identity().as_u64(),
            11,
        ),
        crate::runtime::portal::UiPortalDismissalCause::ExplicitOwnerRequest,
        runtime_surface,
    );
    let transition = session.portal.as_ref().unwrap().prepare(close).unwrap();
    let binding_commit =
        crate::runtime::portal::UiPortalOverlayBindingCommit::from_transition(&transition, None)
            .with_retained_exit(true);
    let (_, retention) = session
        .portal
        .as_mut()
        .unwrap()
        .commit_published_with_exit_retention(transition, true)
        .unwrap();
    assert!(retention.is_some());
    session
        .commit_authored_overlay_binding(binding_commit)
        .unwrap();
    assert_eq!(
        session.authored_overlay_binding_exports().unwrap()[0]
            .rows()
            .len(),
        1,
        "Closing retains the binding until terminalization"
    );

    let retention = retention.expect("Closing Portal retains a terminal receipt");
    let terminal = session
        .portal
        .as_ref()
        .unwrap()
        .prepare_exit_terminal(
            retention,
            crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(
                session.session_identity().as_u64(),
                12,
            ),
        )
        .unwrap();
    let terminal_commit =
        crate::runtime::portal::UiPortalOverlayBindingCommit::from_transition(&terminal, None)
            .with_retained_exit(false);
    session
        .portal
        .as_mut()
        .unwrap()
        .commit_published(terminal)
        .unwrap();
    session
        .commit_authored_overlay_binding(terminal_commit)
        .unwrap();
    assert!(session
        .authored_overlay_binding_exports()
        .unwrap()
        .is_empty());
    let _ = session.shutdown();
}
