use super::{
    WorthQueryGraphCallBindingDenial, WorthQueryGraphProviderCallKind,
    WorthQueryGraphProviderCallRequest, WorthQueryGraphReceiptAdmissionDenial,
};

mod materialization_lifecycle;
mod support;

pub(super) use support::*;

#[test]
fn retained_graph_call_cannot_bind_a_later_call_receipt() {
    let attempt = attempt();
    let first = call(&attempt, "first");
    let second = call(&attempt, "second");
    let foreign_receipt = first
        .streamed_for_test("first", material("first"), projected_work_report())
        .unwrap();

    assert_eq!(
        second.admit_receipt(foreign_receipt).unwrap_err(),
        WorthQueryGraphReceiptAdmissionDenial::ForeignCall
    );
}

#[test]
fn equal_semantic_results_keep_distinct_call_and_product_occurrences() {
    let first_attempt = attempt();
    let second_attempt = attempt();
    let first = call(&first_attempt, "canonical-a");
    let second = call(&second_attempt, "canonical-b");
    let first_product = first
        .admit_receipt(
            first
                .streamed_for_test(
                    "first",
                    material_with_rows(["a", "b"]),
                    projected_work_report(),
                )
                .unwrap(),
        )
        .unwrap();
    let second_product = second
        .admit_receipt(
            second
                .streamed_for_test(
                    "second",
                    material_with_rows(["a", "b"]),
                    projected_work_report(),
                )
                .unwrap(),
        )
        .unwrap();

    let first_product = first_product.graph_read_product().unwrap();
    let second_product = second_product.graph_read_product().unwrap();
    assert!(first_product.rows().eq(second_product.rows()));
    assert_ne!(
        first_product.call_identity(),
        second_product.call_identity()
    );
    assert_ne!(first_product.identity(), second_product.identity());
}

#[test]
fn graph_product_rows_are_canonical_across_field_insertion_order() {
    let attempt = attempt();
    let first = call(&attempt, "field-order-a");
    let second = call(&attempt, "field-order-b");
    let first_receipt = first
        .admit_receipt(
            first
                .streamed_for_test(
                    "first",
                    material_with_field_order(false),
                    projected_work_report(),
                )
                .unwrap(),
        )
        .unwrap();
    let second_receipt = second
        .admit_receipt(
            second
                .streamed_for_test(
                    "second",
                    material_with_field_order(true),
                    projected_work_report(),
                )
                .unwrap(),
        )
        .unwrap();

    assert!(first_receipt
        .graph_read_product()
        .unwrap()
        .rows()
        .eq(second_receipt.graph_read_product().unwrap().rows()));
}

#[test]
fn graph_product_preserves_provider_row_order_without_hashing_rows() {
    let attempt = attempt();
    let first = call(&attempt, "row-order-a");
    let second = call(&attempt, "row-order-b");
    let first_receipt = first
        .admit_receipt(
            first
                .streamed_for_test(
                    "first",
                    material_with_rows(["a", "b"]),
                    projected_work_report(),
                )
                .unwrap(),
        )
        .unwrap();
    let second_receipt = second
        .admit_receipt(
            second
                .streamed_for_test(
                    "second",
                    material_with_rows(["b", "a"]),
                    projected_work_report(),
                )
                .unwrap(),
        )
        .unwrap();

    assert!(!first_receipt
        .graph_read_product()
        .unwrap()
        .rows()
        .eq(second_receipt.graph_read_product().unwrap().rows()));
}

#[test]
fn graph_product_retains_changed_field_values_without_hashing_rows() {
    let first_attempt = attempt();
    let second_attempt = attempt();
    let first = call(&first_attempt, "field-value-a");
    let second = call(&second_attempt, "field-value-b");
    let first_receipt = first
        .admit_receipt(
            first
                .streamed_for_test(
                    "first",
                    material_with_identity_value("vertex-a"),
                    projected_work_report(),
                )
                .unwrap(),
        )
        .unwrap();
    let second_receipt = second
        .admit_receipt(
            second
                .streamed_for_test(
                    "second",
                    material_with_identity_value("vertex-b"),
                    projected_work_report(),
                )
                .unwrap(),
        )
        .unwrap();

    assert!(!first_receipt
        .graph_read_product()
        .unwrap()
        .rows()
        .eq(second_receipt.graph_read_product().unwrap().rows()));
}

#[test]
fn non_projection_call_cannot_seal_projection_material() {
    let attempt = attempt_with_access(
        worth_query_installation::facade::WorthQueryOperationGraphAccess::Observe,
    );
    let call = call_with_kind(
        &attempt,
        "observe-cannot-project",
        WorthQueryGraphProviderCallKind::Observe,
    );

    assert!(call
        .streamed_for_test(
            "unexpected",
            material("unexpected"),
            projected_work_report(),
        )
        .is_err());
}

#[test]
fn provider_session_rejects_resources_admitted_for_another_session() {
    let owner_attempt = attempt();
    let foreign_attempt = attempt();
    let denial = owner_attempt
        .attempt
        .provider_session_for_test()
        .bind_graph_provider_call(
            &owner_attempt.graph,
            call_spec(
                "foreign-resource-attempt",
                WorthQueryGraphProviderCallKind::Project,
            ),
            foreign_attempt.attempt.evidence(),
            foreign_attempt.attempt.resources().shared_envelope(),
        )
        .unwrap_err();

    assert_eq!(
        denial,
        WorthQueryGraphCallBindingDenial::ForeignResourceAttempt
    );
}

#[test]
fn provider_session_rejects_an_installed_but_undeclared_graph_authority() {
    let attempt = attempt();
    let denial = attempt
        .attempt
        .provider_session_for_test()
        .bind_graph_provider_call(
            &attempt.foreign_graph,
            WorthQueryGraphProviderCallRequest::direct(
                WorthQueryGraphProviderCallKind::Project,
                "foreign-graph",
            )
            .with_managed_execution_snapshot("snapshot"),
            attempt.attempt.evidence(),
            attempt.attempt.resources().shared_envelope(),
        )
        .unwrap_err();

    assert_eq!(
        denial,
        WorthQueryGraphCallBindingDenial::BoundOperationAuthorityMismatch
    );
}

#[test]
fn direct_provider_session_rejects_a_caller_authored_workflow_stage() {
    let attempt = attempt();
    let denial = attempt
        .attempt
        .provider_session_for_test()
        .bind_graph_provider_call(
            &attempt.graph,
            WorthQueryGraphProviderCallRequest::workflow_stage(
                WorthQueryGraphProviderCallKind::Project,
                "invented-stage",
                "invented",
            )
            .with_managed_execution_snapshot("snapshot"),
            attempt.attempt.evidence(),
            attempt.attempt.resources().shared_envelope(),
        )
        .unwrap_err();

    assert_eq!(
        denial,
        WorthQueryGraphCallBindingDenial::BoundOperationAuthorityMismatch
    );
}
