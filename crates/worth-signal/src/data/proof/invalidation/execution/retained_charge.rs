use super::{
    FrontierDiagnosticsProjection, FrontierDiagnosticsSidecar, FrontierWaveEntrySummary,
    FrontierWaveSummary, InvalidationTraceRecord, TransitiveFrontierEntrySummary,
    TransitiveFrontierWaveSummary,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for FrontierWaveEntrySummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            node: _,
            classification: _,
            inclusion_basis: _,
            narrowed_scopes,
        } = self;
        narrowed_scopes.retained_heap_charge(work)
    }
}
impl RetainedStorageMeasurement for FrontierWaveSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            wave_index: _,
            aspect: _,
            entries,
        } = self;
        entries.retained_heap_charge(work)
    }
}
impl RetainedStorageMeasurement for TransitiveFrontierWaveSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            wave_index: _,
            entries,
        } = self;
        entries.retained_heap_charge(work)
    }
}
impl RetainedStorageMeasurement for TransitiveFrontierEntrySummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            node: _,
            classification: _,
            inclusion_basis: _,
        } = self;
        Ok(Charge::ZERO)
    }
}
impl RetainedStorageMeasurement for FrontierDiagnosticsProjection {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            frontier_seed_count: _,
            frontier_group_count: _,
            frontier_direct_wave_count: _,
            frontier_transitive_wave_count: _,
            frontier_partition_scoped_check_count: _,
            frontier_direct_dirty_count: _,
            frontier_maybe_stale_count: _,
            frontier_partition_match_count: _,
            frontier_detail_match_count: _,
            frontier_cycle_check_candidate_count: _,
            frontier_cycle_check_visited_count: _,
            frontier_trace_retained_count: _,
        } = self;
        Ok(Charge::ZERO)
    }
}
impl RetainedStorageMeasurement for FrontierDiagnosticsSidecar {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            seed_count: _,
            direct_waves,
            transitive_waves,
            touched_scope_summary,
            counters,
        } = self;
        direct_waves
            .retained_heap_charge(work)?
            .checked_add(transitive_waves.retained_heap_charge(work)?)?
            .checked_add(touched_scope_summary.retained_heap_charge(work)?)?
            .checked_add(counters.retained_heap_charge(work)?)
    }
}
impl RetainedStorageMeasurement for InvalidationTraceRecord {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            node: _,
            aspect: _,
            wave_index: _,
            classification: _,
            inclusion_basis: _,
        } = self;
        Ok(Charge::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostic_frontier_charge_includes_nested_wave_capacity() {
        let mut graph = crate::facade::SignalGraph::new();
        let node = graph.node().build();
        let mut direct_entries = Vec::with_capacity(8);
        direct_entries.push(FrontierWaveEntrySummary {
            node,
            classification: Default::default(),
            inclusion_basis: Default::default(),
            narrowed_scopes: Default::default(),
        });
        let mut transitive_entries = Vec::with_capacity(16);
        transitive_entries.push(TransitiveFrontierEntrySummary {
            node,
            classification: Default::default(),
            inclusion_basis: Default::default(),
        });
        let mut summary = FrontierDiagnosticsSidecar::new(
            1,
            vec![FrontierWaveSummary {
                wave_index: 0,
                aspect: crate::data::aspect::Aspect::new(0),
                entries: direct_entries,
            }],
            vec![TransitiveFrontierWaveSummary {
                wave_index: 1,
                entries: transitive_entries,
            }],
            Default::default(),
            Default::default(),
        );
        let original = summary.clone();
        let before = summary.retained_heap_charge(&mut Work::new(1000)).unwrap();
        let mut delta = 0;
        delta += grow(&mut summary.direct_waves);
        delta += grow(&mut summary.transitive_waves);
        delta += grow(&mut summary.direct_waves[0].entries);
        delta += grow(&mut summary.transitive_waves[0].entries);
        assert_eq!(
            summary
                .retained_heap_charge(&mut Work::new(1000))
                .unwrap()
                .bytes()
                - before.bytes(),
            delta
        );
        assert_eq!(summary, original);
    }
    fn grow<T>(values: &mut Vec<T>) -> u64 {
        let before = values.capacity();
        values.reserve_exact(128);
        ((values.capacity() - before) * std::mem::size_of::<T>()) as u64
    }
}
