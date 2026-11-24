mod adjacency_demand;
mod adjacency_revision_read;
mod adjacency_support;
pub use adjacency_support::RelationalAdjacencySupportDenial;
mod admitted_snapshot;
mod aspect_field_support;
mod aspect_presence_support;
mod aspect_versions;
pub use aspect_presence_support::RelationalAspectPresenceSupportDenial;
mod basis_reads;
mod borrowed_adjacency;
mod borrowed_records;
mod checkpoint_context;
pub(crate) mod checkpoint_selection;
pub(crate) mod contracts;
mod creation;
pub(crate) use checkpoint_context::CheckpointProjectionContext;
pub(crate) use creation::ProjectionInputs;
mod current_adjacency;
mod entity_adjacency;
mod entity_projection;
mod entity_retirement;
pub(in crate::visibility::materialization::read_records) mod exact_basis_reads;
mod field_revisions;
mod field_support;
pub use field_support::RelationalFieldSupportComparisonDenial;
mod frontier_adjacency;
mod historical_basis_reads;
mod kind_scans;
pub(in crate::visibility::materialization::read_records) mod pinned;
pub(crate) mod projection_records;
pub(crate) mod query_locus_projection;
mod read_record_identity_ordering;
mod record_source;
mod registered_snapshot_projection;
mod residency;
mod streamed_basis_reads;
pub(crate) mod view;

pub use adjacency_revision_read::{
    AdjacencyStructuralRevision, AdjacencyStructuralRevisionDenial, RelationalAdjacencyDirection,
};
pub use borrowed_adjacency::RelationalAdjacencyVisit;
pub use borrowed_records::{
    RelationalBorrowedRecordReadDenial, RelationalEntityMetadata, RelationalRelationMetadata,
};
pub use contracts::{
    ProjectionAspectFilter, ProjectionAspectFilterMode, ProjectionAspectRequirement,
    ProjectionAspectScope, ProjectionContractRefusal, ProjectionRecordDomain,
    RelationalProjectionContractDenial,
};
pub use entity_retirement::RelationalEntityRetirement;
pub use projection_records::{
    EntityProjectionRecord, EntityRecordProjection, RelationProjectionRecord,
    RelationRecordProjection, RelationalProjectedFieldObservation,
};
pub(crate) use query_locus_projection::{
    authoritative_state_query_locus_comparison_key, authoritative_state_query_locus_value,
};
#[cfg(test)]
pub(crate) use query_locus_projection::{
    entity_query_locus_comparison_key, entity_query_locus_value,
    relation_query_locus_comparison_key, relation_query_locus_value,
};
pub use view::VisibilityProjectionView;

mod record_projections;
pub(crate) use streamed_basis_reads::ProjectionRecordSource;
