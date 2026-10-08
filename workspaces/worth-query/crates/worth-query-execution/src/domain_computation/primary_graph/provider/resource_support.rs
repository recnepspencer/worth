use std::sync::Arc;

use crate::domain_computation::execution_runtime::WorthQueryApplicationCandidateResourceProfile;
use worth_query_admission::facade::resource_admission::{
    WorthQueryExecutionResourceSupport, WorthQueryFixedExecutionCapacity,
};
use worth_query_declaration::facade::domain_computation::{
    WorthQueryCancellationSafePointFamily, WorthQueryResourceDimension,
    WorthQueryResourceLimitRequest, WorthQuerySemanticScaleAxis, WorthQuerySemanticScaleRequest,
};
use worth_query_installation::facade::{
    WorthQueryExecutionAccessProductFamily, WorthQueryExecutionAllocatorFamily,
    WorthQueryExecutionProviderFamily, WorthQueryExecutionResourceEnvelope,
    APPLICATION_EXECUTION_ACCESS_PRODUCT_FAMILY, APPLICATION_EXECUTION_ALLOCATOR_FAMILY,
    APPLICATION_EXECUTION_PROVIDER_FAMILY, APPLICATION_EXECUTION_SAFE_POINT_FAMILY,
};

pub(super) const UNPUBLISHED_IDEMPOTENCY_CAPACITY: usize = 64;

pub(in crate::domain_computation::primary_graph) struct WorthQueryPrimaryGraphResourceSupport {
    graph: WorthQueryExecutionResourceSupport,
    snapshot:
        worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot,
}

impl WorthQueryPrimaryGraphResourceSupport {
    pub(in crate::domain_computation::primary_graph) fn install(
        maximum_concurrent_graph_work: std::num::NonZeroUsize,
        candidate_resources: WorthQueryApplicationCandidateResourceProfile,
    ) -> Self {
        let (executor, _) = component_support(
            "executor",
            maximum_concurrent_graph_work,
            candidate_resources,
        );
        let (graph, _) =
            component_support("graph", maximum_concurrent_graph_work, candidate_resources);
        let (commit, _) =
            component_support("commit", maximum_concurrent_graph_work, candidate_resources);
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

    pub(in crate::domain_computation::primary_graph) fn snapshot_ref(
        &self,
    ) -> &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot
    {
        &self.snapshot
    }
}

fn component_support(
    component: &str,
    maximum_concurrent_graph_work: std::num::NonZeroUsize,
    candidate_resources: WorthQueryApplicationCandidateResourceProfile,
) -> (
    WorthQueryExecutionResourceSupport,
    Arc<WorthQueryFixedExecutionCapacity>,
) {
    let capacity = Arc::new(
        WorthQueryFixedExecutionCapacity::new(
            format!("primary-relational-provider:{component}"),
            maximum_concurrent_graph_work.get(),
        )
        .expect("static primary provider capacity is valid"),
    );
    let scale = WorthQuerySemanticScaleRequest::selective()
        .with(
            WorthQuerySemanticScaleAxis::CandidateItems,
            candidate_resources.maximum_items(),
        )
        .with(
            WorthQuerySemanticScaleAxis::BatchWidth,
            candidate_resources.maximum_operation_width(),
        );

    let support = WorthQueryExecutionResourceSupport::new(
        WorthQueryExecutionProviderFamily::new(APPLICATION_EXECUTION_PROVIDER_FAMILY)
            .expect("static provider family is canonical"),
        WorthQueryExecutionAccessProductFamily::new(APPLICATION_EXECUTION_ACCESS_PRODUCT_FAMILY)
            .expect("static access-product family is canonical"),
        WorthQueryExecutionAllocatorFamily::new(APPLICATION_EXECUTION_ALLOCATOR_FAMILY)
            .expect("static allocator family is canonical"),
        WorthQueryExecutionResourceEnvelope::atomic(
            scale,
            WorthQueryResourceLimitRequest::selective()
                .with(
                    WorthQueryResourceDimension::CandidateRetainedRepresentationBytes,
                    candidate_resources.maximum_retained_representation_bytes(),
                )
                .with(WorthQueryResourceDimension::RetainedBytes, 262_144),
            WorthQueryCancellationSafePointFamily::new(APPLICATION_EXECUTION_SAFE_POINT_FAMILY)
                .expect("static safe-point family is canonical"),
        ),
        capacity.clone(),
    );
    (support, capacity)
}

#[cfg(test)]
mod atomic_tests;
