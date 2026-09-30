use super::PerformanceAccess;
use crate::transactions::data::BulkMutationLocalityFootprint;

impl PerformanceAccess<'_> {
    pub(crate) fn count_working_state_clone(
        &self,
        partitions: usize,
        entity_slots: usize,
        relation_slots: usize,
    ) {
        self.runtime.services.instrumentation.count(|counters| {
            counters.partitions_cloned += partitions;
            counters.entity_slots_cloned += entity_slots;
            counters.relation_slots_cloned += relation_slots;
        });
    }

    pub(crate) fn count_aosoa_prepare_chunks(&self, chunk_count: usize, slot_count: usize) {
        self.runtime.services.instrumentation.count(|counters| {
            counters.aosoa_entity_chunks_staged += chunk_count;
            counters.aosoa_entity_chunk_slots_materialized += slot_count;
        });
    }

    pub(crate) fn count_aosoa_publish_soa_merge(&self, chunk_count: usize, slot_count: usize) {
        self.runtime.services.instrumentation.count(|counters| {
            counters.aosoa_entity_chunks_published += chunk_count;
            counters.aosoa_entity_slot_soa_merges += slot_count;
            counters.aosoa_publish_soa_merge_count += 1;
        });
    }

    pub(crate) fn count_aosoa_publish_chunks(&self, chunk_count: usize) {
        self.runtime.services.instrumentation.count(|counters| {
            counters.aosoa_entity_chunks_published += chunk_count;
        });
    }

    pub(crate) fn count_bulk_mutation_plan(
        &self,
        locality: &BulkMutationLocalityFootprint,
        normalized_client_key_count: usize,
        lineage_transition_count: usize,
        provenance_record_count: usize,
    ) {
        self.runtime.services.instrumentation.count(|counters| {
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

    pub(crate) fn count_post_commit_consumer_shape(
        &self,
        packets: usize,
        items: usize,
        max_width: usize,
        scope_units: usize,
    ) {
        self.runtime.services.instrumentation.count(|counters| {
            counters.post_commit_consumer_packet_count += packets;
            counters.post_commit_consumer_item_count += items;
            counters.post_commit_consumer_peak_width_total += max_width;
            counters.post_commit_scope_unit_count += scope_units;
        });
    }

    pub(crate) fn count_post_commit_serial_strategy(&self) {
        self.runtime
            .services
            .instrumentation
            .count(|counters| counters.post_commit_serial_strategy_count += 1);
    }
}
