#[path = "support/test_bootstrap_policy.rs"]
mod test_bootstrap_policy;

use worth_server::WorthServerProductOperationInput;

#[path = "support/product_operation_phase_thirteen/fixture.rs"]
mod fixture;
#[path = "support/product_operation_phase_thirteen/parity_driver.rs"]
mod parity_driver;

use fixture::{
    apply_payload, build_server, direct_mutation, direct_read, direct_session, finalize_payload,
    open_mutation_session, prepared_mutation_request, prepared_read_request, render_payload,
    select_payload, StatefulEditorLikeBackend,
};
use parity_driver::WorthServerRouteHttpTestDriver;

#[tokio::test]
async fn product_editor_like_http_and_worth_native_paths_are_plan_equivalent() {
    let backend = StatefulEditorLikeBackend::new();
    let server = build_server(&backend);
    let session = direct_session(&server);
    let initial_basis = backend.basis_digest();
    let direct_render = direct_read(
        &session,
        "product_editor.render",
        render_payload(),
        &initial_basis,
    );
    let compat_render = server
        .compat_http()
        .product_operations()
        .execute(
            &prepared_read_request(&server, "product_editor.render", Some(&initial_basis)),
            WorthServerProductOperationInput::new("product_editor.render", render_payload()),
        )
        .expect("compat render should succeed");
    let direct_select = direct_read(
        &session,
        "product_editor.select",
        select_payload("node-7"),
        &initial_basis,
    );
    let compat_select = server
        .compat_http()
        .product_operations()
        .execute(
            &prepared_read_request(&server, "product_editor.select", Some(&initial_basis)),
            WorthServerProductOperationInput::new(
                "product_editor.select",
                select_payload("node-7"),
            ),
        )
        .expect("compat select should succeed");
    let route_driver = WorthServerRouteHttpTestDriver::new(&server);
    let route_render = route_driver
        .get(
            &format!("/compat/reads/product_editor.render?basis={initial_basis}"),
            &[
                ("x-principal-id", "principal-7"),
                ("x-tenant-id", "tenant-a"),
                ("x-workspace-id", "workspace-42"),
                ("x-branch-id", "branch-9"),
            ],
        )
        .await;
    let route_select = route_driver
        .get(
            &format!("/compat/reads/product_editor.select?basis={initial_basis}&node=node-7"),
            &[
                ("x-principal-id", "principal-7"),
                ("x-tenant-id", "tenant-a"),
                ("x-workspace-id", "workspace-42"),
                ("x-branch-id", "branch-9"),
            ],
        )
        .await;
    let mutation_session = open_mutation_session(&session, &initial_basis, "product_editor.apply");
    let direct_apply = direct_mutation(
        &session,
        "product_editor.apply",
        apply_payload("Renamed"),
        &initial_basis,
        &mutation_session,
    )
    .expect("direct apply should succeed");
    let compat_apply = server
        .compat_http()
        .product_operations()
        .execute(
            &prepared_mutation_request(&server, "product_editor.apply", Some(&initial_basis)),
            WorthServerProductOperationInput::new("product_editor.apply", apply_payload("Renamed"))
                .with_product_session_identity(mutation_session.identity().as_str()),
        )
        .expect("compat apply should succeed");
    let finalize_session =
        open_mutation_session(&session, &initial_basis, "product_editor.finalize");
    let direct_finalize = direct_mutation(
        &session,
        "product_editor.finalize",
        finalize_payload(true),
        &initial_basis,
        &finalize_session,
    )
    .expect("direct finalize should succeed");
    let compat_finalize = server
        .compat_http()
        .product_operations()
        .execute(
            &prepared_mutation_request(&server, "product_editor.finalize", Some(&initial_basis)),
            WorthServerProductOperationInput::new(
                "product_editor.finalize",
                finalize_payload(true),
            )
            .with_product_session_identity(finalize_session.identity().as_str()),
        )
        .expect("compat finalize should succeed");
    let route_apply = route_driver
        .post_json(
            &format!("/compat/mutations/product_editor.apply?basis={initial_basis}"),
            &[
                ("x-principal-id", "principal-7"),
                ("x-tenant-id", "tenant-a"),
                ("x-workspace-id", "workspace-42"),
                ("x-branch-id", "branch-9"),
                ("x-product-session-id", mutation_session.identity().as_str()),
            ],
            &serde_json::json!({ "title": "Renamed" }),
        )
        .await;
    let route_finalize = route_driver
        .post_json(
            &format!("/compat/mutations/product_editor.finalize?basis={initial_basis}"),
            &[
                ("x-principal-id", "principal-7"),
                ("x-tenant-id", "tenant-a"),
                ("x-workspace-id", "workspace-42"),
                ("x-branch-id", "branch-9"),
                ("x-product-session-id", finalize_session.identity().as_str()),
            ],
            &serde_json::json!({ "confirm": true }),
        )
        .await;

    assert_eq!(
        direct_render
            .plan()
            .expect("direct render plan")
            .canonical_digest(),
        compat_render
            .plan()
            .expect("compat render plan")
            .canonical_digest()
    );
    assert_eq!(
        direct_render.envelope().canonical_digest(),
        compat_render.envelope().canonical_digest()
    );
    assert_eq!(
        direct_select
            .plan()
            .expect("direct select plan")
            .canonical_digest(),
        compat_select
            .plan()
            .expect("compat select plan")
            .canonical_digest()
    );
    assert_eq!(
        direct_select.envelope().canonical_digest(),
        compat_select.envelope().canonical_digest()
    );
    assert_eq!(
        direct_apply
            .plan()
            .expect("direct apply plan")
            .canonical_digest(),
        compat_apply
            .plan()
            .expect("compat apply plan")
            .canonical_digest()
    );
    assert_eq!(
        direct_finalize
            .plan()
            .expect("direct finalize plan")
            .canonical_digest(),
        compat_finalize
            .plan()
            .expect("compat finalize plan")
            .canonical_digest()
    );
    assert_eq!(
        route_render.plan_digest(),
        Some(
            direct_render
                .plan()
                .expect("route render plan")
                .canonical_digest()
        )
    );
    assert_eq!(
        route_select.plan_digest(),
        Some(
            direct_select
                .plan()
                .expect("route select plan")
                .canonical_digest()
        )
    );
    assert_eq!(
        route_apply.plan_digest(),
        Some(
            direct_apply
                .plan()
                .expect("route apply plan")
                .canonical_digest()
        )
    );
    assert_eq!(
        route_finalize.plan_digest(),
        Some(
            direct_finalize
                .plan()
                .expect("route finalize plan")
                .canonical_digest()
        )
    );
    assert_eq!(route_render.operation_name(), Some("product_editor.render"));
    assert_eq!(route_select.operation_name(), Some("product_editor.select"));
    assert_eq!(route_apply.operation_name(), Some("product_editor.apply"));
    assert_eq!(
        route_finalize.operation_name(),
        Some("product_editor.finalize")
    );
    assert_eq!(
        route_render.envelope_digest(),
        Some(direct_render.envelope().canonical_digest())
    );
    assert_eq!(
        route_select.envelope_digest(),
        Some(direct_select.envelope().canonical_digest())
    );
    assert!(route_apply.envelope_digest().is_some());
    assert!(route_finalize.envelope_digest().is_some());
}
