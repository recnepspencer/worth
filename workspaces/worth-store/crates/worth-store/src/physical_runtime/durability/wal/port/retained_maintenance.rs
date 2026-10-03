use super::maintenance::{synchronize_maintenance_interval, MaintenanceBarrierDenial};
use super::{DurableMaintenanceReceipt, PhysicalWalAppendPort, ScheduledMaintenanceDenial};
use sha2::{Digest, Sha256};

impl PhysicalWalAppendPort {
    pub(in crate::physical_runtime) fn recovered_copy_obligations(
        &self,
    ) -> Vec<crate::physical_runtime::durability::wal::RetainedExtentCopyObligation> {
        self.owner.recovered_copy_obligations()
    }
    /// Resynchronizes the exact original, verified and retention-held intent.
    /// Reopen facts alone cannot produce a receipt: this executes a fresh
    /// scheduler-admitted barrier and never changes the append frontier.
    pub(in crate::physical_runtime) fn synchronize_retained_maintenance_intent(
        &self,
        payload: &[u8],
    ) -> Result<DurableMaintenanceReceipt, ScheduledMaintenanceDenial> {
        let digest: [u8; 32] = Sha256::digest(payload).into();
        let record = crate::physical_runtime::durability::retention::decode_retirement(payload)
            .filter(|record| !record.completion && record.release.is_some())
            .ok_or(ScheduledMaintenanceDenial::Finish)?;
        let intent = {
            let state = self.owner.shared.lock().unwrap_or_else(|e| e.into_inner());
            if state.sealed {
                return Err(ScheduledMaintenanceDenial::Finish);
            }
            state
                .retained_maintenance
                .iter()
                .find(|intent| {
                    let (segment, generation, start, end, offset, bytes) = intent.interval();
                    intent.digest() == digest
                        && intent.record() == record
                        && state
                            .unresolved_retirement_spans
                            .iter()
                            .any(|hold| *hold == (record.artifact, start, end))
                        && state.segments.entries().iter().any(|entry| {
                            entry.identity().segment().get() == segment
                                && entry.identity().generation().get() == generation
                                && entry.lsn_range().start().get() <= start
                                && entry.lsn_range().end_exclusive().get() >= end
                                && offset
                                    .checked_add(bytes)
                                    .is_some_and(|end| end <= entry.byte_count())
                        })
                })
                .cloned()
        }
        .ok_or(ScheduledMaintenanceDenial::Finish)?;
        let interval = intent.interval();
        let synchronization = synchronize_maintenance_interval(self, intent.artifact(), interval)
            .map_err(|denial| match denial {
            MaintenanceBarrierDenial::Waiting => {
                ScheduledMaintenanceDenial::WrittenAwaitingBarrier { interval }
            }
            MaintenanceBarrierDenial::Failed => ScheduledMaintenanceDenial::Sync,
        })?;
        Ok(DurableMaintenanceReceipt::completed(
            digest,
            interval,
            None,
            synchronization,
        ))
    }
}
