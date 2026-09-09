use std::sync::Arc;

use worth_query_admission::facade::resource_admission::{
    WorthQueryExecutionResourceSupport, WorthQueryFixedExecutionCapacity,
};
use worth_query_declaration::facade::domain_computation::{
    WorthQueryCancellationSafePointFamily, WorthQueryExecutionMode, WorthQueryResourceDimension,
    WorthQueryResourceLimitRequest, WorthQuerySemanticScaleRequest,
};
use worth_query_installation::facade::{
    WorthQueryExecutionAccessProductFamily, WorthQueryExecutionAllocatorFamily,
    WorthQueryExecutionProviderFamily, WorthQueryExecutionResourceEnvelope,
    APPLICATION_EXECUTION_ACCESS_PRODUCT_FAMILY, APPLICATION_EXECUTION_ALLOCATOR_FAMILY,
    APPLICATION_EXECUTION_PROVIDER_FAMILY, APPLICATION_EXECUTION_SAFE_POINT_FAMILY,
};

pub(super) const UNPUBLISHED_IDEMPOTENCY_CAPACITY: usize = 64;

pub(super) struct WorthQueryPrimaryGraphResourceSupport {
    graph: WorthQueryExecutionResourceSupport,
    snapshot:
        worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot,
}

impl WorthQueryPrimaryGraphResourceSupport {
    pub(super) fn install() -> Self {
        let executor = component_support("executor");
        let graph = component_support("graph");
        let commit = component_support("commit");
        let snapshot =
            worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot::new(
                executor,
                Vec::new(),
                vec![("primary".to_owned(), graph.clone())],
                vec![("primary".to_owned(), commit)],
                None,
            );
        Self { graph, snapshot }
    }

    pub(super) fn graph(&self) -> WorthQueryExecutionResourceSupport {
        self.graph.clone()
    }

    pub(super) fn snapshot(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot
    {
        self.snapshot.clone()
    }
}

fn component_support(component: &str) -> WorthQueryExecutionResourceSupport {
    WorthQueryExecutionResourceSupport::new(
        WorthQueryExecutionProviderFamily::new(APPLICATION_EXECUTION_PROVIDER_FAMILY)
            .expect("static provider family is canonical"),
        WorthQueryExecutionAccessProductFamily::new(APPLICATION_EXECUTION_ACCESS_PRODUCT_FAMILY)
            .expect("static access-product family is canonical"),
        WorthQueryExecutionAllocatorFamily::new(APPLICATION_EXECUTION_ALLOCATOR_FAMILY)
            .expect("static allocator family is canonical"),
        WorthQueryExecutionResourceEnvelope::new(
            WorthQuerySemanticScaleRequest::bounded(4_096),
            WorthQueryResourceLimitRequest::bounded(4_096)
                .with(WorthQueryResourceDimension::RetainedBytes, 262_144),
            WorthQueryExecutionMode::Synchronous,
            None,
            WorthQueryCancellationSafePointFamily::new(APPLICATION_EXECUTION_SAFE_POINT_FAMILY)
                .expect("static safe-point family is canonical"),
        ),
        Arc::new(
            WorthQueryFixedExecutionCapacity::new(
                format!("primary-relational-provider:{component}"),
                64,
            )
            .expect("static primary provider capacity is valid"),
        ),
    )
}
