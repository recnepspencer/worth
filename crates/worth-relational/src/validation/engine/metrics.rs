use crate::performance::complexity_counter_sink::ComplexityCounterSink;
use crate::performance::data::runtime_complexity_counters::RuntimeComplexityCounters;
use std::{collections::BTreeSet, sync::Mutex};

mod counters;

pub(crate) struct InvariantMetrics<'runtime> {
    counters: ComplexityCounterSink<'runtime>,
}

impl<'runtime> InvariantMetrics<'runtime> {
    pub(crate) fn new(counters: &'runtime Mutex<RuntimeComplexityCounters>) -> Self {
        Self {
            counters: ComplexityCounterSink::new(counters),
        }
    }
}

impl InvariantMetrics<'_> {
    pub(crate) fn accumulate_round_counts(&self, counts: RuntimeComplexityCounters) {
        self.counters.accumulate(counts);
    }

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
        debug_assert!(
            /* Proof: Each private packet constructor specializes only the selected family and its before/applied observation; every other field of the shared proposal proof must remain exact. */
            planned.packets.iter().all(|packet| planned
                .context
                .for_registration(packet.registration.execution_point())
                .as_ref()
                == Some(packet.planning_context.as_ref()))
        );
    }

    pub(crate) fn record_preparation_shape(
        &self,
        packet_count: usize,
        scope_units: usize,
        strategy: crate::authority::commit::preparation::planning::strategy::PreparationStrategy,
    ) {
        self.counters.count(|counts| {
            counts.count_preparation_packet_shape(
                packet_count,
                packet_count,
                usize::from(packet_count > 0),
                scope_units,
            );
        });
        match strategy.parallel_legality {
            crate::authority::commit::preparation::planning::strategy::ParallelLegality::ProvenParallel => {
                self.counters.count(RuntimeComplexityCounters::count_preparation_parallel_legal);
            }
            crate::authority::commit::preparation::planning::strategy::ParallelLegality::RequiresSerial => {}
        }
        match strategy.parallel_profitability {
            crate::authority::commit::preparation::planning::strategy::ParallelProfitability::Profitable => {
                self.counters.count(RuntimeComplexityCounters::count_preparation_parallel_profitable);
            }
            crate::authority::commit::preparation::planning::strategy::ParallelProfitability::NotProfitable => {}
        }
        match strategy.selected_mode {
            crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection::Serial => {
                self.counters.count(RuntimeComplexityCounters::count_preparation_serial_strategy);
            }
            crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection::StagedParallel => {
                self.counters.count(RuntimeComplexityCounters::count_preparation_staged_parallel_strategy);
            }
        }
    }
    pub(crate) fn count_preparation_reducer_conflicts(&self, conflicts: usize) {
        self.counters
            .count(|counts| counts.count_preparation_reducer_conflicts(conflicts));
    }
}
