//! Count terminal release outcomes on their actual Runtime owner.
use crate::history::retention::RelationalExternalRetentionTerminalCounts;
use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    /// Read-only count of explicit and dropped external retention releases.
    #[doc(hidden)]
    pub fn external_retention_terminal_counts_for_test(
        &self,
    ) -> RelationalExternalRetentionTerminalCounts {
        self.services
            .instrumentation
            .external_retention_terminal_accounting()
            .snapshot()
    }
}
