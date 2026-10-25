use crate::performance::PerformanceAccess;
use std::collections::BTreeSet;

pub(crate) struct InvariantMetrics<'runtime> {
    access: PerformanceAccess<'runtime>,
}

impl<'runtime> InvariantMetrics<'runtime> {
    pub(crate) fn new(access: PerformanceAccess<'runtime>) -> Self {
        Self { access }
    }

    pub(crate) fn count_entity_slot_scans(&self, slots: usize) {
        self.access.count_invariant_entity_slot_scans(slots);
    }

    pub(crate) fn count_relation_slot_scans(&self, slots: usize) {
        self.access.count_invariant_relation_slot_scans(slots);
    }

    pub(crate) fn count_relation_contracts_evaluated(&self, count: usize) {
        self.access
            .count_relation_integrity_contracts_evaluated(count);
    }

    pub(crate) fn count_relation_endpoint_kind_checks(&self, count: usize) {
        self.access.count_relation_endpoint_kind_checks(count);
    }

    pub(crate) fn count_relation_cardinality_checks(&self, count: usize) {
        self.access.count_relation_cardinality_checks(count);
    }

    pub(crate) fn count_relation_cardinality_minimum_certification(
        &self,
        contracts: usize,
        entity_slot_scans: usize,
        relation_slot_scans: usize,
    ) {
        self.access
            .count_relation_cardinality_minimum_certification(
                contracts,
                entity_slot_scans,
                relation_slot_scans,
            );
    }

    pub(crate) fn count_relation_uniqueness_checks(&self, count: usize) {
        self.access.count_relation_uniqueness_checks(count);
    }

    pub(crate) fn count_relation_symmetry_checks(&self, count: usize) {
        self.access.count_relation_symmetry_checks(count);
    }

    pub(crate) fn count_relation_endpoint_deletion_checks(&self, count: usize) {
        self.access.count_relation_endpoint_deletion_checks(count);
    }
}

impl InvariantMetrics<'_> {
    pub(crate) fn record_preparation_plan(
        &self,
        planned: &crate::authority::commit::preparation::facade::PreparedInvariantExecution<'_>,
    ) {
        let counters = crate::validation::execution::planned_packet_counters(planned);
        let scope_units = if planned.packets.iter().any(|packet| {
            matches!(
                packet.locality.partition_scope,
                crate::authority::commit::preparation::proofs::locality::PreparationPartitionScope::AllObserved
            )
        }) {
            1
        } else {
            let mut touched = BTreeSet::new();
            for packet in &planned.packets {
                if let crate::authority::commit::preparation::proofs::locality::PreparationPartitionScope::TouchedPartitions(
                    partitions,
                ) = &packet.locality.partition_scope
                {
                    touched.extend(partitions.iter().copied());
                }
            }
            touched.len()
        };
        self.record_preparation_shape(counters.packet_count, scope_units, planned.strategy);
        debug_assert!(planned
            .packets
            .iter()
            .all(|packet| packet.planning_context == planned.context));
    }

    pub(crate) fn record_preparation_shape(
        &self,
        packet_count: usize,
        scope_units: usize,
        strategy: crate::authority::commit::preparation::planning::strategy::PreparationStrategy,
    ) {
        let performance = &self.access;
        performance.count_preparation_packet_shape(
            packet_count,
            packet_count,
            usize::from(packet_count > 0),
            scope_units,
        );
        match strategy.parallel_legality {
            crate::authority::commit::preparation::planning::strategy::ParallelLegality::ProvenParallel => {
                performance.count_preparation_parallel_legal();
            }
            crate::authority::commit::preparation::planning::strategy::ParallelLegality::RequiresSerial => {}
        }
        match strategy.parallel_profitability {
            crate::authority::commit::preparation::planning::strategy::ParallelProfitability::Profitable => {
                performance.count_preparation_parallel_profitable();
            }
            crate::authority::commit::preparation::planning::strategy::ParallelProfitability::NotProfitable => {}
        }
        match strategy.selected_mode {
            crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection::Serial => {
                performance.count_preparation_serial_strategy();
            }
            crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection::StagedParallel => {
                performance.count_preparation_staged_parallel_strategy();
            }
        }
    }
}
