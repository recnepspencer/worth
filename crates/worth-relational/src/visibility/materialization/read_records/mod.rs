pub(crate) use worth_relational_branch::internal_port::visibility::materialization::read_records::native_scan_stop;
pub(crate) mod projection;
pub(crate) use worth_relational_branch::internal_port::visibility::materialization::read_records::read_footprint;
pub(crate) use worth_relational_branch::internal_port::visibility::materialization::read_records::read_prices;
pub(crate) use worth_relational_branch::internal_port::visibility::materialization::read_records::read_request;
pub(crate) use worth_relational_branch::internal_port::visibility::materialization::read_records::read_work;
pub(crate) mod reader;

#[cfg(test)]
pub(crate) use projection::{
    entity_query_locus_comparison_key, entity_query_locus_value,
    relation_query_locus_comparison_key, relation_query_locus_value,
};
pub use projection::{
    AdjacencyStructuralRevision, AdjacencyStructuralRevisionDenial, EntityProjectionRecord,
    EntityRecordProjection, ProjectionAspectFilter, ProjectionAspectFilterMode,
    ProjectionAspectRequirement, ProjectionAspectScope, ProjectionContractRefusal,
    ProjectionRecordDomain, RelationProjectionRecord, RelationRecordProjection,
    RelationalAdjacencyDirection, RelationalAdjacencySupportDenial, RelationalAdjacencyVisit,
    RelationalAspectPresenceSupportDenial, RelationalBorrowedRecordReadDenial,
    RelationalEntityMetadata, RelationalEntityRetirement, RelationalFieldSupportComparisonDenial,
    RelationalProjectedFieldObservation, RelationalProjectionContractDenial,
    RelationalRelationMetadata, VisibilityProjectionView,
};
pub use reader::{
    AdjacencyTruthReadLimitExceeded, BoundedAdjacencyTruthRead, BoundedEntityKindTruthRead,
    BoundedFrontierAdjacencyTruthRead, BoundedFrontierFieldEqualityTruthRead,
    BoundedRelationKindTruthRead, EntityKindTruthReadLimitExceeded,
    FrontierAdjacencyTruthReadLimitExceeded, FrontierFieldEqualityTruthReadLimitExceeded,
    PinnedStructuralWalk, PinnedStructuralWalkDenial, PinnedStructuralWalkDenialKind,
    PinnedStructuralWalkReads, PinnedStructuralWalkScope, PinnedStructuralWalkVisit,
    PositionedRelationalSnapshot, QueryLeasedReadOutcome, QueryReadExecutionStop,
    QueryReadPacketDenial, RelationKindTruthReadDenial, RelationKindTruthReadLimitExceeded,
    RelationalAppliedCandidateReadIdentity, RelationalAppliedCandidateReadQualification,
    RelationalEntityMetadataPinDenial, RelationalPinnedEntityMetadata, RelationalPinnedRead,
    RelationalPinnedReadAdmissionDenial, RelationalSnapshotPositionAdmissionStop,
    SnapshotPositionDenial,
};

pub(crate) use worth_relational_branch::internal_port::visibility::materialization::read_records::read_stop;

pub use reader::BoundedFrontierReadStop;

#[cfg(test)]
pub(crate) use worth_relational_branch::internal_port::visibility::materialization::read_records::materialization;
