//! Durable Home publication for generated materialization suspend and restore.
use crate::domain_computation::execution_runtime::product_world::{
    publication_denial::{
        WorthQueryProductPublicationDenial as PublicationDenial,
        WorthQueryProductPublicationOutcome as PublicationOutcome,
    },
    WorthQueryProductPublicationBinding, WorthQueryProductPublicationRequestControl,
};
use worth_execution::ExecutionRequest;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductPublicationBindingInternalPort as _;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::{
    PreparedRelationalCommitCandidate, ValidatedRelationalProposal,
};
use worth_runtime_world::facade::{
    NoEffectCompositePublication, ProductUnpublishedOwnerEffects, RuntimeWorldPublicationPort,
    WorldDurablePublicationOutcome, WorldOpenAttemptOutcome, WorldOpenAttemptRefusal,
    WorldOpenedPublicationAttempt, WorldPendingOpenAttempt, WorldPreparedPublicationWithoutSignal,
};

pub(super) enum DurableOpenDenial {
    ClockMismatch,
    Stopped(worth_execution::WorkCeilingDenial),
    Refused(super::WorthQueryGeneratedOutputPublicationNoEffectCause),
}

pub(super) enum DurableMaterializationOpen {
    Memory,
    Opened {
        opened: WorldOpenedPublicationAttempt,
        control: WorthQueryProductPublicationRequestControl,
    },
    Retained(ProductUnpublishedOwnerEffects),
}

