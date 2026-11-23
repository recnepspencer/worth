use super::PerformanceAccess;
use crate::transactions::data::BulkMutationLocalityFootprint;

impl PerformanceAccess<'_> {
    pub(crate) fn count_bulk_mutation_plan(
        &self,
        locality: &BulkMutationLocalityFootprint,
        normalized_client_key_count: usize,
        lineage_transition_count: usize,
        provenance_record_count: usize,
    ) {
        self.counters.count(|counters| {
            counters.bulk_mutation_batch_count += 1;
            counters.bulk_mutation_entity_target_count += locality.entity_target_count;
            counters.bulk_mutation_relation_target_count += locality.relation_target_count;
            counters.bulk_mutation_cross_partition_relation_count +=
                locality.cross_partition_relation_count;
            counters.bulk_mutation_naming_normalization_count += normalized_client_key_count;
            counters.bulk_mutation_lineage_transition_count += lineage_transition_count;
            counters.bulk_mutation_provenance_record_count += provenance_record_count;
        });
    }
}
