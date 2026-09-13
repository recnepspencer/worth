use super::super::ResourceRuntimeState;
use crate::data::resource::*;
use crate::data::telemetry::ResourceTelemetry;
use crate::state::SignalBranchId;

struct RekeyedResourceRestoreState {
    restored_in_flight_width: u32,
}

impl ResourceRuntimeState {
    pub fn bump_restore_epoch(
        &mut self,
        branch_id: SignalBranchId,
        telemetry: &mut ResourceTelemetry,
    ) -> ResourceBranchRestoreReport {
        let rekeyed = self.rekey_resource_state(branch_id);
        let restored_in_flight_width = rekeyed.restored_in_flight_width;
        let retained_summary_width = self.retained_restore_summary_width();
        let broad_rebuild_denial_count = 1;
        telemetry.resource_branch_restore_count += 1;
        telemetry.resource_branch_restore_in_flight_width = telemetry
            .resource_branch_restore_in_flight_width
            .max(restored_in_flight_width as u64);
        telemetry.resource_branch_restore_retained_summary_width = telemetry
            .resource_branch_restore_retained_summary_width
            .max(retained_summary_width as u64);
        telemetry.resource_branch_restore_broad_rebuild_denial_count = telemetry
            .resource_branch_restore_broad_rebuild_denial_count
            .saturating_add(broad_rebuild_denial_count as u64);
        let performance = Self::record_boundary_performance(
            telemetry,
            ResourceBoundaryPerformanceEnvelope::branch_restore(
                restored_in_flight_width,
                retained_summary_width,
                broad_rebuild_denial_count,
            ),
        );
        let report = ResourceBranchRestoreReport::new(
            restored_in_flight_width,
            retained_summary_width,
            broad_rebuild_denial_count,
            performance,
        );
        self.latest_branch_restore_report = Some(report);
        report
    }

    fn rekey_resource_state(&mut self, branch_id: SignalBranchId) -> RekeyedResourceRestoreState {
        self.restore_epoch = self.restore_epoch.saturating_add(1);
        let branch_epoch = ResourceBranchEpoch::new(branch_id, self.restore_epoch);
        self.in_flight_by_request = self
            .in_flight_by_request
            .iter()
            .map(|(request, in_flight)| {
                let mut in_flight = in_flight.clone();
                in_flight.refresh_branch_epoch(branch_epoch);
                (*request, in_flight)
            })
            .collect();
        self.retained_in_flight_history_by_request = self
            .retained_in_flight_history_by_request
            .iter()
            .map(|(request, retained)| {
                let mut retained = retained.clone();
                retained.refresh_branch_epoch(branch_epoch);
                (*request, retained)
            })
            .collect();
        self.pruned_in_flight_history_by_request = self
            .pruned_in_flight_history_by_request
            .iter()
            .map(|(request, pruned)| (*request, pruned.clone().with_branch_epoch(branch_epoch)))
            .collect();
        self.pending_retry_by_request = self
            .pending_retry_by_request
            .iter()
            .map(|(request, scheduled)| {
                (
                    *request,
                    scheduled
                        .clone()
                        .with_previous(scheduled.previous().with_branch_epoch(branch_epoch)),
                )
            })
            .collect();
        self.retained_retry_lineage_by_ordinal = self
            .retained_retry_lineage_by_ordinal
            .iter()
            .map(|(ordinal, retained)| (*ordinal, retained.clone().with_branch_epoch(branch_epoch)))
            .collect();
        self.pruned_retry_lineage_by_ordinal = self
            .pruned_retry_lineage_by_ordinal
            .iter()
            .map(|(ordinal, pruned)| (*ordinal, pruned.clone().with_branch_epoch(branch_epoch)))
            .collect();
        self.rebuild_pending_retry_by_node_index();
        RekeyedResourceRestoreState {
            restored_in_flight_width: self.in_flight_by_request.len() as u32,
        }
    }

    fn rebuild_pending_retry_by_node_index(&mut self) {
        self.pending_retry_by_node = self
            .pending_retry_by_request
            .values()
            .cloned()
            .filter_map(|scheduled| {
                self.in_flight_by_request
                    .get(&scheduled.previous().request_id())
                    .map(|in_flight| (in_flight.node(), scheduled))
            })
            .collect();
    }

    fn retained_restore_summary_width(&self) -> u32 {
        self.lifecycle_by_node
            .len()
            .saturating_add(self.denied_completions.len())
            .saturating_add(self.pruned_denied_completions_by_id.len())
            .saturating_add(self.retained_retry_lineage_by_ordinal.len())
            .saturating_add(self.pruned_retry_lineage_by_ordinal.len())
            .saturating_add(self.retained_in_flight_history_by_request.len())
            .saturating_add(self.pruned_in_flight_history_by_request.len()) as u32
    }
}
