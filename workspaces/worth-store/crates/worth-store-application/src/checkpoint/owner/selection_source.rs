//! Native checkpoint selection source backed by the Home's single owner slot.
use super::{CheckpointSlot, HomeCheckpointOwner};
use std::sync::Weak;
use worth_relational::facade::durability::OpaqueCapturedCheckpointSubject;
use worth_store::physical_runtime::AnchoredRecordMembership;
use worth_store_relational::RelationalCheckpointSelectionSource;

pub(super) struct HomeCheckpointSelectionSource {
    pub(super) owner: Weak<HomeCheckpointOwner>,
}

impl std::fmt::Debug for HomeCheckpointSelectionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HomeCheckpointSelectionSource")
    }
}

impl RelationalCheckpointSelectionSource for HomeCheckpointSelectionSource {
    fn selected_version(
        &self,
        subject: &OpaqueCapturedCheckpointSubject,
    ) -> Result<
        Option<worth_store::physical_runtime::AnchoredRecordVersion>,
        worth_relational::facade::durability::RelationalDurableOwnedReadRefusal,
    > {
        use worth_relational::facade::{
            durability::RelationalDurableOwnedReadRefusal as Refusal,
            storage::RelationalStorageUnavailableCause,
        };
        let owner = self.owner.upgrade().ok_or(Refusal::Storage(
            RelationalStorageUnavailableCause::SourceUnavailable,
        ))?;
        let attempt = match &owner.state.try_lock().map_err(|_| Refusal::Busy)?.slot {
            CheckpointSlot::Checkpoint(attempt) => attempt.clone(),
            CheckpointSlot::Idle
            | CheckpointSlot::Discarding
            | CheckpointSlot::DiscardUnresolved { .. } => return Ok(None),
        };
        if !attempt
            .subject
            .as_ref()
            .is_some_and(|original| original.same_capture(subject))
        {
            return Err(Refusal::ForeignProtection);
        }
        let receipt = attempt.receipt.try_lock().map_err(|_| Refusal::Busy)?;
        Ok(receipt.as_ref().map(|receipt| receipt.version()))
    }

    fn selected_membership(
        &self,
        subject: &OpaqueCapturedCheckpointSubject,
    ) -> Option<AnchoredRecordMembership> {
        let owner = self.owner.upgrade()?;
        let attempt = owner.pending()?;
        if !attempt.subject.as_ref()?.same_capture(subject) {
            return None;
        }
        attempt
            .receipt
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }
}
