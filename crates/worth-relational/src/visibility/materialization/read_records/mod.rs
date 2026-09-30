pub(crate) mod materialization;
mod projection;
mod reader;
mod visibility;

pub(crate) use projection::{
    entity_query_locus_comparison_key, entity_query_locus_value,
    relation_query_locus_comparison_key, relation_query_locus_value,
};
pub use projection::{
    AdjacencyStructuralRevision, AdjacencyStructuralRevisionDenial, EntityProjectionRecord,
    EntityRecordProjection, ProjectionAspectFilter, ProjectionAspectFilterMode,
    ProjectionAspectRequirement, ProjectionAspectScope, RelationProjectionRecord,
    RelationRecordProjection, RelationalAdjacencyDirection, VisibilityProjectionView,
};
pub use reader::{
    AdjacencyTruthReadLimitExceeded, BoundedAdjacencyTruthRead, BoundedEntityKindTruthRead,
    BoundedFrontierAdjacencyTruthRead, BoundedFrontierFieldEqualityTruthRead,
    BoundedRelationKindTruthRead, EntityKindTruthReadLimitExceeded,
    FrontierAdjacencyTruthReadLimitExceeded, FrontierFieldEqualityTruthReadLimitExceeded,
    QueryLeasedReadOutcome, QueryReadExecutionStop, QueryReadPacketDenial,
    RelationKindTruthReadDenial, RelationKindTruthReadLimitExceeded, VisibilityReadContext,
};

use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    pub(crate) fn visibility_reads(&self) -> VisibilityReadContext<'_> {
        VisibilityReadContext::new(self)
    }
}
