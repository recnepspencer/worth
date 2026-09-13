use super::{ProductUnpublishedOwnerEffectsRecord, RelationalRecoveryRecordState};

/// Exclusive borrowed custody while a retained record crosses owner settlement.
/// Unwind restores the route before the catalog update can reinstall the record.
pub(crate) struct RelationalRecoveryLease<'a> {
    record: &'a mut ProductUnpublishedOwnerEffectsRecord,
    state: Option<RelationalRecoveryRecordState>,
}

impl<'a> RelationalRecoveryLease<'a> {
    pub(super) fn new(
        record: &'a mut ProductUnpublishedOwnerEffectsRecord,
        state: RelationalRecoveryRecordState,
    ) -> Self {
        Self {
            record,
            state: Some(state),
        }
    }

    pub(crate) fn finish(
        mut self,
        install: impl FnOnce(&mut ProductUnpublishedOwnerEffectsRecord, RelationalRecoveryRecordState),
    ) {
        let state = self
            .state
            .take()
            .expect("one settlement completion owns the route");
        install(self.record, state);
    }
}

impl std::ops::Deref for RelationalRecoveryLease<'_> {
    type Target = RelationalRecoveryRecordState;
    fn deref(&self) -> &Self::Target {
        self.state.as_ref().expect("live settlement custody")
    }
}

impl std::ops::DerefMut for RelationalRecoveryLease<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state.as_mut().expect("live settlement custody")
    }
}

impl Drop for RelationalRecoveryLease<'_> {
    fn drop(&mut self) {
        if let Some(mut state) = self.state.take() {
            // Consuming an owner witness cannot be undone. Its carried identity
            // and exact basis still name the owner's pending settlement record.
            if state.route.is_none() {
                state.restore_identity_repair();
            }
            self.record.restore_relational_recovery(state);
        }
    }
}