pub(super) fn open_if_home(
    binding: &WorthQueryProductPublicationBinding,
    request: &WorthQueryRequestScope,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<DurableMaterializationOpen, DurableOpenDenial> {
    let home = super::super::program_adoption::publication::preparation::query_home_binding(binding)
        .map_err(map_open_refusal)?;
    if home.is_none() {
        return Ok(DurableMaterializationOpen::Memory);
    }
    let control = binding
        .request_control(request)
        .ok_or(DurableOpenDenial::ClockMismatch)?;
    let pending =
        super::super::program_adoption::publication::preparation::prepare_ordinary_without_signal(
            binding, &control, execution,
        )
        .map_err(map_open_refusal)?;
    match advance_open(pending, binding, request, execution).map_err(map_open_refusal)? {
        OpenedAttempt::Opened(opened) => Ok(DurableMaterializationOpen::Opened { opened, control }),
        OpenedAttempt::Retained(effects) => Ok(DurableMaterializationOpen::Retained(effects)),
    }
}

pub(super) fn prepare_and_execute_opened<Schema: ApplicationSchema, Completion>(
    application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    binding: &WorthQueryProductPublicationBinding,
    opened: WorldOpenedPublicationAttempt,
    control: &WorthQueryProductPublicationRequestControl,
    request: &WorthQueryRequestScope,
    execution: ExecutionRequest<'_, '_>,
    proposal: ValidatedRelationalProposal,
    finish_completion: impl FnOnce(&PreparedRelationalCommitCandidate) -> Completion,
) -> Result<(PublicationOutcome, Completion), (PublicationOutcome, Option<Completion>)> {
    let (query_prep, bound) =
        match super::super::program_adoption::publication::preparation::bind_opened_proposal(
            opened, proposal,
        ) {
            Ok(pair) => pair,
            Err(refused) => {
                return Err((
                    PublicationOutcome::ProductUnpublished(refused.into_open().retain()),
                    None,
                ))
            }
        };
    let candidate = application
        .inner
        .runtime
        .primary_provider_ref()
        .graph_ref()
        .with_runtime(&execution, |runtime| {
            runtime.prepare_validated_proposal(&execution, bound)
        });
    let candidate = match candidate {
        Ok(candidate) => candidate,
        Err(_) => {
            return Err((
                PublicationOutcome::ProductUnpublished(query_prep.retain()),
                None,
            ))
        }
    };
    let completion = finish_completion(&candidate);
    let prepared =
        match super::super::program_adoption::publication::preparation::complete_opened_native(
            query_prep, candidate, &execution,
        ) {
            Ok(prepared) => prepared,
            Err(refused) => {
                return Err((
                    PublicationOutcome::ProductUnpublished(refused.into_open().retain()),
                    Some(completion),
                ))
            }
        };
    let outcome = execute_durable(
        binding.publication().clone(),
        prepared,
        control,
        request,
        execution,
    );
    Ok((outcome, completion))
}

enum OpenedAttempt {
    Opened(WorldOpenedPublicationAttempt),
    Retained(ProductUnpublishedOwnerEffects),
}

fn advance_open(
    mut pending: WorldPendingOpenAttempt,
    binding: &WorthQueryProductPublicationBinding,
    request: &WorthQueryRequestScope,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<OpenedAttempt, WorldOpenAttemptRefusal> {
    let handle = pending.unpublished_recovery_handle();
    loop {
        match pending.advance(execution) {
            WorldOpenAttemptOutcome::Opened(opened) => return Ok(OpenedAttempt::Opened(opened)),
            WorldOpenAttemptOutcome::Pending(next) => {
                if request.interruption().is_some() || execution.consult().is_err() {
                    return Ok(OpenedAttempt::Retained(next.retain()));
                }
                pending = next;
                std::thread::yield_now();
            }
            WorldOpenAttemptOutcome::Unresolved(pending)
            | WorldOpenAttemptOutcome::EvidenceRequired(pending) => {
                return Ok(OpenedAttempt::Retained(pending.retain()))
            }
            WorldOpenAttemptOutcome::Refused(cause) => {
                return match binding.recovery().inspect_effects(&handle) {
                    Ok(effects) => Ok(OpenedAttempt::Retained(effects)),
                    Err(_) => Err(cause),
                };
            }
        }
    }
}

fn execute_durable(
    publication: RuntimeWorldPublicationPort<(), (), (), (), ()>,
    prepared: WorldPreparedPublicationWithoutSignal,
    control: &WorthQueryProductPublicationRequestControl,
    request: &WorthQueryRequestScope,
    execution: ExecutionRequest<'_, '_>,
) -> PublicationOutcome {
    let mut outcome =
        publication.execute_durable_without_signal(execution, prepared, control.cancellation());
    loop {
        match outcome {
            WorldDurablePublicationOutcome::Performed(performed) => {
                crate::domain_computation::execution_runtime::product_world::commit_pulse::note_publication();
                return PublicationOutcome::Performed(performed);
            }
            WorldDurablePublicationOutcome::Pending(pending) => {
                if request.interruption().is_some() || execution.consult().is_err() {
                    return PublicationOutcome::ProductUnpublished(pending.retain());
                }
                outcome = pending.advance(&execution);
                std::thread::yield_now();
            }
            WorldDurablePublicationOutcome::ProductUnpublished(effects)
            | WorldDurablePublicationOutcome::RecoveryRequired {
                custody: effects, ..
            } => return PublicationOutcome::ProductUnpublished(effects),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn finish_suspension_outcome<Schema: ApplicationSchema>(
    application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: &crate::basis::WorthQueryProductBranchLease,
    outcome: Result<PublicationOutcome, PublicationDenial<NoEffectCompositePublication>>,
    branch: crate::basis::WorthQueryProductBranch,
    correspondence: std::sync::Arc<
        crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
    >,
    producer: super::ProducerQualification,
    completion: Option<worth_relational::facade::branch::RelationalMaterializationSuspensionCompletion>,
) -> Result<super::WorthQuerySuspendedGeneratedOutput, super::WorthQueryGeneratedOutputSuspensionFailure>
{
    match outcome {
        Ok(PublicationOutcome::Performed(performed)) => {
            let commit = performed
                .component_results()
                .relational_commit_result()
                .cloned()
                .expect("a relational-only publication returns its relational result");
            let mut publication = performed.consume();
            let observation = publication
                .take_successor_observation()
                .expect("the requested successor observation is retained");
            let mut next_binding = application.materialization_publication_binding(observation);
            let _ = next_binding.carry_selected_program_source_from(&product.publication_binding());
            Ok(super::suspended_output(
                next_binding,
                branch,
                correspondence,
                producer,
                completion.expect("performed suspension carries prepared Relational completion"),
                commit,
            ))
        }
        Err(denial) => Err(map_suspension_denial(denial)),
        Ok(PublicationOutcome::ProductUnpublished(effects)) => {
            let product = application.unpublished_materialization(effects, product);
            Err(
                super::WorthQueryGeneratedOutputSuspensionFailure::ProductUnpublished(
                    super::WorthQueryGeneratedOutputSuspensionRecovery::new(
                        product,
                        completion,
                        branch,
                        correspondence,
                        producer,
                    ),
                ),
            )
        }
    }
}

pub(super) fn map_suspension_denial(
    denial: PublicationDenial<NoEffectCompositePublication>,
) -> super::WorthQueryGeneratedOutputSuspensionFailure {
    use crate::domain_computation::primary_graph::application_contribution::advancement_denial;
    match denial {
        PublicationDenial::ClockMismatch => {
            super::WorthQueryGeneratedOutputSuspensionFailure::ExecutionDenied(
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(
                    crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::ForeignAdmissionClock,
                ),
            )
        }
        PublicationDenial::Stopped(stop) => {
            super::WorthQueryGeneratedOutputSuspensionFailure::ExecutionDenied(advancement_denial(
                stop,
            ))
        }
        PublicationDenial::Refused(_) => {
            super::WorthQueryGeneratedOutputSuspensionFailure::PublicationNoEffect
        }
    }
}

pub(super) fn map_open_suspension_denial(
    denial: DurableOpenDenial,
) -> super::WorthQueryGeneratedOutputSuspensionFailure {
    use crate::domain_computation::primary_graph::application_contribution::advancement_denial;
    match denial {
        DurableOpenDenial::ClockMismatch => {
            super::WorthQueryGeneratedOutputSuspensionFailure::ExecutionDenied(
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(
                    crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::ForeignAdmissionClock,
                ),
            )
        }
        DurableOpenDenial::Stopped(stop) => {
            super::WorthQueryGeneratedOutputSuspensionFailure::ExecutionDenied(advancement_denial(
                stop,
            ))
        }
        DurableOpenDenial::Refused(_) => {
            super::WorthQueryGeneratedOutputSuspensionFailure::PublicationNoEffect
        }
    }
}

fn map_open_refusal(cause: WorldOpenAttemptRefusal) -> DurableOpenDenial {
    match cause {
        WorldOpenAttemptRefusal::Reservation(no_effect) => {
            match crate::domain_computation::primary_graph::publication_request_denial::publication(
                no_effect.cause(),
            ) {
                Err(stop) => DurableOpenDenial::Stopped(stop),
                Ok(_) => DurableOpenDenial::Refused(super::restoration::map_world_cause(
                    no_effect.cause(),
                )),
            }
        }
        WorldOpenAttemptRefusal::Durable(
            worth_runtime_world::facade::WorldDurableMediumDenial::Request(stop),
        ) => DurableOpenDenial::Stopped(stop),
        _ => DurableOpenDenial::Refused(
            super::WorthQueryGeneratedOutputPublicationNoEffectCause::PreEffectFailure,
        ),
    }
}
