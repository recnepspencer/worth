//! A readmitted writeback archive begins without process-local records.
use super::*;
use crate::diagnostics::writeback_retention::WritebackDiagnosticScope;
use worth_foundational::facade::DurableSignalOwnerIdentity;
impl BridgeDiagnosticsState {
    pub(crate) fn readmit_writeback_diagnostics(&mut self, owner: DurableSignalOwnerIdentity) {
        for slot in &self.writeback_execution_records {
            slot.expire();
        }
        for slot in &self.writeback_replay_records {
            slot.expire();
        }
        self.writeback_execution_records.clear();
        self.writeback_replay_records.clear();
        self.latest_writeback_execution_by_record_identity.clear();
        self.latest_writeback_execution_by_candidate_digest.clear();
        self.latest_writeback_replay_by_record_identity.clear();
        self.writeback_scope = Arc::new(WritebackDiagnosticScope { owner: Some(owner) });
    }
}
