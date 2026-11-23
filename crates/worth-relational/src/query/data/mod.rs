pub(crate) mod canonical_digest;
pub(crate) mod checked_fragment_reduction;
pub(crate) mod fragment_reduction;
#[cfg(test)]
mod fragment_reduction_tests;
pub(crate) mod index_access_parity;
pub(crate) mod planned_packets;
pub(crate) mod snapshot_pinned_plan;

#[cfg(test)]
pub(crate) use canonical_digest::query_index_parity_basis_digest;
pub(crate) use checked_fragment_reduction::{
    query_reduction_scratch_bound, reduce_query_fragments_checked,
};
#[cfg(test)]
pub use fragment_reduction::reduce_query_fragments;
pub use fragment_reduction::{
    CanonicalQueryResult, QueryComplexitySummary, QueryFragmentCounters, QueryWorkerFragment,
    TraversalEntityVisitKey, TraversalReductionBasis, TraversalRelationVisitKey,
};
pub use index_access_parity::{IndexParityMode, IndexQueryRejectionClass, QueryAccessPath};
pub use planned_packets::{
    deterministic_query_fragment_key, DeterministicQueryFragmentKey, DeterministicQueryPlanKey,
    PartitionHint, PlannedQueryPacket, QueryAccessContract, QueryExecutionShape,
    QueryLocalityClass, QueryOrderingContract, QueryParallelLegality, QueryParallelProfitability,
    QueryPlanContextId, QueryPlanEvidenceBasis, QueryScope, QuerySerialReason, ReadPacketPlan,
    ReductionDiscipline,
};

pub(crate) mod query_outcome;

pub(crate) mod index_parity_outcome;
