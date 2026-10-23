use std::sync::Arc;

use super::{
    ConditionalDefinitionAttemptCustody, PlannedProductReferenceSuccessor,
    PreparedCompositePublicationWithSignal,
};
use crate::recovery::{
    ProductUnpublishedRecoveryHandle, RecoveryCatalog, ReservedProductUnpublishedRecordUpdate,
};

/// Reserved continuation of one original unpublished conditional definition.
/// Dropping preparation restores the same catalog custody. Execution transfers
/// that custody only to the exact successor whose reservations this value owns.
#[derive(Debug)]
#[must_use = "conditional recovery must be executed or returned to its original catalog record"]
pub struct PreparedConditionalDefinitionRecovery {
    pub(crate) prepared: Option<PreparedCompositePublicationWithSignal>,
    pub(crate) custody: Arc<ConditionalDefinitionAttemptCustody>,
    pub(crate) update: Option<ReservedProductUnpublishedRecordUpdate>,
    pub(crate) catalog: RecoveryCatalog,
    pub(crate) successor: ProductUnpublishedRecoveryHandle,
    pub(crate) completed: bool,
}

impl PreparedConditionalDefinitionRecovery {
    /// The reserved World successor. Downstream owners use this identity to
    /// reserve their delivery and receipt custody before executing recovery.
    pub fn planned_successor(&self) -> &PlannedProductReferenceSuccessor {
        self.prepared
            .as_ref()
            .expect("unexecuted recovery owns preparation")
            .planned_successor()
    }

    /// Identity of this continuation's reserved unpublished record. This is a
    /// correlation handle; it cannot execute or publish the prepared successor.
    pub fn unpublished_recovery_handle(&self) -> ProductUnpublishedRecoveryHandle {
        self.successor.clone()
    }

    pub(crate) fn complete(&mut self) {
        self.completed = true;
        self.update.take().expect("original catalog guard").finish();
    }
}

impl Drop for PreparedConditionalDefinitionRecovery {
    fn drop(&mut self) {
        // Release an unexecuted successor before deciding where the exact
        // candidate remains. During unwind an executed attempt has already
        // exposed its actual effects through its ordinary active record Drop.
        drop(self.prepared.take());
        let Some(mut update) = self.update.take() else {
            return;
        };
        if !self.completed && self.catalog.lookup_record(&self.successor).is_none() {
            update
                .record_mut()
                .expect("exclusive original recovery record")
                .restore_conditional_definition(Arc::clone(&self.custody));
        }
        update.finish();
    }
}
