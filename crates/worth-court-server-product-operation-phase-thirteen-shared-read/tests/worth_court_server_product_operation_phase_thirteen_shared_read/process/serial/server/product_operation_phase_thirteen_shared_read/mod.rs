#[allow(unused_imports)]
pub(crate) use {
    worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::actions_payload,
    worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::build_server_with_execution_authority,
    worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::direct_session,
    worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::render_payload,
    worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::select_payload,
    worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::StatefulEditorLikeBackend,
};

#[test]
fn host_execution_authority_runs_charged_shared_read_batch() {
    if worth_testkit_server::test_process::reexecute_isolated_test() {
        return;
    }

    use {std::num::NonZeroUsize, std::sync::Arc};
    use {
        worth_execution::CancellationSource, worth_execution::CancellationToken,
        worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig,
        worth_execution::LeaseRequest,
    };
    use {
        worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
        worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
    };

    let authority = Arc::new(
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(1 << 27),
        })
        .unwrap(),
    );
    let backend = worth_testkit_server::shared_support_product_operation_phase_thirteen_fixture::StatefulEditorLikeBackend::new();
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
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), None, Some(30)),
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
    assert_eq!(
        lease.charged_memory_bytes(),
        0,
        "completed owner releases its tickets"
    );
    let finite_lease = server
        .execution_authority()
        .unwrap()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), Some(1 << 27), Some(30)),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let finite = session
        .product_operations()
        .execute_shared_read_batch_with_lease(inputs.clone(), &finite_lease)
        .expect("explicit finite policy admits the same batch");
    assert_eq!(finite.canonical_digest(), actual.canonical_digest());
    assert_eq!(finite_lease.charged_memory_bytes(), 0);

    let cancelled = CancellationSource::new();
    cancelled.cancel();
    let cancelled_lease = server
        .execution_authority()
        .unwrap()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), Some(1 << 27), Some(10)),
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
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), Some(1 << 27), Some(4)),
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
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), Some(1024), Some(10)),
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
                worth_server::WorthServerProductReadBatchStop::PacketStopped {
                    reason: worth_execution::MapStop::Admission(
                        worth_execution::LeaseDenial::MemoryExhausted(
                            worth_execution::MemoryLimitDenial::Capacity { .. }
                        )
                    ),
                    ..
                }
            )
        ),
        "{stopped:?}"
    );
}
