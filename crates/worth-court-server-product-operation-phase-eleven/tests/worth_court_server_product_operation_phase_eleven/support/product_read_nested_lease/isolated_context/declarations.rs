#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_product_adapter_phase_nine_fixture as fixture};




#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};

pub(crate) use {std::num::NonZeroUsize, std::sync::atomic::AtomicU8, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};

pub(crate) use {serde_json::json};
pub(crate) use {worth_execution::CancellationSource, worth_execution::CancellationToken, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionResourceLease, worth_execution::LeaseRequest, worth_execution::MapKernelContext, worth_execution::MapKernelStop, worth_execution::MapStop};
pub(crate) use {worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy};
pub(crate) use {worth_relational::facade::query::PlannedQueryPacket, worth_relational::facade::query::SnapshotPinnedQueryPlan, worth_relational::facade::runtime::RelationalRuntime, worth_relational::facade::runtime::RelationalRuntimeApi, worth_relational::facade::transactions::RecordRef};
pub(crate) use {worth_server::surfaces::CompatHttpSurface, worth_server::surfaces::WorthNativeSurface, worth_server::WorthServer, worth_server::WorthServerOperationRegistration, worth_server::WorthServerProductAdapterExecutionError, worth_server::WorthServerProductApplicationAdapter, worth_server::WorthServerProductApplicationAdapterRegistration, worth_server::WorthServerProductOperationBasisKind, worth_server::WorthServerProductOperationDeclaration, worth_server::WorthServerProductOperationErrorMaps, worth_server::WorthServerProductOperationInput, worth_server::WorthServerProductOperationPayload, worth_server::WorthServerProductOperationSuccess, worth_server::WorthServerProductOperationSupportSnapshot, worth_server::WorthServerProductReadBatchStop, worth_server::WorthServerScheduledProductOperation};

pub(crate) const NESTED_CANCELLED: u8 = 1;
pub(crate) const NESTED_RESOURCE: u8 = 2;

pub(crate) struct NestedRelationalReadAdapter {
    pub(crate) runtime: Arc<RelationalRuntime>,
    pub(crate) plan: SnapshotPinnedQueryPlan,
    pub(crate) cancellation: CancellationSource,
    pub(crate) cancel_inside_adapter: AtomicU8,
    pub(crate) nested_stop: AtomicU8,
    pub(crate) seen_lease_address: AtomicUsize,
}

impl WorthServerProductApplicationAdapter for NestedRelationalReadAdapter {
    fn execute(
        &self,
        operation: &WorthServerScheduledProductOperation,
    ) -> Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError> {
        worth_testkit_server::shared_support_product_result_schema_bound_json::publish_schema_bound_json(
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
            .store(lease as *const _ as usize, std::sync::atomic::Ordering::SeqCst);
        context.checkpoint(0)?;
        if self.cancel_inside_adapter.load(std::sync::atomic::Ordering::SeqCst) != 0 {
            self.cancellation.cancel();
        }
        match self.runtime.read_truth().execute_query_plan(
            self.plan.clone(),
            &worth_execution::ExecutionRequest::leased(lease),
        ) {
            Ok(Some(_)) => Ok(self.execute(operation)),
            Ok(None) => Err(worth_execution::MapKernelStop::NestedStopped),
            Err(worth_relational::facade::runtime::QueryReadExecutionStop::PacketStopped {
                reason:
                    worth_execution::MapStop::Failure {
                        cause: worth_execution::MapKernelFailure::Stop(worth_execution::MapKernelStop::Cancelled),
                        ..
                    },
                ..
            }) => {
                self.nested_stop.store(NESTED_CANCELLED, std::sync::atomic::Ordering::SeqCst);
                Err(worth_execution::MapKernelStop::NestedStopped)
            }
            Err(
                worth_relational::facade::runtime::QueryReadExecutionStop::PreparationStopped {
                    reason:
                        worth_execution::MapStop::Failure {
                            cause: worth_execution::MapKernelFailure::Stop(worth_execution::MapKernelStop::Cancelled),
                            ..
                        },
                    ..
                },
            ) => {
                self.nested_stop.store(NESTED_CANCELLED, std::sync::atomic::Ordering::SeqCst);
                Err(worth_execution::MapKernelStop::NestedStopped)
            }
            Err(worth_relational::facade::runtime::QueryReadExecutionStop::PacketStopped {
                reason:
                    worth_execution::MapStop::Admission(worth_execution::LeaseDenial::MemoryExhausted(_))
                    | worth_execution::MapStop::Failure {
                        cause: worth_execution::MapKernelFailure::ResultCapacityExceeded,
                        ..
                    },
                ..
            }) => {
                self.nested_stop.store(NESTED_RESOURCE, std::sync::atomic::Ordering::SeqCst);
                Err(worth_execution::MapKernelStop::NestedStopped)
            }
            Err(
                worth_relational::facade::runtime::QueryReadExecutionStop::PreparationStopped {
                    reason: worth_execution::MapStop::Admission(worth_execution::LeaseDenial::MemoryExhausted(_)),
                    ..
                },
            ) => {
                self.nested_stop.store(NESTED_RESOURCE, std::sync::atomic::Ordering::SeqCst);
                Err(worth_execution::MapKernelStop::NestedStopped)
            }
            Err(other) => panic!("unexpected nested read stop: {other:?}"),
        }
    }
}

pub(crate) fn lease_request(memory: u64, cancellation: CancellationToken) -> LeaseRequest {
    LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Automatic,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::new(4).unwrap(), Some(memory), Some(1_000_000)),
        ),
        deadline: None,
        cancellation,
    }
}