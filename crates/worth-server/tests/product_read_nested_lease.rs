#[path = "support/product_adapter_phase_nine/fixture.rs"]
mod fixture;
#[path = "../../worth-relational/examples/support.rs"]
mod relational_support;

use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicU8, AtomicUsize, Ordering},
        Arc,
    },
};

use serde_json::json;
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionResourceLease, LeaseRequest, MapKernelContext, MapKernelStop, MapStop,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    query::{PlannedQueryPacket, SnapshotPinnedQueryPlan},
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    transactions::RecordRef,
};
use worth_server::{
    surfaces::{CompatHttpSurface, WorthNativeSurface},
    WorthServer, WorthServerOperationRegistration, WorthServerProductAdapterExecutionError,
    WorthServerProductApplicationAdapter, WorthServerProductApplicationAdapterRegistration,
    WorthServerProductOperationBasisKind, WorthServerProductOperationDeclaration,
    WorthServerProductOperationErrorMaps, WorthServerProductOperationInput,
    WorthServerProductOperationPayload, WorthServerProductOperationSuccess,
    WorthServerProductOperationSupportSnapshot, WorthServerProductReadBatchStop,
    WorthServerScheduledProductOperation,
};

const NESTED_CANCELLED: u8 = 1;
const NESTED_RESOURCE: u8 = 2;

struct NestedRelationalReadAdapter {
    runtime: Arc<RelationalRuntime>,
    plan: SnapshotPinnedQueryPlan,
    cancellation: CancellationSource,
    cancel_inside_adapter: AtomicU8,
    nested_stop: AtomicU8,
    seen_lease_address: AtomicUsize,
}

impl WorthServerProductApplicationAdapter for NestedRelationalReadAdapter {
    fn execute(
        &self,
        operation: &WorthServerScheduledProductOperation,
    ) -> Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError> {
        fixture::schema_bound_json::publish_schema_bound_json(
            "nested.relational",
            operation.plan().declaration().result_contract(),
            "nested.relational.result.v1",
            json!({"nested": "complete"}),
        )
    }

    fn execute_with_lease(
        &self,
        operation: &WorthServerScheduledProductOperation,
        lease: &ExecutionResourceLease<'_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<
        Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError>,
        MapKernelStop,
    > {
        self.seen_lease_address
            .store(lease as *const _ as usize, Ordering::SeqCst);
        context.checkpoint(0)?;
        if self.cancel_inside_adapter.load(Ordering::SeqCst) != 0 {
            self.cancellation.cancel();
        }
        match self
            .runtime
            .read_truth()
            .execute_query_plan_with_lease(self.plan.clone(), lease)
        {
            Ok(Some(_)) => Ok(self.execute(operation)),
            Ok(None) => Err(MapKernelStop::NestedStopped),
            Err(worth_relational::facade::runtime::QueryReadExecutionStop::PacketStopped {
                reason:
                    MapStop::Failure {
                        cause: worth_execution::MapKernelFailure::Stop(MapKernelStop::Cancelled),
                        ..
                    },
                ..
            }) => {
                self.nested_stop.store(NESTED_CANCELLED, Ordering::SeqCst);
                Err(MapKernelStop::NestedStopped)
            }
            Err(
                worth_relational::facade::runtime::QueryReadExecutionStop::PreparationStopped {
                    reason:
                        MapStop::Failure {
                            cause: worth_execution::MapKernelFailure::Stop(MapKernelStop::Cancelled),
                            ..
                        },
                    ..
                },
            ) => {
                self.nested_stop.store(NESTED_CANCELLED, Ordering::SeqCst);
                Err(MapKernelStop::NestedStopped)
            }
            Err(worth_relational::facade::runtime::QueryReadExecutionStop::PacketStopped {
                reason:
                    MapStop::Admission(worth_execution::LeaseDenial::MemoryExhausted(_))
                    | MapStop::Failure {
                        cause: worth_execution::MapKernelFailure::ResultCapacityExceeded,
                        ..
                    },
                ..
            }) => {
                self.nested_stop.store(NESTED_RESOURCE, Ordering::SeqCst);
                Err(MapKernelStop::NestedStopped)
            }
            Err(
                worth_relational::facade::runtime::QueryReadExecutionStop::PreparationStopped {
                    reason: MapStop::Admission(worth_execution::LeaseDenial::MemoryExhausted(_)),
                    ..
                },
            ) => {
                self.nested_stop.store(NESTED_RESOURCE, Ordering::SeqCst);
                Err(MapKernelStop::NestedStopped)
            }
            Err(other) => panic!("unexpected nested read stop: {other:?}"),
        }
    }
}

fn lease_request(memory: u64, cancellation: CancellationToken) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), memory, 1_000_000),
        ),
        deadline: None,
        cancellation,
    }
}

