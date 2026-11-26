//! Partial live admission stays inside the original pending ACK owner.
use worth_relational_contracts::internal_port as contracts;
use super::{
    PreparedCheckpointDirectory, PreparedCheckpointDirectoryRetirement,
    RelationalCheckpointDirectory,
};
use crate::durability::checkpoint_history::readmission::SelectedCheckpointRead;
use worth_relational_storage::internal_port::durability::medium::checkpoint_acknowledgement_denial::RelationalCheckpointAcknowledgementDenial as Ack;
use worth_relational_storage::internal_port::durability::medium::member_custody::{RelationalDurableMemberCustody, RelationalDurableMemberCustodyOwnerPort as _};
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
use worth_execution::{ExecutionArray, ExecutionRequest};
#[derive(Debug)]
pub(crate) struct CheckpointDirectoryAdmission {
    draft: Option<PreparedCheckpointDirectory>,
    members: Option<ExecutionArray<RelationalDurableMemberLocator>>,
    read: Option<SelectedCheckpointRead>,
    basis: Option<RelationalDurableMemberCustody>,
    retirement: Option<PreparedCheckpointDirectoryRetirement>,
}
impl CheckpointDirectoryAdmission {
    pub(crate) fn new(draft: PreparedCheckpointDirectory) -> Self {
        Self {
            draft: Some(draft),
            members: None,
            read: None,
            basis: None,
            retirement: None,
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
        if self.retirement.is_none() {
            self.retirement = Some(PreparedCheckpointDirectoryRetirement::prepare(request)?);
        }
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
        // G7: the commit index opens read-free over the admitted roster.
        let commit_index = std::sync::Arc::new(
            worth_relational_branch::internal_port::history::commit::paged_index::reader::ColdCommitIndex::open(
                std::sync::Arc::new(draft.manifest.commits.clone()),
                draft.source.clone(),
            ),
        );
        Ok(RelationalCheckpointDirectory {
            admission_owner: Default::default(),
            admission_image: None,
            source_schemas: Default::default(),
            manifest: draft.manifest,
            source: draft.source,
            history_members: draft.history_members,
            canonical_read: self.read.take().expect("admitted original selected read"),
            commit_index,
            _basis: self.basis.take().expect("admitted original member custody"),
            retirement: self
                .retirement
                .take()
                .expect("directory retirement storage prepaid before custody"),
        })
    }
}
pub(super) fn opening_stop(
    cause: Ack,
) -> contracts::durability::data::recovery_stop::RelationalRecoveryStop {
    use super::super::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
    match cause {
        Ack::Recovery(cause) => cause,
        Ack::OwnedRead(cause) => {
            worth_relational_storage::internal_port::durability::medium::owned_read_stop::owned_stop(
                cause,
                contracts::storage::data::storage_unavailable::RelationalStorageSubject::Envelope(
                    contracts::publication::patch::data::patch_position::PatchStreamPosition(0),
                ),
            )
        }
        Ack::Stopped(cause) => RelationalRecoveryStop::Request(cause.into()),
        _ => DurabilityError::new(
            RecoveryFailureClass::ColdStorageUnavailable,
            "selected directory admission unavailable",
        )
        .into(),
    }
}
