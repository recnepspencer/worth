//! Borrowed complexity telemetry: global or provisional, with no new allocation.
// Diagnostic counters keep their valid numeric prefix on unwind; they grant no authority.
use crate::performance::data::RuntimeComplexityCounters;
use std::sync::Mutex;

pub(super) struct ComplexityCounterSink<'owner> {
    counters: &'owner Mutex<RuntimeComplexityCounters>,
}

impl<'owner> ComplexityCounterSink<'owner> {
    pub(super) fn new(counters: &'owner Mutex<RuntimeComplexityCounters>) -> Self {
        Self { counters }
    }

    pub(super) fn count(&self, update: impl FnOnce(&mut RuntimeComplexityCounters)) {
        update(
            &mut self
                .counters
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
        );
    }

    pub(super) fn snapshot(&self) -> RuntimeComplexityCounters {
        self.counters
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::performance::PerformanceAccess;

    #[test]
    fn abandoned_counts_do_not_escape_and_settlement_preserves_existing_gauges() {
        let parent = Mutex::new(RuntimeComplexityCounters {
            preparation_packet_count: 7,
            commit_topology_flags: 11,
            partitions_touched_by_commit: 3,
            ..Default::default()
        });
        {
            let pending = Mutex::new(RuntimeComplexityCounters::default());
            let sink = ComplexityCounterSink::new(&pending);
            sink.count(|counts| counts.preparation_packet_count += 5);
            assert_eq!(parent.lock().unwrap().preparation_packet_count, 7);
        }
        assert_eq!(parent.lock().unwrap().preparation_packet_count, 7);
        let accepted = RuntimeComplexityCounters {
            preparation_packet_count: 2,
            ..Default::default()
        };
        PerformanceAccess::from_counters(&parent).accumulate_round_counts(accepted);
        let actual = parent.lock().unwrap();
        assert_eq!(actual.preparation_packet_count, 9);
        assert_eq!(actual.commit_topology_flags, 11);
        assert_eq!(actual.partitions_touched_by_commit, 3);
    }
}
