//! A retained index image is admitted explicitly, before an operation uses it.
mod captured_inputs;
mod member_export;
mod source_retirement;
use super::super::RuntimeOwnedState;
use super::{IndexDefinitionReadBinding, IndexingState, IndexingSubsystem};
use crate::durability::data::RelationalCheckpointDirectory;
use crate::durability::log::native_checkpoint_codec::member_frame::{self, MemberKind};
use crate::runtime::HistorySubsystem;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub(super) struct DeferredIndexImage {
    history: HistorySubsystem,
    projections: crate::indexes::CheckpointIndexProjections,
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
    pub(super) fn capture_generations(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::durability::derived_index_artifacts::captured_decode::CapturedCheckpointGenerations,
        crate::durability::data::RelationalRecoveryStop,
    > {
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
            self.capture_generations_in_reconstruction(lane, request)
        })
    }
    fn readmit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Arc<IndexingState>, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
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
                        self.projections
                            .verify_with_images(
                                lane,
                                execution,
                                &state,
                                checkpoint.bases(),
                                directory.clone(),
                                &self.history,
                                self.schemas.clone(),
                            )
                            .map_err(|stop| match stop {
                                crate::durability::data::RelationalRecoveryStop::Request(cause) => {
                                    Failure::Request(cause)
                                }
                                crate::durability::data::RelationalRecoveryStop::Content(cause) => {
                                    Failure::Content(cause)
                                }
                                crate::durability::data::RelationalRecoveryStop::Durability(
                                    error,
                                ) => error
                                    .storage_unavailable
                                    .map(Failure::StorageUnavailable)
                                    .unwrap_or_else(|| {
                                        Failure::StorageUnavailable(refusal(Cause::Damaged))
                                    }),
                                crate::durability::data::RelationalRecoveryStop::AdmissionImage(
                                    crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal::OwnerUnavailable
                                    | crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal::MissingArtifact
                                    | crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal::MissingCaptureOrigin
                                ) => Failure::StorageUnavailable(refusal(Cause::SourceUnavailable)),
                                crate::durability::data::RelationalRecoveryStop::Allocation(cause) => match cause.classify() {
                                    worth_execution::ExecutionAllocationDenialClass::Request(cause) => Failure::Request(cause.into()),
                                    worth_execution::ExecutionAllocationDenialClass::Physical(_) => Failure::Request(
                                        crate::execution::RelationalExecutionDenialCause::AllocationFailed),
                                },
                                _ => Failure::StorageUnavailable(refusal(Cause::Damaged)),
                            })?;
                    }
                    crate::execution::check_request(execution).map_err(Failure::Request)?;
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

impl IndexingSubsystem {
    pub(crate) fn install_cold_checkpoint(
        &self,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        directory: Arc<RelationalCheckpointDirectory>,
        runtime: &crate::runtime::RelationalRuntime,
        schemas: crate::durability::checkpoints::region_directory::ReadmittedSchemas,
    ) -> Result<(), RelationalStorageUnavailable> {
        let placement = directory
            .source
            .retain_members(&[directory.manifest.derived_indexes])
            .map_err(|cause| RelationalStorageUnavailable {
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
        let history = runtime.history.detached_owner_snapshot().map_err(|_| {
            RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Index(
                    directory
                        .manifest
                        .index_definitions
                        .first()
                        .map_or(crate::indexes::data::DerivedIndexId(0), |definition| {
                            definition.index_id
                        }),
                ),
                cause: Cause::Damaged,
            }
        })?;
        state.cold_checkpoint = Some(Arc::new(DeferredIndexImage {
            admitted: Mutex::new(DeferredIndexState {
                physical: crate::durability::medium::NativeSourceMemberState::new(
                    directory.source.clone(),
                    placement,
                ),
                directory: crate::durability::data::NativeCheckpointDirectoryEdge::new(directory),
                data: None,
            }),
            history,
            projections: crate::indexes::checkpoint_index_projections(lane, runtime),
            schemas,
        }));
        self.install(state);
        Ok(())
    }
    pub(crate) fn carried_checkpoint_image(
        &self,
    ) -> Option<crate::durability::medium::RelationalDurableMemberLocator> {
        self.state
            .read()
            .cold_checkpoint
            .as_ref()
            .map(|image| image.directory().manifest.derived_indexes)
    }
}

impl IndexDefinitionReadBinding {
    pub(crate) fn demand_checkpoint_image(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Failure> {
        if self.state.read().catalog.definitions.is_empty() {
            return Ok(());
        }
        demand(&self.state, &self.generation_publication, execution)
    }
}

fn demand(
    owner: &RuntimeOwnedState<IndexingState>,
    gate: &Arc<super::GenerationPublicationGate>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Failure> {
    let Some(image) = owner.read().cold_checkpoint.clone() else {
        return Ok(());
    };
    let admitted = image.readmit(execution)?;
    // Physical readmission above never waits while excluding catalog writers.
    let _generations =
        gate.enter(|| crate::execution::check_request(execution).map_err(Failure::Request))?;
    let mut state = owner.write();
    if state
        .cold_checkpoint
        .as_ref()
        .is_some_and(|pending| Arc::ptr_eq(pending, &image))
    {
        for generation in admitted.generations.all() {
            state.restore_generation_payload(generation.as_ref().clone());
        }
        state.bases.merge(&admitted.bases);
        state.cold_checkpoint = None;
    }
    Ok(())
}

impl IndexingState {
    /// A retained payload is not a publication at its authored version.
    pub(super) fn restore_generation_payload(
        &mut self,
        generation: crate::indexes::data::DerivedIndexGeneration,
    ) {
        self.next_generation_id = self
            .next_generation_id
            .max(generation.generation_id.0.saturating_add(1));
        self.generations.insert(generation);
    }
    pub(crate) fn demand_checkpoint_image(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Failure> {
        let Some(image) = self.cold_checkpoint.clone() else {
            return Ok(());
        };
        let admitted = image.readmit(execution)?;
        for generation in admitted.generations.all() {
            self.restore_generation_payload(generation.as_ref().clone());
        }
        self.bases.merge(&admitted.bases);
        self.cold_checkpoint = None;
        Ok(())
    }
}
