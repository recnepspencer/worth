//! Prepare one task allocation before consuming an actual displaced directory alias.
use super::*;
use crate::durability::checkpoint_history::readmission::SelectedCheckpointReadRetirement;
use crate::durability::medium::{
    RelationalCheckpointAcknowledgementDenial as Ack, RelationalDurableMemberRetirement,
    RelationalDurableMemberRetirementObservation as O, RetiredRelationalDurableMemberCustody,
};
use std::{alloc::Layout, mem::MaybeUninit};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
pub(crate) struct PreparedCheckpointDirectoryRetirement {
    storage: Box<MaybeUninit<CheckpointDirectoryRetirement>>,
    memory: Option<ExecutionMemoryReservation>,
}
pub(crate) struct CheckpointDirectoryRetirement {
    manifest: RelationalCheckpointManifest,
    source: Arc<dyn RelationalDurableColdSource>,
    history_members: Arc<super::super::RelationalHistoryMemberDirectory>,
    member: Option<RelationalDurableMemberRetirement>,
    read: SelectedCheckpointReadRetirement,
    retired_member: Option<RetiredRelationalDurableMemberCustody>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl PreparedCheckpointDirectoryRetirement {
    pub(crate) fn prepare(request: &ExecutionRequest<'_, '_>) -> Result<Self, Ack> {
        let bytes = Layout::new::<CheckpointDirectoryRetirement>().size();
        let memory = request
            .reserve_retained_memory(bytes as u64)
            .map_err(|cause| {
                Ack::Recovery(super::super::RelationalRecoveryStop::Request(cause.into()))
            })?;
        Ok(Self {
            storage: Box::new_uninit(),
            memory,
        })
    }
    pub(crate) fn fill(
        self,
        directory: RelationalCheckpointDirectory,
    ) -> Box<CheckpointDirectoryRetirement> {
        let RelationalCheckpointDirectory {
            manifest,
            source,
            history_members,
            canonical_read,
            _basis,
        } = directory;
        let task = CheckpointDirectoryRetirement {
            manifest,
            source,
            history_members,
            member: Some(_basis.into_retirement()),
            read: canonical_read.into_retirement(),
            retired_member: None,
            _memory: self.memory,
        };
        Box::write(self.storage, task)
    }
}
impl CheckpointDirectoryRetirement {
    /// No allocation, arbitrary Drop or source replacement occurs in this step.
    pub(crate) fn advance(&mut self, request: &ExecutionRequest<'_, '_>) -> Result<bool, Ack> {
        if let Some(member) = &mut self.member {
            match member.advance(request, self.source.as_ref()) {
                O::Complete => {
                    self.retired_member = Some(
                        self.member
                            .take()
                            .expect("completed owner retirement is present")
                            .into_retired()
                            .ok()
                            .expect("completed owner retirement retains inert data"),
                    );
                }
                observation => return progress(observation),
            }
        }
        progress(self.read.advance(request, self.source.as_ref()))
    }
}
fn progress(observation: O) -> Result<bool, Ack> {
    match observation {
        O::Complete => Ok(true),
        O::Advanced => Ok(false),
        O::Busy => Err(Ack::Busy),
        O::Stopped(cause) => Err(Ack::Recovery(
            super::super::RelationalRecoveryStop::Request(cause.into()),
        )),
        O::Refused(cause) => Err(Ack::Retirement(cause)),
    }
}
impl std::fmt::Debug for PreparedCheckpointDirectoryRetirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PreparedCheckpointDirectoryRetirement")
    }
}
impl std::fmt::Debug for CheckpointDirectoryRetirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckpointDirectoryRetirement")
            .field(
                "canonical_members",
                &self.manifest.canonical_members.entries().len(),
            )
            .field("history", &Arc::as_ptr(&self.history_members))
            .field("member_active", &self.member.is_some())
            .finish_non_exhaustive()
    }
}
