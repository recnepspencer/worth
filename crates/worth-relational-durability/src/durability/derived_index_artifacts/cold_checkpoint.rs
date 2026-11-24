//! Closeable deferred index custody authenticates the exact original member.
use crate::durability::data::RelationalCheckpointDirectory;
use contracts::storage::data::storage_unavailable::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause, RelationalStorageUnavailableCauseOwnerPort as _,
};
use std::sync::{Arc, Mutex};
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_indexes::internal_port::indexes::authority::checkpoint_admission::CheckpointIndexProjections;
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::IndexingState;
use worth_relational_storage::internal_port::storage::data::member_frame;
use worth_relational_storage::internal_port::storage::data::member_frame::MemberKind;
use worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure as Failure;

#[path = "cold_checkpoint/member_export.rs"]
pub mod member_export;
#[path = "cold_checkpoint/source_retirement.rs"]
pub mod source_retirement;

#[derive(Debug)]
pub struct DeferredIndexImage {
    history: HistorySubsystem,
    projections: CheckpointIndexProjections,
    schemas: crate::durability::checkpoints::region_directory::ReadmittedSchemas,
    admitted: Mutex<DeferredIndexState>,
}

#[derive(Debug)]
struct DeferredIndexState {
    data: Option<Result<Arc<IndexingState>, RelationalStorageUnavailable>>,
    directory: crate::durability::data::NativeCheckpointDirectoryEdge,
    physical: worth_relational_storage::internal_port::durability::medium::member_custody::native_source_slot::NativeSourceMemberState,
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
    pub fn readmit_into_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &mut IndexingState,
    ) -> Result<(), Failure> {
        let admitted = self.readmit(execution)?;
        snapshot.merge_checkpoint_image(&admitted);
        Ok(())
    }

    pub fn history(&self) -> &HistorySubsystem {
        &self.history
    }

    pub fn readmit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<IndexingState>, Failure> {
        contracts::execution::request_custody::check_request(execution)
            .map_err(Failure::Request)?;
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
                directory.manifest.index_definitions.first().map_or(
                    contracts::indexes::data::definition::DerivedIndexId(0),
                    |definition| definition.index_id,
                ),
            ),
            cause,
        };
        let result =
            contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(
                |lane| {
                    let key = directory.manifest.derived_indexes;
                    let bytes = directory
                        .source
                        .read_member(key, 1 << 30)
                        .map_err(refusal)?;
                    let payload = member_frame::decode(&bytes, key, MemberKind::DerivedIndex)
                        .map_err(refusal)?;
                    let (legacy, checkpoint, format): (
                worth_relational_branch::internal_port::indexes::data::DerivedIndexArtifacts,
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
                                contracts::durability::data::recovery_stop::RelationalRecoveryStop::Request(cause) => {
                                    Failure::Request(cause)
                                }
                                contracts::durability::data::recovery_stop::RelationalRecoveryStop::Content(cause) => {
                                    Failure::Content(cause)
                                }
                                contracts::durability::data::recovery_stop::RelationalRecoveryStop::Durability(
                                    error,
                                ) => error
                                    .storage_unavailable
                                    .map(Failure::StorageUnavailable)
                                    .unwrap_or_else(|| {
                                        Failure::StorageUnavailable(refusal(Cause::Damaged))
                                    }),
                                contracts::durability::data::recovery_stop::RelationalRecoveryStop::AdmissionImage(
                                    contracts::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal::OwnerUnavailable
                                    | contracts::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal::MissingArtifact
                                    | contracts::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal::MissingCaptureOrigin
                                ) => Failure::StorageUnavailable(refusal(Cause::SourceUnavailable)),
                                contracts::durability::data::recovery_stop::RelationalRecoveryStop::Allocation(cause) => match cause.classify() {
                                    worth_execution::ExecutionAllocationDenialClass::Request(cause) => Failure::Request(cause.into()),
                                    worth_execution::ExecutionAllocationDenialClass::Physical(_) => Failure::Request(
                                        contracts::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed),
                                },
                                _ => Failure::StorageUnavailable(refusal(Cause::Damaged)),
                            })?;
                    }
                    contracts::execution::request_custody::check_request(execution)
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

pub struct PreparedIndexImage {
    directory: Arc<RelationalCheckpointDirectory>,
    placement: worth_relational_storage::internal_port::durability::medium::member_custody::RelationalDurableMemberCustody,
    state: IndexingState,
}

impl PreparedIndexImage {
    pub fn prepare(
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
                subject: RelationalStorageSubject::Index(
                    contracts::indexes::data::definition::DerivedIndexId(0),
                ),
                cause: Cause::Damaged,
            })?;
        Ok(Self {
            directory,
            placement,
            state,
        })
    }

    pub fn admit(
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
                    physical: worth_relational_storage::internal_port::durability::medium::member_custody::native_source_slot::NativeSourceMemberState::new(
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

pub fn storage_subject(directory: &RelationalCheckpointDirectory) -> RelationalStorageSubject {
    RelationalStorageSubject::Index(directory.manifest.index_definitions.first().map_or(
        contracts::indexes::data::definition::DerivedIndexId(0),
        |d| d.index_id,
    ))
}
