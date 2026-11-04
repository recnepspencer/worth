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
    storage: Box<MaybeUninit<DirectoryRetirementPayload>>,
    memory: Option<ExecutionMemoryReservation>,
}
pub(crate) struct CheckpointDirectoryRetirement {
    payload: Box<DirectoryRetirementPayload>,
}
struct DirectoryRetirementPayload {
    _source_schemas: std::sync::OnceLock<
        Arc<
            crate::durability::source_initial_inputs::schema_catalog::ReadmittedSourceSchemaCatalog,
        >,
    >,
    manifest: RelationalCheckpointManifest,
    source: Option<Arc<dyn RelationalDurableColdSource>>,
    history_members: Arc<super::super::RelationalHistoryMemberDirectory>,
    member: Option<RelationalDurableMemberRetirement>,
    read: SelectedCheckpointReadRetirement,
    retired_member: Option<RetiredRelationalDurableMemberCustody>,
    // Keep the original funding inside its prepaid payload. The edge carries
    // one pointer, and every retained field drops before its reservation.
    _memory: Option<ExecutionMemoryReservation>,
}
impl PreparedCheckpointDirectoryRetirement {
    pub(crate) fn prepare(request: &ExecutionRequest<'_, '_>) -> Result<Self, Ack> {
        let bytes = Layout::new::<DirectoryRetirementPayload>().size();
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
}
impl RelationalCheckpointDirectory {
    pub(crate) fn into_retirement(self) -> CheckpointDirectoryRetirement {
        let Self {
            source_schemas,
            manifest,
            source,
            history_members,
            canonical_read,
            _basis,
            retirement,
        } = self;
        let payload = DirectoryRetirementPayload {
            _source_schemas: source_schemas,
            manifest,
            source: Some(source),
            history_members,
            member: Some(_basis.into_retirement()),
            read: canonical_read.into_retirement(),
            retired_member: None,
            _memory: retirement.memory,
        };
        CheckpointDirectoryRetirement {
            payload: Box::write(retirement.storage, payload),
        }
    }
}

impl CheckpointDirectoryRetirement {
    pub(crate) fn advance(&mut self, request: &ExecutionRequest<'_, '_>) -> Result<bool, Ack> {
        self.payload.advance(request)
    }
}
impl DirectoryRetirementPayload {
    /// No allocation, arbitrary Drop or source replacement occurs in this step.
    pub(crate) fn advance(&mut self, request: &ExecutionRequest<'_, '_>) -> Result<bool, Ack> {
        let Some(source) = &self.source else {
            return Ok(true);
        };
        if let Some(member) = &mut self.member {
            match member.advance(request, source.as_ref()) {
                O::Complete => {
                    self.retired_member = Some(
                        self.member
                            .take()
                            .expect("completed owner retirement is present")
                            .into_retired()
                            .unwrap_or_else(|_| {
                                unreachable!("completed owner retirement retains inert data")
                            }),
                    );
                }
                observation => return progress(observation),
            }
        }
        let complete = progress(self.read.advance(request, source.as_ref()))?;
        if complete {
            self.source.take();
        }
        Ok(complete)
    }
}
impl RelationalCheckpointDirectory {
    pub(crate) fn preflight_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Ack> {
        if !std::ptr::addr_eq(self.source.as_ref(), source) {
            return Err(Ack::Retirement(
                crate::durability::medium::RelationalDurableMemberRetirementRefusal::ForeignSource,
            ));
        }
        self._basis
            .preflight_retirement(source)
            .map_err(Ack::Retirement)?;
        self.canonical_read
            .preflight_retirement(source)
            .map_err(Ack::Retirement)
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
                &self.payload.manifest.canonical_members.entries().len(),
            )
            .field(
                "history_owners",
                &Arc::strong_count(&self.payload.history_members),
            )
            .field("member_active", &self.payload.member.is_some())
            .finish_non_exhaustive()
    }
}
