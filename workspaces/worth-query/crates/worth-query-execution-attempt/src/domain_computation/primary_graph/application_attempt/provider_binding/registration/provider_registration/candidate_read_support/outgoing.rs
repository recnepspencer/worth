//! Touch-filtered outgoing reclosure (PR C §4). Outgoing selector support is
//! query support: it carries no touch key (brief §1.4), so its filter is the
//! conservative one, an exact comparison of each live entry from its own pin
//! to the candidate leaving the wave, charged per entry. A completed row whose
//! every live outgoing entry is equal there selects exactly what it selected,
//! so it keeps that support and is not re-closed; every other row is.
use super::{
    charge_scan, log_work, CandidateReadSupport, Denial, Kind,
    WorthQueryPrimaryGraphApplicationAttempt,
};
use worth_execution::ExecutionOrderedMap;

impl WorthQueryPrimaryGraphApplicationAttempt {
    /// Keep the outgoing support of the wave tasks whose every live outgoing
    /// entry is equal at `current`, and return them in order. Every other
    /// outgoing entry (changed, superseded, or with no owner) is removed: the
    /// wave re-closes its row before returning. An entry with no owner names
    /// no row, so it keeps nothing (full reclosure).
    pub(in crate::domain_computation::primary_graph) fn refresh_outgoing_read_support(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        before: &worth_relational::facade::snapshots::SnapshotHandle,
        current: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) -> Result<Vec<usize>, Denial> {
        let subject = "outgoing discovery support refresh";
        let stopped = |cause| match cause {
            worth_execution::ExecutionOrderedMapDenial::Allocation(cause) => {
                Denial::allocation_denied(subject, cause)
            }
            worth_execution::ExecutionOrderedMapDenial::Request(cause) => {
                Denial::relational_read_stopped(cause.into(), subject)
            }
            worth_execution::ExecutionOrderedMapDenial::CapacityOverflow => {
                Denial::new(Kind::CandidateCapacityExceeded, subject)
            }
        };
        let supports = &mut self.candidate_read_support;
        let mut touched = ExecutionOrderedMap::<usize, ()>::default();
        let mut equal = ExecutionOrderedMap::<usize, ()>::default();
        let mut unowned = false;
        for entry in &supports.values {
            let (CandidateReadSupport::Outgoing(_), None) = (&entry.support, entry.window_end)
            else {
                continue;
            };
            let Some(owner) = entry.owner else {
                unowned = true;
                continue;
            };
            charge_scan(execution, subject, log_work(touched.len()))?;
            if touched.contains_key(&owner) {
                continue;
            }
            charge_scan(execution, subject, 1)?;
            crate::domain_computation::execution_observation::record_retained_reads_visited(1);
            let changed = match entry
                .support
                .changed_fact(execution, runtime, before, entry.support.pin(), current)
            {
                Ok(changed) => changed.is_some(),
                // Unavailable at this image: re-close the row.
                Err(cause) if !crate::domain_computation::primary_graph::provider::application_attempt_state::candidate_read_support::is_hard_comparison_stop(&cause) => true,
                Err(cause) => return Err(cause),
            };
            let into = if changed { &mut touched } else { &mut equal };
            charge_scan(execution, subject, log_work(into.len()))?;
            if !into.contains_key(&owner) {
                into.insert_cloned_native_for_request(&owner, &(), *execution)
                    .map_err(stopped)?;
            }
        }
        let mut kept = Vec::new();
        if !unowned {
            for (&owner, ()) in equal.iter() {
                charge_scan(execution, subject, log_work(touched.len()))?;
                if !touched.contains_key(&owner) {
                    kept.push(owner);
                }
            }
        }
        let count = supports.values.len();
        supports.values.retain(|entry| {
            !matches!(entry.support, CandidateReadSupport::Outgoing(_))
                || (entry.window_end.is_none()
                    && entry
                        .owner
                        .is_some_and(|owner| kept.binary_search(&owner).is_ok()))
        });
        if supports.values.len() != count {
            // Slots moved: index the kept entries again.
            supports.by_read = ExecutionOrderedMap::default();
            supports.by_owner = ExecutionOrderedMap::default();
            let values = std::mem::take(&mut supports.values);
            for (slot, entry) in values.iter().enumerate() {
                supports.index(execution, slot, entry.owner, &entry.support, subject)?;
            }
            supports.values = values;
        }
        Ok(kept)
    }
}
