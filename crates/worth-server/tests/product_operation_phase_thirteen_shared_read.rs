#[path = "support/product_operation_phase_thirteen/fixture.rs"]
mod fixture;

use fixture::{
    actions_payload, build_server, build_server_with_execution_authority, direct_read,
    direct_session, render_payload, select_payload, StatefulEditorLikeBackend,
};

#[test]
fn host_execution_authority_runs_charged_shared_read_batch() {
    use std::{num::NonZeroUsize, sync::Arc};
    use worth_execution::{
        CancellationSource, CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig,
        LeaseRequest,
    };
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };

    let authority = Arc::new(
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: 1 << 27,
        })
        .unwrap(),
    );
    let backend = StatefulEditorLikeBackend::new();
    let server = build_server_with_execution_authority(&backend, Arc::clone(&authority));
    assert!(std::ptr::eq(
        server.execution_authority().unwrap(),
        authority.as_ref()
    ));
    let session = direct_session(&server);
    let basis = backend.basis_digest();
    let inputs = vec![
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
    ];
    let expected = session
        .product_operations()
        .execute_shared_read_batch(inputs.clone())
        .unwrap();
    let lease = server
        .execution_authority()
        .unwrap()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), 1 << 27, 30),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let actual = session
        .product_operations()
        .execute_shared_read_batch_with_lease(inputs.clone(), &lease)
        .expect("host-backed leased batch");
    assert_eq!(actual.canonical_digest(), expected.canonical_digest());
    assert!(actual.execution_report().unwrap().charged_work() > 3);

    let cancelled = CancellationSource::new();
    cancelled.cancel();
    let cancelled_lease = server
        .execution_authority()
        .unwrap()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), 1 << 27, 10),
            ),
            deadline: None,
            cancellation: cancelled.token(),
        })
        .unwrap();
    assert!(matches!(
        session
            .product_operations()
            .execute_shared_read_batch_with_lease(inputs.clone(), &cancelled_lease),
        Err(worth_server::WorthServerProductReadBatchStop::Preflight(
            worth_execution::MapKernelStop::Cancelled
        ))
    ));

    let work_limited_lease = server
        .execution_authority()
        .unwrap()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), 1 << 27, 4),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    assert!(matches!(
        session
            .product_operations()
            .execute_shared_read_batch_with_lease(inputs.clone(), &work_limited_lease),
        Err(
            worth_server::WorthServerProductReadBatchStop::PacketStopped {
                reason: worth_execution::MapStop::WorkExhausted { .. },
                ..
            }
        )
    ));

    let tight_lease = server
        .execution_authority()
        .unwrap()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), 1024, 10),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let stopped = session
        .product_operations()
        .execute_shared_read_batch_with_lease(inputs, &tight_lease);
    assert!(
        matches!(
            stopped,
            Err(
                worth_server::WorthServerProductReadBatchStop::PacketAdmission(
                    worth_execution::MapDenial::MemoryOverflow
                )
            )
        ),
        "{stopped:?}"
    );
}

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
