//! Ordinary generation maintenance receives the original sealed publication.

use crate::runtime::RelationalRuntime;
use crate::transactions::data::CommitResult;

impl RelationalRuntime {
    /// Retire unselected derived index generations after this owner's settled
    /// publication. Work follows its affected index scopes,
    /// plus at most the configured retention batch of older generations.
    ///
    /// Release temporary before-basis and settlement holds first. Intentional
    /// readers and retained receipts keep their exact generations selectable.
    /// `None` defers maintenance while publication is not quiescent. Payloads
    /// are destroyed after releasing catalog and canonical publication locks.
    pub fn run_index_generation_reclamation_after_publication(
        &self,
        committed: &CommitResult,
    ) -> Result<Option<usize>, crate::mvcc::RelationalPublicationDenial> {
        let actual_runtime_instance_id = committed.snapshot.runtime_instance_id();
        if actual_runtime_instance_id != self.runtime_instance_id() {
            return Err(crate::mvcc::RelationalPublicationDenial::ForeignRuntime {
                expected_runtime_instance_id: self.runtime_instance_id(),
                actual_runtime_instance_id,
            });
        }
        let retired = self
            .history
            .canonical_checkpoint_gate()
            .with_settled_publication_snapshot(|| {
                let publication = committed.envelope();
                self.indexes.retire_after_publication(
                    &self.history,
                    publication,
                    self.config.storage.retention.reclaim_batch_size,
                )
            });
        Ok(retired.map(|generations| {
            let count = generations.len();
            drop(generations);
            count
        }))
    }
}
