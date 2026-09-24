use super::PerformanceAccess;

impl PerformanceAccess<'_> {
    pub(crate) fn count_invariant_entity_slot_scans(&self, slots: usize) {
        self.runtime
            .services
            .instrumentation
            .count(|counters| counters.invariant_entity_slot_scans += slots);
    }

    pub(crate) fn count_invariant_relation_slot_scans(&self, slots: usize) {
        self.runtime
            .services
            .instrumentation
            .count(|counters| counters.invariant_relation_slot_scans += slots);
    }

    pub(crate) fn count_custom_invariant_preparation(&self) {
        self.runtime
            .services
            .instrumentation
            .count(|counters| counters.custom_invariant_preparation_count += 1);
    }

    pub(crate) fn count_custom_invariant_execution(&self) {
        self.runtime
            .services
            .instrumentation
            .count(|counters| counters.custom_invariant_execution_count += 1);
    }

    pub(crate) fn count_custom_invariant_panic(&self) {
        self.runtime
            .services
            .instrumentation
            .count(|counters| counters.custom_invariant_panic_count += 1);
    }

    pub(crate) fn count_custom_invariant_traversal(&self, frontier: usize, steps: usize) {
        self.runtime.services.instrumentation.count(|counters| {
            counters.custom_invariant_traversal_frontier_count += frontier;
            counters.custom_invariant_traversal_step_count += steps;
        });
    }

    pub(crate) fn count_custom_invariant_candidate_inputs(
        &self,
        entity_reads: usize,
        relation_reads: usize,
        adjacency_gathers: usize,
        adjacency_count_reads: usize,
        adjacency_relation_ids: usize,
        reuse_hits: usize,
    ) {
        self.runtime.services.instrumentation.count(|counters| {
            counters.custom_invariant_candidate_entity_reads += entity_reads;
            counters.custom_invariant_candidate_relation_reads += relation_reads;
            counters.custom_invariant_candidate_adjacency_gathers += adjacency_gathers;
            counters.custom_invariant_candidate_adjacency_count_reads += adjacency_count_reads;
            counters.custom_invariant_candidate_adjacency_relation_ids += adjacency_relation_ids;
            counters.custom_invariant_candidate_reuse_hits += reuse_hits;
        });
    }

    pub(crate) fn count_relation_integrity_contracts_evaluated(&self, count: usize) {
        self.runtime
            .services
            .instrumentation
            .count(|counters| counters.relation_integrity_contracts_evaluated += count);
    }

    pub(crate) fn count_relation_endpoint_kind_checks(&self, count: usize) {
        self.runtime
            .services
            .instrumentation
            .count(|counters| counters.relation_endpoint_kind_checks += count);
    }

    pub(crate) fn count_relation_cardinality_checks(&self, count: usize) {
        self.runtime
            .services
            .instrumentation
            .count(|counters| counters.relation_cardinality_checks += count);
    }

    pub(crate) fn count_relation_cardinality_minimum_certification(
        &self,
        contracts: usize,
        entity_slot_scans: usize,
        relation_slot_scans: usize,
    ) {
        self.runtime.services.instrumentation.count(|counters| {
            counters.relation_cardinality_minimum_certification_contracts_evaluated += contracts;
            counters.relation_cardinality_minimum_certification_entity_slot_scans +=
                entity_slot_scans;
            counters.relation_cardinality_minimum_certification_relation_slot_scans +=
                relation_slot_scans;
        });
    }

    pub(crate) fn count_relation_uniqueness_checks(&self, count: usize) {
        self.runtime
            .services
            .instrumentation
            .count(|counters| counters.relation_uniqueness_checks += count);
    }

    pub(crate) fn count_relation_uniqueness_candidates(&self, count: usize) {
        self.runtime
            .services
            .instrumentation
            .count(|counters| counters.relation_uniqueness_candidates_scanned += count);
    }

    pub(crate) fn count_relation_symmetry_checks(&self, count: usize) {
        self.runtime
            .services
            .instrumentation
            .count(|counters| counters.relation_symmetry_checks += count);
    }

    pub(crate) fn count_relation_endpoint_deletion_checks(&self, count: usize) {
        self.runtime
            .services
            .instrumentation
            .count(|counters| counters.relation_endpoint_deletion_checks += count);
    }
}
