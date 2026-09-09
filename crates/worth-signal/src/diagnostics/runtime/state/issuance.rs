use super::DiagnosticsState;

impl DiagnosticsState {
    /// Rejected work can retain issued diagnostic identities. Restore contents
    /// independently, but never issue those identities again on a later attempt.
    /// This fixed-size operation does not merge rejected history or indexes.
    pub(crate) fn preserve_issuance_from(&mut self, performed: &Self) {
        let (artifact, sequence) = performed.lineage_allocator_state();
        self.synchronize_lineage_allocator(artifact, sequence);
        let (snapshot, branch) = performed.branch_snapshot_allocator_state();
        self.synchronize_branch_snapshot_allocator(snapshot, branch);
        self.next_replay_cursor = self.next_replay_cursor.max(performed.next_replay_cursor);
    }
}
