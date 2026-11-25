//! Pure request and mutation fixtures for historical lineage courts.
use super::*;

pub(super) fn main_resolution_request(
    lineage_id: crate::facade::identity::LineageId,
) -> HistoricalResolutionRequest {
    HistoricalResolutionRequest {
        branch_id: BranchId("main".to_string()),
        lineage_id,
        boundedness_basis: HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
    }
}

pub(super) fn main_graph_request() -> LineageGraphRequest {
    LineageGraphRequest {
        branch_id: BranchId("main".to_string()),
        traversal_basis: LineageGraphTraversalBasis::FullBranchGraphMaterialization,
    }
}

pub(super) fn replace_batch_with(
    entity_id: crate::facade::identity::EntityId,
    batch_name: &str,
    label: &str,
) -> WorkerIntentBatch {
    WorkerIntentBatch::new(batch_name).push(MutationIntent::Entity(EntityMutationIntent::Replace(
        ReplaceEntityIntent {
            entity_id,
            replacement: crate::transactions::data::EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw(label),
                fields: single_string_aspect_field_patch(
                    aspect_key("name"),
                    field_key("name"),
                    label,
                ),
            },
        },
    )))
}

pub(super) fn replace_batch(entity_id: crate::facade::identity::EntityId) -> WorkerIntentBatch {
    replace_batch_with(entity_id, "replace", "replacement")
}
