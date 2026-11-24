pub mod canonical_digest;
pub mod checked_fragment_reduction;
pub mod fragment_reduction;
pub mod index_access_parity;
pub mod planned_packets;

pub use canonical_digest::query_index_parity_basis_digest;
pub use checked_fragment_reduction::{
    query_reduction_scratch_bound, reduce_query_fragments_checked,
};
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

mod fragment_charge;

#[cfg(test)]
mod fragment_reduction_tests;
#[doc(hidden)]
pub mod reduction_oracle;
