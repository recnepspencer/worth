#[allow(unused_imports)]
pub(crate) use crate::shared_support_product_operation_phase_thirteen_fixture as fixture;
#[allow(unused_imports)]
pub(crate) use crate::shared_support_test_bootstrap_policy as test_bootstrap_policy;

pub(crate) use fixture::{
    actions_payload, build_server, build_server_with_execution_authority, direct_read,
    direct_session, render_payload, select_payload, StatefulEditorLikeBackend,
};

#[test]
fn product_editor_like_render_select_and_actions_run_concurrently() {
    let backend = StatefulEditorLikeBackend::new();
    let server = build_server(&backend);
    let session = direct_session(&server);
    let basis = backend.basis_digest();

    let concurrent = session
        .product_operations()
        .execute_shared_read_batch(vec![
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.render",
                render_payload(),
            )
            .with_basis_digest(&basis),
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.select",
                select_payload("node-7"),
            )
            .with_basis_digest(&basis),
            worth_server::WorthServerProductOperationInput::new(
                "product_editor.available_actions",
                actions_payload(),
            )
            .with_basis_digest(&basis),
        ])
        .expect("shared-read batch should complete");
    let serialized = [
        direct_read(&session, "product_editor.render", render_payload(), &basis),
        direct_read(
            &session,
            "product_editor.select",
            select_payload("node-7"),
            &basis,
        ),
        direct_read(
            &session,
            "product_editor.available_actions",
            actions_payload(),
            &basis,
        ),
    ];

    assert_eq!(concurrent.counters().planned_batch_width(), 3);
    assert_eq!(concurrent.counters().admitted_read_slot_count(), 3);
    assert_eq!(concurrent.counters().queued_read_slot_count(), 3);
    assert_eq!(concurrent.counters().completed_read_slot_count(), 3);
    assert_eq!(
        concurrent
            .counters()
            .forbidden_global_lock_acquisition_count(),
        0
    );
    for (concurrent_operation, serialized_operation) in
        concurrent.operations().iter().zip(serialized.iter())
    {
        assert_eq!(
            concurrent_operation
                .scheduler_admission()
                .expect("shared read batch preserves scheduler proof")
                .scheduler_lane(),
            "shared-read"
        );
        assert_eq!(
            concurrent_operation.envelope().canonical_digest(),
            serialized_operation.envelope().canonical_digest()
        );
    }
}
