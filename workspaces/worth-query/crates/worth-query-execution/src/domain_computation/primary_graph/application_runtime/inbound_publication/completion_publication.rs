//! Internal completion uses World's actual Home-open boundary before Native preparation.
use crate::domain_computation::execution_runtime::product_world::{
    publication_denial::{
        WorthQueryProductPublicationDenial as RequestDenial,
        WorthQueryProductPublicationOutcome as Outcome,
    },
    WorthQueryPreparedProductPublication, WorthQueryProductPublicationBinding,
    WorthQueryProductPublicationRequestControl,
};
use crate::domain_computation::primary_graph::provider::{
    WorthQueryInboundCompletionPreparationDenial, WorthQueryPrimaryGraphProvider,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_relational::facade::mvcc::{RelationalTransactionIntent, ValidatedRelationalProposal};
use worth_runtime_world::facade::{
    NoEffectCompositePublication, ProductUnpublishedOwnerEffects, ProductUnpublishedRecoveryHandle,
    RuntimeWorldPublicationPort, WorldDurablePublicationOutcome, WorldOpenAttemptOutcome,
    WorldOpenAttemptRefusal, WorldPreparedPublicationWithoutSignal,
};

pub(super) enum CompletionPreparationDenial {
    Request(RequestDenial<NoEffectCompositePublication>),
    Native(WorthQueryInboundCompletionPreparationDenial),
    Opening(WorldOpenAttemptRefusal),
}

pub(super) enum PreparedCompletionPublication {
    Memory(WorthQueryPreparedProductPublication),
    Home {
        publication: RuntimeWorldPublicationPort<(), (), (), (), ()>,
        prepared: WorldPreparedPublicationWithoutSignal,
        control: WorthQueryProductPublicationRequestControl,
        request: WorthQueryRequestScope,
    },
    Retained(
        ProductUnpublishedOwnerEffects,
        Option<std::sync::Arc<worth_runtime_world::facade::WorldNativePreparationDenial>>,
    ),
}

impl PreparedCompletionPublication {
    pub(super) fn prepare(
        provider: &WorthQueryPrimaryGraphProvider,
        binding: &WorthQueryProductPublicationBinding,
        proposal: ValidatedRelationalProposal,
        request: &WorthQueryRequestScope,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, CompletionPreparationDenial> {
        use CompletionPreparationDenial as Denial;
        let home = binding
            .publication()
            .query_publication_home_binding(binding.observation())
            .map_err(Denial::Opening)?;
        if home.is_none() {
            let candidate = provider
                .prepare_completion_proposal(execution, proposal)
                .map_err(Denial::Native)?;
            return binding
                .prepare_relational_candidate(candidate, request, false)
                .map(Self::Memory)
                .map_err(Denial::Request);
        }
        let control = binding
            .request_control(request)
            .ok_or(Denial::Request(RequestDenial::ClockMismatch))?;
        let pending = binding
            .publication()
            .prepare_query_publication_without_signal(
                execution,
                binding.observation().clone(),
                binding.admitted_program_source().map_err(Denial::Opening)?,
                RelationalTransactionIntent::ordinary(),
                control.cancellation(),
                control.deadline(),
            )
            .map_err(Denial::Opening)?;
        let opened = match advance_original_open(pending, binding, request, execution)
            .map_err(Denial::Opening)?
        {
            OpenedCompletion::Opened(opened) => opened,
            OpenedCompletion::Retained(effects) => return Ok(Self::Retained(effects, None)),
        };
        let prepared =
            match provider.prepare_opened_completion_proposal(execution, opened, proposal) {
                Ok(prepared) => prepared,
                Err(refused) => {
                    let (open, cause) = refused.into_parts();
                    return Ok(Self::Retained(
                        open.retain(),
                        Some(std::sync::Arc::new(cause)),
                    ));
                }
            };
        Ok(Self::Home {
            publication: binding.publication().clone(),
            prepared,
            control,
            request: request.clone(),
        })
    }

    pub(super) fn preparation_denial(
        &self,
    ) -> Option<std::sync::Arc<worth_runtime_world::facade::WorldNativePreparationDenial>> {
        match self {
            Self::Retained(_, cause) => cause.clone(),
            Self::Memory(_) | Self::Home { .. } => None,
        }
    }

    pub(super) fn unpublished_recovery_handle(&self) -> ProductUnpublishedRecoveryHandle {
        match self {
            Self::Memory(prepared) => prepared.unpublished_recovery_handle(),
            Self::Home { prepared, .. } => prepared.unpublished_recovery_handle(),
            Self::Retained(effects, _) => effects.recovery_handle(),
        }
    }

    pub(super) fn execute(
        self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Outcome, RequestDenial<NoEffectCompositePublication>> {
        match self {
            Self::Memory(prepared) => prepared.execute(execution),
            Self::Retained(effects, _) => Ok(Outcome::ProductUnpublished(effects)),
            Self::Home {
                publication,
                prepared,
                control,
                request,
            } => Ok(execute_home_publication(
                publication,
                prepared,
                &control,
                &request,
                execution,
            )),
        }
    }
}

pub(super) fn opening_request_stop(
    cause: WorldOpenAttemptRefusal,
) -> Option<worth_execution::WorkCeilingDenial> {
    match cause {
        WorldOpenAttemptRefusal::Reservation(no_effect) => {
            crate::domain_computation::primary_graph::publication_request_denial::publication(
                no_effect.cause(),
            )
            .err()
        }
        WorldOpenAttemptRefusal::Durable(cause) => {
            use worth_runtime_world::facade::WorldDurableMediumDenial as Medium;
            match cause {
                Medium::Request(stop) => Some(stop),
                Medium::NameAllocation
                | Medium::OccurrenceReopenRequired
                | Medium::Allocation(_)
                | Medium::Backing(_)
                | Medium::Ordered(_)
                | Medium::Frame(_)
                | Medium::NativeDescription(_)
                | Medium::Unavailable
                | Medium::PublicationBusy
                | Medium::PublicationUnresolved
                | Medium::StaleCheckpoint
                | Medium::CheckpointSelectionRequired
                | Medium::CheckpointOriginRetention
                | Medium::Storage(_) => None,
            }
        }
        WorldOpenAttemptRefusal::TransitionAdmission(_)
        | WorldOpenAttemptRefusal::NotHomeBound
        | WorldOpenAttemptRefusal::ProgramSourceOwnerRequired
        | WorldOpenAttemptRefusal::ProgramSource(_)
        | WorldOpenAttemptRefusal::IdentityScope => None,
    }
}

// A Pending observation is one retained physical step, not a new attempt.
enum OpenedCompletion {
    Opened(worth_runtime_world::facade::WorldOpenedPublicationAttempt),
    Retained(ProductUnpublishedOwnerEffects),
}

fn advance_original_open(
    mut pending: worth_runtime_world::facade::WorldPendingOpenAttempt,
    binding: &WorthQueryProductPublicationBinding,
    request: &WorthQueryRequestScope,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<OpenedCompletion, WorldOpenAttemptRefusal> {
    let handle = pending.unpublished_recovery_handle();
    loop {
        match pending.advance(execution) {
            WorldOpenAttemptOutcome::Opened(opened) => return Ok(OpenedCompletion::Opened(opened)),
            WorldOpenAttemptOutcome::Pending(next) => {
                if request.interruption().is_some() || execution.consult().is_err() {
                    return Ok(OpenedCompletion::Retained(next.retain()));
                }
                pending = next;
                std::thread::yield_now();
            }
            WorldOpenAttemptOutcome::Unresolved(pending)
            | WorldOpenAttemptOutcome::EvidenceRequired(pending) => {
                return Ok(OpenedCompletion::Retained(pending.retain()))
            }
            WorldOpenAttemptOutcome::Refused(cause) => {
                // World retains the original independently of this diagnostic read.
                return match binding.recovery().inspect_effects(&handle) {
                    Ok(effects) => Ok(OpenedCompletion::Retained(effects)),
                    Err(_) => Err(cause),
                };
            }
        }
    }
}

fn execute_home_publication(
    publication: RuntimeWorldPublicationPort<(), (), (), (), ()>,
    prepared: WorldPreparedPublicationWithoutSignal,
    control: &WorthQueryProductPublicationRequestControl,
    request: &WorthQueryRequestScope,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Outcome {
    let mut outcome =
        publication.execute_durable_without_signal(execution, prepared, control.cancellation());
    loop {
        match outcome {
            WorldDurablePublicationOutcome::Performed(performed) => {
                return Outcome::Performed(performed)
            }
            WorldDurablePublicationOutcome::Pending(pending) => {
                if request.interruption().is_some() || execution.consult().is_err() {
                    return Outcome::ProductUnpublished(pending.retain());
                }
                outcome = pending.advance(&execution);
                std::thread::yield_now();
            }
            WorldDurablePublicationOutcome::ProductUnpublished(effects)
            | WorldDurablePublicationOutcome::RecoveryRequired {
                custody: effects, ..
            } => return Outcome::ProductUnpublished(effects),
        }
    }
}
