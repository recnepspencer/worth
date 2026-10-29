use worth_server::{
    WorthServerProductRouteParityCertificationProof, WorthServerProductRouteParityEntry,
};

use super::super::fixture::{
    apply_payload, direct_mutation, direct_read, finalize_payload, open_mutation_session,
    prepared_mutation_request, prepared_read_request, render_payload, select_payload,
};
#[allow(unused_imports)]
use crate::shared_support_route_assembly_phase_twelve_request_driver as route_driver;

use route_driver::WorthServerRouteHttpTestDriver;

pub async fn observed_route_parity(
    server: &worth_server::WorthServer,
    session: &worth_server::WorthServerWorthNativeSession,
    initial_basis: &str,
    after_apply_basis: &str,
) -> WorthServerProductRouteParityCertificationProof {
    let direct_render = direct_read(
        session,
        "product_editor.render",
        render_payload(),
        initial_basis,
    );
    let compat_render = server
        .compat_http()
        .product_operations()
        .execute(
            &prepared_read_request(server, "product_editor.render", Some(initial_basis)),
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.render",
                render_payload(),
            ),
        )
        .expect("compat render");
    let direct_select = direct_read(
        session,
        "product_editor.select",
        select_payload("node-7"),
        initial_basis,
    );
    let compat_select = server
        .compat_http()
        .product_operations()
        .execute(
            &prepared_read_request(server, "product_editor.select", Some(initial_basis)),
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.select",
                select_payload("node-7"),
            ),
        )
        .expect("compat select");
    let route_driver = WorthServerRouteHttpTestDriver::new(server);
    let route_render = route_driver
        .get(
            &format!("/compat/reads/product_editor.render?basis={initial_basis}"),
            headers(),
        )
        .await;
    let route_select = route_driver
        .get(
            &format!("/compat/reads/product_editor.select?basis={initial_basis}&node=node-7"),
            headers(),
        )
        .await;
    let apply_session = open_mutation_session(session, initial_basis, "product_editor.apply");
    let direct_apply = direct_mutation(
        session,
        "product_editor.apply",
        apply_payload("Parity Rename"),
        initial_basis,
        &apply_session,
    )
    .expect("direct apply");
    let compat_apply = server
        .compat_http()
        .product_operations()
        .execute(
            &prepared_mutation_request(server, "product_editor.apply", Some(initial_basis)),
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.apply",
                apply_payload("Parity Rename"),
            )
            .with_product_session_identity(apply_session.identity().as_str()),
        )
        .expect("compat apply");
    let finalize_session =
        open_mutation_session(session, after_apply_basis, "product_editor.finalize");
    let direct_finalize = direct_mutation(
        session,
        "product_editor.finalize",
        finalize_payload(true),
        after_apply_basis,
        &finalize_session,
    )
    .expect("direct finalize");
    let compat_finalize = server
        .compat_http()
        .product_operations()
        .execute(
            &prepared_mutation_request(server, "product_editor.finalize", Some(after_apply_basis)),
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.finalize",
                finalize_payload(true),
            )
            .with_product_session_identity(finalize_session.identity().as_str()),
        )
        .expect("compat finalize");
    let route_apply = route_driver
        .post_json(
            &format!("/compat/mutations/product_editor.apply?basis={initial_basis}"),
            &mutation_headers(apply_session.identity().as_str()),
            &serde_json::json!({ "title": "Parity Rename" }),
        )
        .await;
    let route_finalize = route_driver
        .post_json(
            &format!("/compat/mutations/product_editor.finalize?basis={after_apply_basis}"),
            &mutation_headers(finalize_session.identity().as_str()),
            &serde_json::json!({ "confirm": true }),
        )
        .await;

    WorthServerProductRouteParityCertificationProof::new(&[
        WorthServerProductRouteParityEntry::new(
            "product_editor.render",
            &direct_render,
            &compat_render,
            route_render.plan_digest().expect("route render plan"),
            route_render
                .envelope_digest()
                .expect("route render envelope"),
            true,
        ),
        WorthServerProductRouteParityEntry::new(
            "product_editor.select",
            &direct_select,
            &compat_select,
            route_select.plan_digest().expect("route select plan"),
            route_select
                .envelope_digest()
                .expect("route select envelope"),
            true,
        ),
        WorthServerProductRouteParityEntry::new(
            "product_editor.apply",
            &direct_apply,
            &compat_apply,
            route_apply.plan_digest().expect("route apply plan"),
            route_apply.envelope_digest().expect("route apply envelope"),
            false,
        ),
        WorthServerProductRouteParityEntry::new(
            "product_editor.finalize",
            &direct_finalize,
            &compat_finalize,
            route_finalize.plan_digest().expect("route finalize plan"),
            route_finalize
                .envelope_digest()
                .expect("route finalize envelope"),
            false,
        ),
    ])
    .expect("route parity proof")
}

fn headers() -> &'static [(&'static str, &'static str)] {
    &[
        ("x-principal-id", "principal-7"),
        ("x-tenant-id", "tenant-a"),
        ("x-workspace-id", "workspace-42"),
        ("x-branch-id", "branch-9"),
    ]
}

fn mutation_headers(session_identity: &str) -> [(&str, &str); 5] {
    [
        ("x-principal-id", "principal-7"),
        ("x-tenant-id", "tenant-a"),
        ("x-workspace-id", "workspace-42"),
        ("x-branch-id", "branch-9"),
        ("x-product-session-id", session_identity),
    ]
}
