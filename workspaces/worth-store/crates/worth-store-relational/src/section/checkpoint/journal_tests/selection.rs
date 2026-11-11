//! Same original pending capture and actual physical receipt for this fixture.
use std::sync::Mutex;
use worth_relational::facade::durability::{
    OpaqueCapturedCheckpointSubject, RelationalDurableOwnedReadRefusal as Refusal,
};
use worth_store::physical_runtime::{AnchoredRecordMembership, AnchoredRecordVersion};

#[derive(Debug, Default)]
pub(super) struct Selected {
    pub(super) subject: Mutex<Option<OpaqueCapturedCheckpointSubject>>,
    pub(super) receipt: Mutex<Option<AnchoredRecordMembership>>,
}
impl crate::RelationalCheckpointSelectionSource for Selected {
    fn selected_version(
        &self,
        subject: &OpaqueCapturedCheckpointSubject,
    ) -> Result<Option<AnchoredRecordVersion>, Refusal> {
        let original = self.subject.try_lock().map_err(|_| Refusal::Busy)?;
        let Some(original) = original.as_ref() else {
            return Ok(None);
        };
        if !original.same_capture(subject) {
            return Err(Refusal::ForeignSource);
        }
        Ok(self
            .receipt
            .try_lock()
            .map_err(|_| Refusal::Busy)?
            .as_ref()
            .map(AnchoredRecordMembership::version))
    }
    fn selected_membership(
        &self,
        subject: &OpaqueCapturedCheckpointSubject,
    ) -> Option<AnchoredRecordMembership> {
        if !self.subject.lock().unwrap().as_ref()?.same_capture(subject) {
            return None;
        }
        self.receipt.lock().unwrap().clone()
    }
}
