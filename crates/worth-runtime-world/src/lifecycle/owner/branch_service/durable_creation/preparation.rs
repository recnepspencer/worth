//! Reserve exact original identities, owner targets and response backing before ACK.
use super::*;
use crate::branch::{RelationalBranchCreationPlan as Native, SignalBranchCreationPlan as Signal};
use crate::durability::*;
use crate::publication::durable_signal::WorldDurableSignalBacking;
use worth_foundational::facade::*;
impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn prepare_durable_creation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: RuntimeWorldHomeBranchCreationRequest,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Result<ActiveAttemptCustody, RuntimeWorldBranchAdmissionDenial> {
        use RuntimeWorldBranchAdmissionDenial as Denial;
        let binding = self
            .state
            .durable
            .as_ref()
            .ok_or(Denial::MemoryCreationRequestRequired)?
            .clone();
        let RuntimeWorldHomeBranchCreationRequest {
            source,
            program_source,
            intent,
            completion,
        } = request;
        let revision = program_source.program_revision();
        let commitment =
            crate::publication::world_branch_program_source_commitment(&source, revision)
                .map_err(|_| Denial::ProgramSourceMismatch)?;
        binding
            .binding()
            .branch_program_source_authority()
            .ok_or(Denial::ProgramSourceRequired)?
            .admit(&program_source, &commitment)
            .map_err(|_| Denial::ForeignProgramSource)?;
        let name = intent.name().clone();
        let mut attempt = RuntimeWorldPreparationService::prepare_creation(
            self,
            source.clone(),
            intent,
            cancellation,
            None,
        )?;
        let mut reservation = self
            .state
            .branches
            .reserve_branch(self.owner_identity(), name.clone())
            .map_err(map_registry_denial)?;
        let (branch, incarnation) = self
            .issue_branch_identities(name)
            .map_err(|()| Denial::IdentityExhausted)?;
        let witness = reservation
            .bind_creation_destination(branch.clone(), incarnation)
            .map_err(map_registry_denial)?;
        attempt.bind_destination(witness.clone());
        let owner = self
            .owner_identity()
            .durable()
            .ok_or(Denial::ForeignOwner)?;
        let association = CompositeAttemptAssociation::from_recorded(
            DurableCompositePublicationAttemptIdentity::from_recorded(
                owner,
                attempt.identity().ordinal(),
            ),
            DurableProductBranchIdentity::from_recorded(
                owner,
                source.branch_identity().name().as_str().to_owned(),
            )
            .map_err(|_| Denial::ForeignOwner)?,
            DurableProductBranchIncarnation::from_recorded(
                owner,
                source.lifecycle_incarnation().ordinal(),
            ),
        )
        .map_err(|_| Denial::ForeignOwner)?;
        let native_port = self.state.relational_services().fork_port();
        let (native_source, native_target, relational) = match attempt.plan().relational() {
            Native::ReuseExact => (None, None, WorldRelationalCreationInput::ReuseExact),
            Native::ForkExact { target } => {
                let observed = self.observe_creation_native_source(&source)?;
                let reserved = native_port
                    .reserve_fork_target(target.clone())
                    .map_err(|cause| super::super::native_fork_denial::admission_denial(&cause))?;
                let (canonical_before, transition_before) = native_port
                    .observe_original_frontiers(execution)
                    .map_err(|_| Denial::OwnerUnavailable)?;
                (
                    Some(observed),
                    Some(reserved),
                    WorldRelationalCreationInput::ForkExact {
                        target,
                        canonical_before,
                        transition_before,
                    },
                )
            }
        };
        let attachment = match &self.state.signal {
            crate::lifecycle::signal_services::RuntimeWorldSignalServices::Authored(actual) => {
                actual
            }
            crate::lifecycle::signal_services::RuntimeWorldSignalServices::Memory { .. } => {
                return Err(Denial::HomeCreationRequestRequired)
            }
        };
        let (signal_prepared, signal_backing) = match attempt.plan().signal() {
            Signal::ReuseExact => (None, None),
            Signal::ForkExact { target } => {
                let point = attachment
                    .recorded_current_basis(source.basis().signal_basis())
                    .map_err(|_| Denial::ForkSourceChanged)?;
                let actual = attachment
                    .source_port()
                    .readmit_descriptor(point.descriptor())
                    .map_err(|_| Denial::ForkSourceChanged)?;
                let prepared = attachment
                    .prepare_branch_fork(execution, &actual, target.clone(), association.clone())
                    .map_err(|_| Denial::OwnerUnavailable)?;
                (
                    Some(prepared),
                    Some(
                        WorldDurableSignalBacking::reserve(execution)
                            .map_err(|_| Denial::CapacityExhausted)?,
                    ),
                )
            }
        };
        let commit = if attempt.plan().relational().requires_owner_effect()
            || attempt.plan().signal().requires_owner_effect()
        {
            attempt.reserved_commit_identity().ordinal()
        } else {
            source.selected_commit().ordinal()
        };
        let subject = WorldDurableArtifact::from_input(
            WorldArtifactInput::CreationIntent(WorldCreationIntentInput {
                association: &association,
                custody: DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(
                    owner,
                    attempt.recovery_handle().identity().ordinal(),
                ),
                source: WorldHeadInput::from_observation(&source)
                    .map_err(|_| Denial::CapacityExhausted)?,
                destination: WorldHeadInput {
                    owner,
                    name: branch.name().as_str(),
                    incarnation: DurableProductBranchIncarnation::from_recorded(
                        owner,
                        incarnation.ordinal(),
                    ),
                    generation: DurableProductReferenceGeneration::from_recorded(0),
                    commit: DurableCompositeCommitIdentity::from_recorded(owner, commit),
                },
                program_revision: revision,
                relational,
                signal: attempt.plan().signal(),
            }),
            execution,
        )
        .map_err(|_| Denial::CapacityExhausted)?;
        let caller_bytes =
            encode_association(execution, &association).map_err(|_| Denial::CapacityExhausted)?;
        let native_caller =
            worth_relational::facade::mvcc::RelationalCallerAssociation::from_bytes(
                caller_bytes.as_ref(),
            )
            .map_err(|_| Denial::CapacityExhausted)?;
        let origin = PreparedCheckpointOccurrence::prepare(execution)
            .map_err(|_| Denial::CapacityExhausted)?;
        let funding = execution
            .reserve_retained_memory(std::mem::size_of::<WorldDurableCreationCustody>() as u64)
            .map_err(|_| Denial::CapacityExhausted)?;
        let cell = self
            .state
            .branches
            .branch_cell(source.branch_identity())
            .ok_or(Denial::RetiredBranch)?;
        let permit = cell
            .begin_durable_creation(&source, subject.clone(), execution)
            .map_err(|cause| match cause {
                crate::branch::WorldCreationPermitDenial::Medium(cause) => {
                    drop(cause);
                    Denial::CapacityExhausted
                }
                crate::branch::WorldCreationPermitDenial::Source(loss) => {
                    if loss.is_retired() {
                        Denial::RetiredBranch
                    } else if source
                        .mismatch_against_snapshot(loss.observed_head())
                        .is_some()
                    {
                        Denial::StaleSourceHead
                    } else {
                        Denial::OwnerUnavailable
                    }
                }
            })?;
        let native_custody = attempt.take_relational_custody();
        let signal_custody = attempt.take_signal_custody();
        let parts = attempt.into_parts();
        let mut custody = parts.custody;
        custody.install_durable_creation(Box::new(WorldDurableCreationCustody {
            preparation_denial: None,
            completion,
            phase: CreationPhase::Intent,
            binding,
            subject,
            permit: Some(permit),
            plan: parts.plan,
            source,
            branch,
            incarnation,
            reservation,
            witness,
            native_source,
            native_target,
            native_original: None,
            native_caller: Some(native_caller),
            native_custody,
            signal_custody,
            signal_prepared,
            signal_backing,
            signal_task: None,
            progress: crate::publication::CompositeAttemptProgress::untouched(),
            selected: None,
            release_subject: None,
            unsubmitted: None,
            released: None,
            no_effect: None,
            origin: Some(origin),
            _funding: funding,
        }));
        custody.begin_owner_execution();
        Ok(custody)
    }
}
