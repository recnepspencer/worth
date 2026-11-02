//! Partial live admission stays inside the original pending ACK owner.
use super::{PreparedCheckpointDirectory, RelationalCheckpointDirectory};
use crate::durability::{
    checkpoint_history::readmission::SelectedCheckpointRead,
    medium::{
        RelationalCheckpointAcknowledgementDenial as Ack, RelationalDurableMemberCustody,
        RelationalDurableMemberLocator,
    },
};
use worth_execution::{ExecutionArray, ExecutionRequest};
#[derive(Debug)]
pub(crate) struct CheckpointDirectoryAdmission {
    draft: Option<PreparedCheckpointDirectory>,
    members: Option<ExecutionArray<RelationalDurableMemberLocator>>,
    read: Option<SelectedCheckpointRead>,
    basis: Option<RelationalDurableMemberCustody>,
}
impl CheckpointDirectoryAdmission {
    pub(crate) fn new(draft: PreparedCheckpointDirectory) -> Self {
        Self {
            draft: Some(draft),
            members: None,
            read: None,
            basis: None,
        }
    }
    pub(crate) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalCheckpointDirectory, Ack> {
        let draft = self
            .draft
            .as_ref()
            .expect("incomplete admission retains its original draft");
        if self.members.is_none() {
            self.members = Some(
                draft
                    .manifest
                    .semantic_members(request)
                    .map_err(Ack::Recovery)?,
            );
        }
        if self.read.is_none() {
            self.read = Some(
                SelectedCheckpointRead::capture(draft.source.clone(), request)
                    .map_err(Ack::OwnedRead)?,
            );
        }
        if self.basis.is_none() {
            self.basis = Some(
                RelationalDurableMemberCustody::capture(
                    &draft.source,
                    request,
                    self.members
                        .as_ref()
                        .expect("admitted semantic roster")
                        .elements(),
                )
                .map_err(Ack::OwnedRead)?,
            );
        }
        // Nothing fallible follows either completed handoff. Partial owners above
        // remain in this exact state on every refusal, including Busy/Stopped.
        let draft = self
            .draft
            .take()
            .expect("admission owns the same original draft");
        Ok(RelationalCheckpointDirectory {
            source_schemas: Default::default(),
            manifest: draft.manifest,
            source: draft.source,
            history_members: draft.history_members,
            canonical_read: self.read.take().expect("admitted original selected read"),
            _basis: self.basis.take().expect("admitted original member custody"),
        })
    }
}
pub(super) fn opening_stop(cause: Ack) -> super::super::RelationalRecoveryStop {
    use super::super::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
    match cause {
        Ack::Recovery(cause) => cause,
        Ack::OwnedRead(cause) => crate::durability::checkpoint_history::capture::owned_stop(
            cause,
            crate::storage::data::RelationalStorageSubject::Envelope(
                crate::publication::patch::data::PatchStreamPosition(0),
            ),
        ),
        Ack::Stopped(cause) => RelationalRecoveryStop::Request(cause.into()),
        _ => DurabilityError::new(
            RecoveryFailureClass::ColdStorageUnavailable,
            "selected directory admission unavailable",
        )
        .into(),
    }
}