#[test]
fn leased_server_adapter_propagates_exact_lease_to_nested_relational_read() {
    let authority = Arc::new(
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(16 * 1024 * 1024),
        })
        .unwrap(),
    );
    let runtime = Arc::new(
        RelationalRuntimeApi::builder()
            .schema_registry(relational_support::demo_schema_registry())
            .build(),
    );
    let (created, entity_id) = relational_support::create_entity(&runtime, &"x".repeat(32_000));
    let context = runtime
        .read_truth()
        .query_plan_context(&created.snapshot)
        .unwrap();
    let plan = runtime
        .read_truth()
        .plan_query_packet(
            &created.snapshot,
            PlannedQueryPacket::explicit_targets(
                "nested-server-read",
                context,
                vec![RecordRef::Entity(entity_id)],
            ),
        )
        .unwrap();
    let cancellation = CancellationSource::new();
    let adapter = Arc::new(NestedRelationalReadAdapter {
        runtime,
        plan,
        cancellation: cancellation.clone(),
        cancel_inside_adapter: AtomicU8::new(0),
        nested_stop: AtomicU8::new(0),
        seen_lease_address: AtomicUsize::new(0),
    });
    let declaration = WorthServerProductOperationDeclaration::product_read(
        "nested.relational",
        "nested.relational.payload.v1",
        fixture::result_contract("nested.relational.result.v1"),
        WorthServerProductOperationBasisKind::DurableProductDerived,
        WorthServerProductOperationSupportSnapshot::production_admitted("nested-ready"),
    )
    .with_error_map(WorthServerProductOperationErrorMaps::passthrough());
    let registration =
        WorthServerProductApplicationAdapterRegistration::new("nested-relational", adapter.clone())
            .with_operation(declaration)
            .with_operation(
                WorthServerProductOperationDeclaration::product_read(
                    "nested.large-registration",
                    "nested.relational.payload.v1",
                    fixture::result_contract("nested.relational.result.v1"),
                    WorthServerProductOperationBasisKind::DurableProductDerived,
                    WorthServerProductOperationSupportSnapshot::production_admitted(
                        "x".repeat(100_000),
                    ),
                )
                .with_error_map(WorthServerProductOperationErrorMaps::passthrough()),
            );
    let server = WorthServer::builder()
        .with_config(fixture::base_config())
        .register_operations(WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(WorthNativeSurface::enabled())
        .register_surface(CompatHttpSurface::phase_one_enabled())
        .register_product_adapters(vec![registration])
        .with_execution_authority(Arc::clone(&authority))
        .build()
        .unwrap();
    let session = fixture::direct_session(&server);
    let input = || {
        WorthServerProductOperationInput::new(
            "nested.relational",
            WorthServerProductOperationPayload::json("nested.relational.payload.v1", json!({})),
        )
        .with_basis_digest("basis:r0")
    };

    let unleased = session
        .product_operations()
        .execute_shared_read_batch(vec![input()])
        .expect("serial adapter entry remains usable");
    assert!(unleased.execution_report().is_none());
    assert_eq!(adapter.seen_lease_address.load(Ordering::SeqCst), 0);

    let wide = authority
        .request_lease(lease_request(8 * 1024 * 1024, cancellation.token()))
        .unwrap();
    let completed = session
        .product_operations()
        .execute_shared_read_batch_with_lease(vec![input()], &wide)
        .expect("nested read should complete under the caller lease");
    assert_eq!(completed.operations().len(), 1);
    assert!(completed.execution_report().unwrap().charged_work() > 1);
    assert_eq!(
        adapter.seen_lease_address.load(Ordering::SeqCst),
        &wide as *const _ as usize
    );

    let small_lease = authority
        .request_lease(lease_request(64 * 1024, cancellation.token()))
        .unwrap();
    let large_registration = WorthServerProductOperationInput::new(
        "nested.large-registration",
        WorthServerProductOperationPayload::json("nested.relational.payload.v1", json!({})),
    )
    .with_basis_digest("basis:r0");
    assert!(matches!(
        session
            .product_operations()
            .execute_shared_read_batch_with_lease(vec![large_registration], &small_lease),
        Err(WorthServerProductReadBatchStop::PacketStopped {
            reason: MapStop::Failure {
                cause: worth_execution::MapKernelFailure::ResultCapacityExceeded,
                ..
            },
            ..
        })
    ));
    assert_eq!(
        adapter.seen_lease_address.load(Ordering::SeqCst),
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
            WorthServerProductReadBatchStop::PacketStopped {
                reason: MapStop::Failure {
                    cause: worth_execution::MapKernelFailure::Stop(MapKernelStop::NestedStopped),
                    ..
                },
                ..
            }
        ),
        "{stopped:?}"
    );
    assert_eq!(adapter.nested_stop.load(Ordering::SeqCst), NESTED_RESOURCE);
    assert_eq!(
        adapter.seen_lease_address.load(Ordering::SeqCst),
        &tight as *const _ as usize
    );

    adapter.nested_stop.store(0, Ordering::SeqCst);
    adapter.cancel_inside_adapter.store(1, Ordering::SeqCst);
    let cancelling = authority
        .request_lease(lease_request(8 * 1024 * 1024, cancellation.token()))
        .unwrap();
    let stopped = session
        .product_operations()
        .execute_shared_read_batch_with_lease(vec![input()], &cancelling)
        .unwrap_err();
    assert!(matches!(
        stopped,
        WorthServerProductReadBatchStop::PacketStopped {
            reason: MapStop::Failure {
                cause: worth_execution::MapKernelFailure::Stop(MapKernelStop::NestedStopped),
                ..
            },
            ..
        }
    ));
    assert_eq!(adapter.nested_stop.load(Ordering::SeqCst), NESTED_CANCELLED);
    assert_eq!(
        adapter.seen_lease_address.load(Ordering::SeqCst),
        &cancelling as *const _ as usize
    );
}
