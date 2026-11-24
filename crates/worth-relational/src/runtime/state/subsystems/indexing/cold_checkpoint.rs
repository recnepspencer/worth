//! Closeable deferred index custody authenticates the exact original member.
use crate::durability::data::RelationalCheckpointDirectory;
use crate::indexes::authority::CheckpointIndexProjections;
use crate::runtime::state::subsystems::history::HistorySubsystem;
use crate::runtime::state::subsystems::indexing::IndexingState;
use crate::storage::data::member_frame::{self, MemberKind};
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
use std::sync::{Arc, Mutex};

#[path = "cold_checkpoint/member_export.rs"]
mod member_export;
#[path = "cold_checkpoint/source_retirement.rs"]
mod source_retirement;

#[derive(Debug)]
pub(crate) struct DeferredIndexImage {
    history: HistorySubsystem,
    projections: CheckpointIndexProjections,
    schemas: crate::durability::checkpoints::region_directory::ReadmittedSchemas,
    admitted: Mutex<DeferredIndexState>,
}

#[derive(Debug)]
struct DeferredIndexState {
    data: Option<Result<Arc<IndexingState>, RelationalStorageUnavailable>>,
    directory: crate::durability::data::NativeCheckpointDirectoryEdge,
    physical: crate::durability::medium::NativeSourceMemberState,
}
impl std::ops::Deref for DeferredIndexState {
    type Target = Option<Result<Arc<IndexingState>, RelationalStorageUnavailable>>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl std::ops::DerefMut for DeferredIndexState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl DeferredIndexImage {
    pub(crate) fn readmit_into_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &mut IndexingState,
    ) -> Result<(), Failure> {
        let admitted = self.readmit(execution)?;
        snapshot.merge_checkpoint_image(&admitted);
        Ok(())
    }

    pub(crate) fn history(&self) -> &HistorySubsystem {
        &self.history
    }

    pub(crate) fn readmit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<IndexingState>, Failure> {
        crate::execution::request_custody::check_request(execution).map_err(Failure::Request)?;
        let directory = self.directory();
        let mut admitted = self
            .admitted
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(result) = &**admitted {
            return result.clone().map_err(Into::into);
        }
        let refusal = |cause| RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Index(
                directory
                    .manifest
                    .index_definitions
                    .first()
                    .map_or(crate::indexes::data::DerivedIndexId(0), |definition| {
                        definition.index_id
                    }),
            ),
            cause,
        };
        let result =
            crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(
                |lane| {
                    let key = directory.manifest.derived_indexes;
                    let bytes = directory
                        .source
                        .read_member(key, 1 << 30)
                        .map_err(refusal)?;
                    let payload = member_frame::decode(&bytes, key, MemberKind::DerivedIndex)
                        .map_err(refusal)?;
                    let (legacy, checkpoint, format): (
                crate::indexes::data::DerivedIndexArtifacts,
                Option<crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts>, u16,
            ) = rmp_serde::from_slice(payload).map_err(|_| refusal(Cause::Damaged))?;
                    if !crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts::supports_outer_format(format, checkpoint.is_some()) {
                return Err(refusal(Cause::Damaged).into());
            }
                    let mut state = IndexingState::default();
                    state.restore_generation_frontier(directory.manifest.index_generation_frontier);
                    state
                        .readmit_checkpoint_catalog(
                            &directory.manifest.index_definitions.ordinary_owned_copy(),
                            directory.manifest.index_definition_frontier,
                        )
                        .map_err(|_| refusal(Cause::Damaged))?;
                    crate::durability::derived_index_artifacts::restore_checkpoint_derived_index_artifacts(
                lane, &mut state, &self.history, &legacy, checkpoint.as_ref(),
                directory.manifest.index_generation_frontier, &mut Default::default(),
            ).map_err(|_| refusal(Cause::Damaged))?;
                    if let Some(checkpoint) = &checkpoint {
                        super::projection_verification::verify_with_images(
                                &self.projections,
                                lane,
                                execution,
                                &state,
                                checkpoint.bases(),
                                directory.clone(),
                                &self.history,
                                self.schemas.clone(),
                            )
                            .map_err(|stop| match stop {
                                crate::durability::data::recovery_stop::RelationalRecoveryStop::Request(cause) => {
                                    Failure::Request(cause)
                                }
                                crate::durability::data::recovery_stop::RelationalRecoveryStop::Content(cause) => {
                                    Failure::Content(cause)
                                }
                                crate::durability::data::recovery_stop::RelationalRecoveryStop::Durability(
                                    error,
                                ) => error
                                    .storage_unavailable
                                    .map(Failure::StorageUnavailable)
                                    .unwrap_or_else(|| {
                                        Failure::StorageUnavailable(refusal(Cause::Damaged))
                                    }),
                                crate::durability::data::recovery_stop::RelationalRecoveryStop::AdmissionImage(
                                    crate::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal::OwnerUnavailable
                                    | crate::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal::MissingArtifact
                                    | crate::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal::MissingCaptureOrigin
                                ) => Failure::StorageUnavailable(refusal(Cause::SourceUnavailable)),
                                crate::durability::data::recovery_stop::RelationalRecoveryStop::Allocation(cause) => match cause.classify() {
                                    worth_execution::ExecutionAllocationDenialClass::Request(cause) => Failure::Request(cause.into()),
                                    worth_execution::ExecutionAllocationDenialClass::Physical(_) => Failure::Request(
                                        crate::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed),
                                },
                                _ => Failure::StorageUnavailable(refusal(Cause::Damaged)),
                            })?;
                    }
                    crate::execution::request_custody::check_request(execution)
                        .map_err(Failure::Request)?;
                    Ok(Arc::new(state))
                },
            );
        match &result {
            Ok(state) => {
                **admitted = Some(Ok(state.clone()));
            }
            Err(Failure::StorageUnavailable(failure)) if failure.cause.is_proven_damage() => {
                **admitted = Some(Err(*failure));
            }
            Err(Failure::StorageUnavailable(_))
            | Err(Failure::Request(_) | Failure::Content(_)) => {}
        }
        result
    }
}

