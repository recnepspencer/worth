pub(crate) mod materialization;
mod native_scan_stop;
mod projection;
pub(crate) mod read_footprint;
pub(crate) mod read_prices;
pub(crate) mod read_request;
pub(crate) mod read_work;
mod reader;
mod visibility;

pub(crate) use projection::{
    authoritative_state_query_locus_comparison_key, authoritative_state_query_locus_value,
    entity_query_locus_comparison_key, entity_query_locus_value,
    relation_query_locus_comparison_key, relation_query_locus_value,
};
pub use projection::{
    AdjacencyStructuralRevision, AdjacencyStructuralRevisionDenial, EntityProjectionRecord,
    EntityRecordProjection, ProjectionAspectFilter, ProjectionAspectFilterMode,
    ProjectionAspectRequirement, ProjectionAspectScope, RelationProjectionRecord,
    RelationRecordProjection, RelationalAdjacencyDirection, RelationalAdjacencyVisit,
    RelationalBorrowedRecordReadDenial, RelationalEntityMetadata, RelationalEntityRetirement,
    RelationalProjectedFieldObservation, RelationalRelationMetadata,
    RelationalSnapshotProjectionAdmissionStop, VisibilityProjectionView,
};
pub use reader::{
    AdjacencyTruthReadLimitExceeded, BoundedAdjacencyTruthRead, BoundedEntityKindTruthRead,
    BoundedFrontierAdjacencyTruthRead, BoundedFrontierFieldEqualityTruthRead,
    BoundedRelationKindTruthRead, EntityKindTruthReadLimitExceeded,
    FrontierAdjacencyTruthReadLimitExceeded, FrontierFieldEqualityTruthReadLimitExceeded,
    PositionedRelationalSnapshot, QueryLeasedReadOutcome, QueryReadExecutionStop,
    QueryReadPacketDenial, RelationKindTruthReadDenial, RelationKindTruthReadLimitExceeded,
    RelationalPinnedRead, RelationalPinnedReadAdmissionDenial,
    RelationalSnapshotPositionAdmissionStop, SnapshotPositionDenial, VisibilityReadContext,
};

use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    pub(crate) fn visibility_reads(&self) -> VisibilityReadContext<'_> {
        VisibilityReadContext::new(self)
    }
}

pub(crate) mod read_stop;
pub(crate) use projection::ProjectionRecordSource;

pub use reader::BoundedFrontierReadStop;
