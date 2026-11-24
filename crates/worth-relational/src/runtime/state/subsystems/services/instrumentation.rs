// Diagnostic counters keep their valid numeric prefix on unwind; they grant no authority.
use std::sync::{Arc, Mutex};

use crate::performance::data::runtime_complexity_counters::RuntimeComplexityCounters;

#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeInstrumentation {
    pub(crate) complexity_counters: Arc<Mutex<RuntimeComplexityCounters>>,
    basis_counters: Arc<Mutex<crate::branch::basis_counters::RelationalBranchBasisCostCounters>>,
    external_retention_terminals:
        Arc<crate::history::retention::RelationalExternalRetentionTerminalAccounting>,
}

impl RuntimeInstrumentation {
    pub(crate) fn count_descriptor_version_mismatch(&self) {
        self.count(RuntimeComplexityCounters::count_descriptor_version_mismatch);
    }

    pub(crate) fn preparation_round(&self) -> Self {
        Self {
            complexity_counters: Arc::default(),
            basis_counters: Arc::default(),
            external_retention_terminals: self.external_retention_terminals.clone(),
        }
    }
    pub(crate) fn new() -> Self {
        Self {
            complexity_counters: Arc::new(Mutex::new(RuntimeComplexityCounters::default())),
            basis_counters: Arc::new(Mutex::new(
                crate::branch::basis_counters::RelationalBranchBasisCostCounters::default(),
            )),
            external_retention_terminals: Arc::new(Default::default()),
        }
    }

    pub(crate) fn fork(&self) -> Self {
        Self {
            complexity_counters: Arc::new(Mutex::new(
                self.complexity_counters
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .clone(),
            )),
            basis_counters: Arc::new(Mutex::new(self.basis_counters())),
            external_retention_terminals: Arc::new(Default::default()),
        }
    }

    pub(crate) fn complexity_snapshot(&self) -> RuntimeComplexityCounters {
        crate::performance::complexity_counter_sink::ComplexityCounterSink::new(
            &self.complexity_counters,
        )
        .snapshot()
    }

    pub(crate) fn count(&self, update: impl FnOnce(&mut RuntimeComplexityCounters)) {
        update(
            &mut self
                .complexity_counters
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
        );
    }

    pub(crate) fn count_query_packet_shape(
        &self,
        packets: usize,
        items: usize,
        max_width: usize,
        scope_units: usize,
    ) {
        self.count(|counters| {
            counters.query_packet_count += packets;
            counters.query_packet_item_count += items;
            counters.query_packet_peak_width_total += max_width;
            counters.query_scope_unit_count += scope_units;
        });
    }

    pub(crate) fn count_query_parallel_legal(&self) {
        self.count(|counters| counters.query_parallel_legal_count += 1);
    }

    pub(crate) fn count_query_parallel_profitable(&self) {
        self.count(|counters| counters.query_parallel_profitable_count += 1);
    }

    pub(crate) fn count_query_fragment_scratch_reuse_by(&self, count: usize) {
        self.count(|counters| {
            counters.query_fragment_scratch_reuse_count += count;
        });
    }

    pub(crate) fn count_query_emissions(&self, entities: usize, relations: usize) {
        self.count(|counters| {
            counters.query_authoritative_entity_records_emitted += entities;
            counters.query_authoritative_relation_records_emitted += relations;
        });
    }

    pub(crate) fn count_query_serial_strategy(&self) {
        self.count(RuntimeComplexityCounters::count_query_serial_strategy);
    }

    pub(crate) fn count_query_staged_parallel_strategy(&self) {
        self.count(RuntimeComplexityCounters::count_query_staged_parallel_strategy);
    }

    pub(crate) fn count_query_index_attempt(&self) {
        self.count(RuntimeComplexityCounters::count_query_index_attempt);
    }

    pub(crate) fn count_query_index_path(&self) {
        self.count(RuntimeComplexityCounters::count_query_index_path);
    }

    pub(crate) fn count_query_index_parity_verification(&self) {
        self.count(RuntimeComplexityCounters::count_query_index_parity_verification);
    }

    pub(crate) fn count_basis(
        &self,
        update: impl FnOnce(&mut crate::branch::basis_counters::RelationalBranchBasisCostCounters),
    ) {
        update(
            &mut self
                .basis_counters
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
        );
    }

    pub(crate) fn basis_counters(
        &self,
    ) -> crate::branch::basis_counters::RelationalBranchBasisCostCounters {
        let mut counters = *self
            .basis_counters
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let terminals = self.external_retention_terminals.snapshot();
        counters.external_retention_releases = counters
            .external_retention_releases
            .saturating_add(terminals.total());
        counters.external_retention_drop_releases = counters
            .external_retention_drop_releases
            .saturating_add(terminals.dropped_releases);
        counters
    }

    pub(crate) fn external_retention_terminal_accounting(
        &self,
    ) -> Arc<crate::history::retention::RelationalExternalRetentionTerminalAccounting> {
        Arc::clone(&self.external_retention_terminals)
    }
}

mod bulk_mutation_counters;

mod merge_execution_counters;
