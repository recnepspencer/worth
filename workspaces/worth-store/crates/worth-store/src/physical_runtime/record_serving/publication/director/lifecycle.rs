use std::sync::Arc;

use super::{RecordPublicationDirector, RecordPublicationTerminalState};

impl RecordPublicationDirector {
    pub(in crate::physical_runtime) fn retention_checkpoint_due(&self) -> bool {
        let (charged, limit) = self.root_owner.publication_admission().growth_usage();
        if charged.saturating_mul(2) < limit {
            return false;
        }
        self.wal.has_reclaimable_closed_prefix()
    }

    pub(in crate::physical_runtime) fn stop_and_extract(
        director: Arc<Self>,
    ) -> RecordPublicationTerminalState {
        let mutations = director.mutations.stop_and_drain();
        let director = Arc::try_unwrap(director)
            .unwrap_or_else(|_| unreachable!("submission capabilities retain only weak authority"));
        let wal_tail = director.wal.recovery_tail();
        let wal_observation = director.wal.observation();
        let performance_witness = director.durability_policy_basis.physical_witness();
        let roots = director.root_owner.into_recovery_root_basis();
        RecordPublicationTerminalState {
            residue: director.residue,
            mutations,
            roots,
            wal_tail,
            wal_observation,
            performance_witness,
        }
    }
}
