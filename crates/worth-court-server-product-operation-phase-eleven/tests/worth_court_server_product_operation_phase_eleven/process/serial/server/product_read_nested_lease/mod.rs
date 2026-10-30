#[allow(unused_imports)]
use crate::product_read_nested_lease::isolated_context::*;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_product_adapter_phase_nine_fixture as fixture;

#[allow(unused_imports)]
use worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy;

use serde_json::json;
use worth_execution::ExecutionAuthorityConfig;

#[test]
fn leased_server_adapter_propagates_exact_lease_to_nested_relational_read() {
    if worth_testkit_server::test_process::reexecute_isolated_test() {
        return;
    }

    let authority = std::sync::Arc::new(
        worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: std::num::NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(16 * 1024 * 1024),
        })
        .unwrap(),
    );
    let runtime = std::sync::Arc::new(
        worth_relational::facade::runtime::RelationalRuntimeApi::builder()
            .schema_registry(crate::product_read_nested_lease::isolated_context::relational_support::demo_schema_registry())
            .build()
            .expect("valid native runtime configuration"),
    );
    let fixture_host_request =
        crate::product_read_nested_lease::isolated_context::relational_support::serial_request();
    let fixture_execution = &worth_execution::ExecutionRequest::serial(&fixture_host_request);
    let (created, entity_id) =
        crate::product_read_nested_lease::isolated_context::relational_support::create_entity(
            fixture_execution,
            &runtime,
            &"x".repeat(32_000),
        );
    let context = runtime
        .read_truth()
        .query_plan_context(&created.snapshot)
        .unwrap();
    let plan = runtime
        .read_truth()
        .plan_query_packet(
            &created.snapshot,
            worth_relational::facade::query::PlannedQueryPacket::explicit_targets(
                "nested-server-read",
                context,
                vec![worth_relational::facade::transactions::RecordRef::Entity(
                    entity_id,
                )],
            ),
        )
        .unwrap();
    let cancellation = worth_execution::CancellationSource::new();
    let adapter = std::sync::Arc::new(NestedRelationalReadAdapter {
        runtime,
        plan,
        cancellation: cancellation.clone(),
        cancel_inside_adapter: std::sync::atomic::AtomicU8::new(0),
        nested_stop: std::sync::atomic::AtomicU8::new(0),
        seen_lease_address: std::sync::atomic::AtomicUsize::new(0),
    });
    let declaration = worth_server::WorthServerProductOperationDeclaration::product_read(
        "nested.relational",
        "nested.relational.payload.v1",
        worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::result_contract(
            "nested.relational.result.v1",
        ),
        worth_server::WorthServerProductOperationBasisKind::DurableProductDerived,
        worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
            "nested-ready",
        ),
    )
    .with_error_map(worth_server::WorthServerProductOperationErrorMaps::passthrough());
    let registration =
        worth_server::WorthServerProductApplicationAdapterRegistration::new("nested-relational", adapter.clone())
            .with_operation(declaration)
            .with_operation(
                worth_server::WorthServerProductOperationDeclaration::product_read(
                    "nested.large-registration",
                    "nested.relational.payload.v1",
                    worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::result_contract("nested.relational.result.v1"),
                    worth_server::WorthServerProductOperationBasisKind::DurableProductDerived,
                    worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
                        "x".repeat(100_000),
                    ),
                )
                .with_error_map(worth_server::WorthServerProductOperationErrorMaps::passthrough()),
            );
    let server = worth_server::WorthServer::builder()
        .with_config(
            worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::base_config(),
        )
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(worth_server::surfaces::WorthNativeSurface::enabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::phase_one_enabled())
        .register_product_adapters(vec![registration])
        .register_product_adapter(
            crate::product_read_nested_lease::isolated_context::default_checked_lane::registration(
            ),
        )
        .with_execution_authority(std::sync::Arc::clone(&authority))
        .build()
        .unwrap();
    let session =
        worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::direct_session(
            &server,
        );
    let input = || {
        worth_server::WorthServerProductOperationInput::new(
            "nested.relational",
            worth_server::WorthServerProductOperationPayload::json(
                "nested.relational.payload.v1",
                json!({}),
            ),
        )
        .with_basis_digest("basis:r0")
    };

    let unleased = session
        .product_operations()
        .execute_shared_read_batch(vec![input()])
        .expect("serial adapter entry remains usable");
    assert!(unleased.execution_report().is_none());
    assert_eq!(
        adapter
            .seen_lease_address
            .load(std::sync::atomic::Ordering::SeqCst),
        0
    );

    let wide = authority
        .request_lease(lease_request(8 * 1024 * 1024, cancellation.token()))
        .unwrap();
    crate::product_read_nested_lease::isolated_context::default_checked_lane::assert_refused(
        &session, &wide,
    );
    let completed = session
        .product_operations()
        .execute_shared_read_batch_with_lease(vec![input()], &wide)
        .expect("nested read should complete under the caller lease");
    assert_eq!(completed.operations().len(), 1);
    assert!(completed.execution_report().unwrap().charged_work() > 1);
    assert_eq!(
        adapter
            .seen_lease_address
            .load(std::sync::atomic::Ordering::SeqCst),
        &wide as *const _ as usize
    );

    let small_lease = authority
        .request_lease(lease_request(64 * 1024, cancellation.token()))
        .unwrap();
    let large_registration = worth_server::WorthServerProductOperationInput::new(
        "nested.large-registration",
        worth_server::WorthServerProductOperationPayload::json(
            "nested.relational.payload.v1",
            json!({}),
        ),
    )
    .with_basis_digest("basis:r0");
    // The declaration's retained support snapshot is charged to the caller's
    // lease before the adapter runs, so the small lease refuses its memory.
    let refused = session
        .product_operations()
        .execute_shared_read_batch_with_lease(vec![large_registration], &small_lease)
        .unwrap_err();
    assert!(
        matches!(
            refused,
            worth_server::WorthServerProductReadBatchStop::PacketStopped {
                reason: worth_execution::MapStop::Admission(
                    worth_execution::LeaseDenial::MemoryExhausted(_)
                ),
                ..
            }
        ),
        "{refused:?}"
    );
    assert_eq!(
        adapter
            .seen_lease_address
            .load(std::sync::atomic::Ordering::SeqCst),
        &wide as *const _ as usize
    );

    let tight = authority
        .request_lease(lease_request(64 * 1024, cancellation.token()))
        .unwrap();
    let stopped = session
        .product_operations()
        .execute_shared_read_batch_with_lease(vec![input()], &tight)
        .unwrap_err();
    assert!(
        matches!(
            stopped,
            worth_server::WorthServerProductReadBatchStop::PacketStopped {
                reason: worth_execution::MapStop::Failure {
                    cause: worth_execution::MapKernelFailure::Stop(
                        worth_execution::MapKernelStop::NestedStopped
                    ),
                    ..
                },
                ..
            }
        ),
        "{stopped:?}"
    );
    assert_eq!(
        adapter
            .nested_stop
            .load(std::sync::atomic::Ordering::SeqCst),
        NESTED_RESOURCE
    );
    assert_eq!(
        adapter
            .seen_lease_address
            .load(std::sync::atomic::Ordering::SeqCst),
        &tight as *const _ as usize
    );

    adapter
        .nested_stop
        .store(0, std::sync::atomic::Ordering::SeqCst);
    adapter
        .cancel_inside_adapter
        .store(1, std::sync::atomic::Ordering::SeqCst);
    let cancelling = authority
        .request_lease(lease_request(8 * 1024 * 1024, cancellation.token()))
        .unwrap();
    let stopped = session
        .product_operations()
        .execute_shared_read_batch_with_lease(vec![input()], &cancelling)
        .unwrap_err();
    assert!(matches!(
        stopped,
        worth_server::WorthServerProductReadBatchStop::PacketStopped {
            reason: worth_execution::MapStop::Failure {
                cause: worth_execution::MapKernelFailure::Stop(
                    worth_execution::MapKernelStop::NestedStopped
                ),
                ..
            },
            ..
        }
    ));
    assert_eq!(
        adapter
            .nested_stop
            .load(std::sync::atomic::Ordering::SeqCst),
        NESTED_CANCELLED
    );
    assert_eq!(
        adapter
            .seen_lease_address
            .load(std::sync::atomic::Ordering::SeqCst),
        &cancelling as *const _ as usize
    );
}
