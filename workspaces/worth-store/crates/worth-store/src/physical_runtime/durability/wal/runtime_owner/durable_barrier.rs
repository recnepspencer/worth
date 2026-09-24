use worth_store_wal::LogSequenceNumber;

use super::PhysicalWalRuntimeState;

impl PhysicalWalRuntimeState {
    pub(super) fn record_durable_barrier(
        &mut self,
        lsn_start: u64,
        lsn_end_exclusive: u64,
    ) -> bool {
        let expected_start = self
            .durable_lsn_end
            .or_else(|| self.segments.first_lsn_start())
            .map(LogSequenceNumber::get);
        let appended_end = self.frontier.last_lsn_end().map(LogSequenceNumber::get);
        if self.sealed
            || expected_start != Some(lsn_start)
            || appended_end.is_none_or(|end| end < lsn_end_exclusive)
            || lsn_start >= lsn_end_exclusive
        {
            self.sealed = true;
            return false;
        }
        self.durable_lsn_end = Some(LogSequenceNumber::new(lsn_end_exclusive));
        true
    }

    pub(super) fn checkpoint_source_range(
        &self,
    ) -> Option<worth_store_physical_format::CheckpointWalSourceRange> {
        if self.sealed {
            return None;
        }
        let begin = self.segments.first_lsn_start()?.get();
        let end = self.durable_lsn_end?.get();
        worth_store_physical_format::CheckpointWalSourceRange::new(begin, end)
    }
}
