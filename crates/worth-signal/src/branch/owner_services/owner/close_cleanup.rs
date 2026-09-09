use super::super::lifecycle_state::SignalOwnerCloseCoordinator;
use super::{SignalOwner, OWNER_CLOSE_BATCH_SIZE};
use crate::branch::owner_services::operation_control::SignalOwnerOperationBoundary;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::branch::owner_services) enum SignalOwnerCloseBatchKind {
    Registry,
    Metadata,
}

impl<D, I, T> SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn finish_owner_close_cleanup(&self) {
        self.finish_owner_close_cleanup_with_observer(|_, _| {});
    }

    fn finish_owner_close_cleanup_with_observer(
        &self,
        mut observe_detached_batch: impl FnMut(SignalOwnerCloseBatchKind, usize),
    ) {
        self.conditional_retention.close();
        let Some(cleanup_claim) = self.lifecycle.claim_cleanup() else {
            return;
        };
        self.retention.close_owner();
        loop {
            let temporal_batch = self
                .conditional_temporal
                .take_close_batch(OWNER_CLOSE_BATCH_SIZE);
            if temporal_batch.is_empty() {
                break;
            }
            self.counters.record_close_batch();
            drop(temporal_batch);
        }
        loop {
            let registry_batch = self.registry.take_close_batch(OWNER_CLOSE_BATCH_SIZE);
            if !registry_batch.is_empty() {
                debug_assert!(registry_batch.cleaned_entries() <= OWNER_CLOSE_BATCH_SIZE);
                self.counters.record_close_batch();
                self.reach_operation_boundary(SignalOwnerOperationBoundary::OwnerCloseBatch);
                observe_detached_batch(
                    SignalOwnerCloseBatchKind::Registry,
                    registry_batch.cleaned_entries(),
                );
                drop(registry_batch);
                continue;
            }
            drop(registry_batch);
            let metadata_batch = self.metadata.take_close_batch(OWNER_CLOSE_BATCH_SIZE);
            if metadata_batch.is_empty() {
                drop(metadata_batch);
                break;
            }
            debug_assert!(metadata_batch.cleaned_entries() <= OWNER_CLOSE_BATCH_SIZE);
            self.counters.record_close_batch();
            self.reach_operation_boundary(SignalOwnerOperationBoundary::OwnerCloseBatch);
            observe_detached_batch(
                SignalOwnerCloseBatchKind::Metadata,
                metadata_batch.cleaned_entries(),
            );
            drop(metadata_batch);
        }
        cleanup_claim.complete();
    }

    #[cfg(test)]
    pub(in crate::branch::owner_services) fn close_with_cleanup_observer(
        &self,
        observe_detached_batch: impl FnMut(SignalOwnerCloseBatchKind, usize),
    ) -> Result<(), super::super::lifecycle_state::SignalOwnerCloseDenial> {
        self.lifecycle
            .begin_explicit_close(self.runtime_instance_id())?;
        self.retention.close_owner();
        self.finish_owner_close_cleanup_with_observer(observe_detached_batch);
        self.lifecycle.wait_until_closed();
        Ok(())
    }
}

impl<D, I, T> SignalOwnerCloseCoordinator for SignalOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn finish_owner_close(&self) {
        self.finish_owner_close_cleanup();
    }
}
