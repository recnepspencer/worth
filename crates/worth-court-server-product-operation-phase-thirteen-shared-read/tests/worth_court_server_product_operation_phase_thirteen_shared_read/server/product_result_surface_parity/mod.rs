#[allow(unused_imports)]
use crate::product_result_surface_parity::ordinary_context::*;
use serde_json::json;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy;

#[allow(unused_imports)]
use worth_testkit_server::shared_support_durable_product_mutation as durable_support;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_route_assembly_phase_twelve_request_driver as request_driver;

#[tokio::test]
async fn worth_native_and_http_project_identical_result_and_completion_artifacts() {
    let direct_executor = worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor::default();
    let direct_server =
        worth_testkit_server::shared_support_durable_product_mutation::runtime::build_server(
            &direct_executor,
        );
    let direct = worth_testkit_server::shared_support_durable_product_mutation::runtime::session(
        &direct_server,
        "tenant-a",
        "workspace-42",
    )
    .product_operations()
    .execute(durable_host_input("surface-key"))
    .expect("direct durable mutation should commit");
    let direct_artifact = direct.result_artifact().expect("direct result artifact");
    let direct_completion = direct
        .durable_mutation_receipt()
        .expect("direct durable completion receipt");

    let http_executor = worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor::default();
    let http_server =
        worth_testkit_server::shared_support_durable_product_mutation::runtime::build_server(
            &http_executor,
        );
    let response = worth_testkit_server::shared_support_route_assembly_phase_twelve_request_driver::WorthServerRouteHttpTestDriver::new(&http_server)
        .post_json(
            "/compat/mutations/product.host_connection.upsert?basis=basis:0",
            &[
                ("x-principal-id", "principal-7"),
                ("x-tenant-id", "tenant-a"),
                ("x-workspace-id", "workspace-42"),
                ("x-branch-id", "branch-9"),
                ("idempotency-key", "surface-key"),
            ],
            &json!({ "connection_id": "host-7" }),
        )
        .await;
    let body = response.json_body().expect("HTTP JSON result");
    let http_artifact = &body["result"];
    let http_completion = &body["durable_completion"];

    assert_eq!(
        http_artifact["schema_identity"],
        direct_artifact.contract().schema().identity()
    );
    assert_eq!(
        http_artifact["schema_version"],
        direct_artifact.contract().schema().version()
    );
    assert_eq!(http_artifact["body"], *direct_artifact.body().value());
    assert_eq!(http_artifact["body_digest"], direct_artifact.body_digest());
    assert_eq!(
        http_artifact["artifact_digest"],
        direct_artifact.artifact_digest()
    );
    assert_eq!(
        http_completion["completion_digest"],
        direct_completion.completion_digest()
    );
    assert_eq!(
        http_completion["next_basis"],
        direct_completion.next_basis().value()
    );
    assert_eq!(
        http_completion["product_commit_digest"],
        direct_completion.product_commit_digest()
    );
}

#[test]
fn diagnostic_richness_does_not_change_semantic_result_or_completion() {
    let standard_executor = worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor::default();
    let standard_server =
        worth_testkit_server::shared_support_durable_product_mutation::runtime::build_server(
            &standard_executor,
        );
    let standard = session_with_diagnostics(
        &standard_server,
        worth_server::request_context::DiagnosticRichnessProfile::Standard,
    )
    .product_operations()
    .execute(durable_host_input("diagnostic-key"))
    .expect("standard diagnostic mutation should commit");

    let rich_executor = worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor::default();
    let rich_server =
        worth_testkit_server::shared_support_durable_product_mutation::runtime::build_server(
            &rich_executor,
        );
    let rich = session_with_diagnostics(
        &rich_server,
        worth_server::request_context::DiagnosticRichnessProfile::Forensic,
    )
    .product_operations()
    .execute(durable_host_input("diagnostic-key"))
    .expect("forensic diagnostic mutation should commit");

    assert_eq!(standard.result_artifact(), rich.result_artifact());
    assert_eq!(
        standard
            .durable_mutation_receipt()
            .expect("standard completion")
            .completion_digest(),
        rich.durable_mutation_receipt()
            .expect("rich completion")
            .completion_digest()
    );
}

#[test]
fn envelope_identity_commits_to_the_product_owned_completion() {
    let left_executor =
        worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor::with_product_commit_namespace("store-a");
    let left_server =
        worth_testkit_server::shared_support_durable_product_mutation::runtime::build_server(
            &left_executor,
        );
    let left = worth_testkit_server::shared_support_durable_product_mutation::runtime::session(
        &left_server,
        "tenant-a",
        "workspace-42",
    )
    .product_operations()
    .execute(durable_host_input("completion-binding-key"))
    .expect("left durable mutation should commit");

    let right_executor =
        worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor::with_product_commit_namespace("store-b");
    let right_server =
        worth_testkit_server::shared_support_durable_product_mutation::runtime::build_server(
            &right_executor,
        );
    let right = worth_testkit_server::shared_support_durable_product_mutation::runtime::session(
        &right_server,
        "tenant-a",
        "workspace-42",
    )
    .product_operations()
    .execute(durable_host_input("completion-binding-key"))
    .expect("right durable mutation should commit");

    assert_eq!(left.result_artifact(), right.result_artifact());
    assert_ne!(
        left.durable_mutation_receipt()
            .expect("left completion")
            .completion_digest(),
        right
            .durable_mutation_receipt()
            .expect("right completion")
            .completion_digest()
    );
    assert_ne!(
        left.envelope().canonical_digest(),
        right.envelope().canonical_digest()
    );
}