pub(crate) struct PreparedIndexImage {
    directory: Arc<RelationalCheckpointDirectory>,
    placement: crate::durability::medium::RelationalDurableMemberCustody,
    state: IndexingState,
}

impl PreparedIndexImage {
    pub(crate) fn prepare(
        directory: Arc<RelationalCheckpointDirectory>,
    ) -> Result<Self, RelationalStorageUnavailable> {
        let placement = directory
            .source
            .retain_members(&[directory.manifest.derived_indexes])
            .map_err(|cause| RelationalStorageUnavailable {
                subject: storage_subject(&directory),
                cause,
            })?;
        let mut state = IndexingState::default();
        state.restore_generation_frontier(directory.manifest.index_generation_frontier);
        state
            .readmit_checkpoint_catalog(
                &directory.manifest.index_definitions.ordinary_owned_copy(),
                directory.manifest.index_definition_frontier,
            )
            .map_err(|_| RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Index(crate::indexes::data::DerivedIndexId(0)),
                cause: Cause::Damaged,
            })?;
        Ok(Self {
            directory,
            placement,
            state,
        })
    }

    pub(crate) fn admit(
        self,
        history: HistorySubsystem,
        projections: CheckpointIndexProjections,
        schemas: crate::durability::checkpoints::region_directory::ReadmittedSchemas,
    ) -> (IndexingState, Arc<DeferredIndexImage>) {
        let Self {
            directory,
            placement,
            state,
        } = self;
        (
            state,
            Arc::new(DeferredIndexImage {
                admitted: Mutex::new(DeferredIndexState {
                    physical: crate::durability::medium::NativeSourceMemberState::new(
                        directory.source.clone(),
                        placement,
                    ),
                    directory: crate::durability::data::NativeCheckpointDirectoryEdge::new(
                        directory,
                    ),
                    data: None,
                }),
                history,
                projections,
                schemas,
            }),
        )
    }
}

pub(crate) fn storage_subject(
    directory: &RelationalCheckpointDirectory,
) -> RelationalStorageSubject {
    RelationalStorageSubject::Index(
        directory
            .manifest
            .index_definitions
            .first()
            .map_or(crate::indexes::data::DerivedIndexId(0), |d| d.index_id),
    )
}
