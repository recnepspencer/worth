mod canonical_digest;
mod checked_fragment_reduction;
mod fragment_reduction;
#[cfg(test)]
mod fragment_reduction_tests;
mod index_access_parity;
mod planned_packets;

pub(crate) use canonical_digest::{
    query_authoritative_entity_record_digest, query_authoritative_relation_record_digest,
    query_index_parity_basis_digest, query_result_reduction_digest,
};
pub(crate) use checked_fragment_reduction::reduce_query_fragments_checked;
pub use fragment_reduction::{
    reduce_query_fragments, CanonicalQueryResult, QueryComplexitySummary, QueryExecutionOutcome,
    QueryFragmentCounters, QueryWorkerFragment, TraversalEntityVisitKey, TraversalReductionBasis,
    TraversalRelationVisitKey,
};
pub use index_access_parity::{
    IndexParityMode, IndexParityVerifiedQueryOutcome, IndexQueryRejectionClass, QueryAccessPath,
};
pub use planned_packets::{
    deterministic_query_fragment_key, DeterministicQueryFragmentKey, DeterministicQueryPlanKey,
    PartitionHint, PlannedQueryPacket, QueryAccessContract, QueryExecutionShape,
    QueryLocalityClass, QueryOrderingContract, QueryParallelLegality, QueryParallelProfitability,
    QueryPlanContextId, QueryPlanEvidenceBasis, QueryScope, QuerySerialReason, ReadPacketPlan,
    ReductionDiscipline, SnapshotPinnedQueryPlan,
};
