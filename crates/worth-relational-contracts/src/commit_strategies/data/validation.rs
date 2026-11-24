#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StrategyPreviewValidationCostSummary {
    preview_version_id: crate::identity::data::VersionId,
    merged_intent_count: usize,
    touched_partition_count: usize,
    bulk_entity_slots_reserved: usize,
    bulk_relation_slots_reserved: usize,
    post_mutation_preview_pass_count: usize,
}

impl StrategyPreviewValidationCostSummary {
    pub fn new(
        preview_version_id: crate::identity::data::VersionId,
        merged_intent_count: usize,
        touched_partition_count: usize,
        bulk_entity_slots_reserved: usize,
        bulk_relation_slots_reserved: usize,
        post_mutation_preview_pass_count: usize,
    ) -> Self {
        Self {
            preview_version_id,
            merged_intent_count,
            touched_partition_count,
            bulk_entity_slots_reserved,
            bulk_relation_slots_reserved,
            post_mutation_preview_pass_count,
        }
    }

    pub fn preview_version_id(&self) -> crate::identity::data::VersionId {
        self.preview_version_id
    }

    pub fn merged_intent_count(&self) -> usize {
        self.merged_intent_count
    }

    pub fn touched_partition_count(&self) -> usize {
        self.touched_partition_count
    }

    pub fn bulk_entity_slots_reserved(&self) -> usize {
        self.bulk_entity_slots_reserved
    }

    pub fn bulk_relation_slots_reserved(&self) -> usize {
        self.bulk_relation_slots_reserved
    }

    pub fn post_mutation_preview_pass_count(&self) -> usize {
        self.post_mutation_preview_pass_count
    }
}
