//! Fresh typed admission resumes the original managed publication, never its handler.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::IdempotencyResolutionOps;
use crate::domain_computation::primary_graph::product_operation::ProductOperationContextScopedOps;
use crate::domain_computation::primary_graph::provider::ProducerScopedOps;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationIdempotencyResolution as Resolution,
    WorthQueryManagedApplicationRecoveryDenial as RecoveryDenial,
    WorthQueryManagedApplicationRecoveryOutcome as RecoveryOutcome,
};
use worth_query_declaration::facade::application_operation::ApplicationEncodedInput;

pub(in crate::domain_computation::primary_graph) struct RecoveredProducerPublication {
    pub(in crate::domain_computation::primary_graph) receipt: WorthQueryApplicationCommitReceipt,
    pub(in crate::domain_computation::primary_graph) change: crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange,
    pub(in crate::domain_computation::primary_graph) observation: worth_runtime_world::facade::ProductBranchObservation,
    pub(in crate::domain_computation::primary_graph) retired_carrier: WorthQueryRecoveredProgramOutputSource,
    pub(in crate::domain_computation::primary_graph) original: Option<std::sync::Arc<crate::domain_computation::primary_graph::application_output_demand::OriginalProducingPublication>>,
}

impl ContinuationState {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution) fn progress<
        S,
        B,
    >(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        installed: &TypedInstalledProducer<S, B>,
        runtime: &dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<S>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        invocation: &ProducerInvocationAuthority<'_>,
        edition: super::super::super::InstalledProducerEdition,
    ) -> Result<Option<RecoveredProducerPublication>, WorthQueryOutputDemandDenial>
    where
        S: ApplicationSchema + 'static,
        B: WorthQueryApplicationProducerBinding<S>,
    {
        if self.edition != edition || &self.mode != invocation.mode() {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                B::IDENTITY,
            ));
        }
        self.close_prior(phase, runtime)
            .map_err(|_| retryable(WorthQueryOutputDemandDenialKind::ProducerUnavailable))?;
        let branch = match &self.state {
            PublicationState::Unpublished(recovery) => WorthQueryProductBranch::from_occurrence(
                recovery
                    .inspect()
                    .map_err(|stop| {
                        retryable(WorthQueryOutputDemandDenialKind::WorldRecovery(stop))
                    })?
                    .expected_product()
                    .lifecycle_incarnation(),
            ),
            PublicationState::Performed {
                carrier: Some(carrier),
                ..
            } => carrier.product_branch(),
            PublicationState::Performed { carrier: None, .. } => {
                return Err(denial(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    B::IDENTITY,
                ))
            }
            PublicationState::Closed | PublicationState::Closing => {
                return Err(denial(
                    WorthQueryOutputDemandDenialKind::Closed,
                    B::IDENTITY,
                ))
            }
        };
        let execution = phase
            .checked_execution_request_for(runtime.product_runtime())
            .map_err(|stop| retryable(WorthQueryOutputDemandDenialKind::ExecutionRequest(stop)))?;
        let product = runtime
            .product_runtime()
            .admit_product_occurrence(branch.occurrence())
            .map_err(|stop| {
                readmission_denial(WorthQueryOutputDemandDenial::product_selection(
                    stop,
                    B::IDENTITY,
                ))
            })?;
        let selected = runtime
            .on_product_for_owner_recovery(&execution, product)
            .map_err(|stop| {
                readmission_denial(WorthQueryOutputDemandDenial::product_selection(
                    stop,
                    B::IDENTITY,
                ))
            })?;
        require_selected_program(&selected, invocation.mode()).map_err(readmission_denial)?;
        let shared = selected.prepare_shared_query_basis().map_err(|(_, stop)| {
            readmission_denial(
                source_readmission::ready_resource_denial(B::IDENTITY, stop).into_output_denial(),
            )
        })?;
        let control = select_invocation_control(&execution, runtime, invocation)
            .map_err(|stop| stop.into_output_denial().with_recovery_posture(crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandRecoveryPosture::Retryable))?;
        let input = self
            .input
            .as_deref()
            .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::Closed, B::IDENTITY))?
            .downcast_ref::<ApplicationEncodedInput<
                <Operation<S, B> as ApplicationMutationBinding<S>>::InputBinding,
            >>()
            .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::ForeignDemand, B::IDENTITY))?;
        let prepared = selected_public::prepare_recovery_public_mutation::<S, B>(
            &execution,
            installed,
            runtime,
            &shared,
            control.as_ref(),
            principal,
            request,
            input.input(),
            self,
        )
        .map_err(|stop| {
            readmission_denial(selected_public::preparation_stop::<S, B>(stop).into_output_denial())
        })?;
        if let PublicationState::Unpublished(recovery) = &self.state {
            let (outcome, carrier) = runtime
                .recover_original_producer_publication(
                    phase,
                    recovery,
                    &prepared.operation,
                    self.idempotency,
                )
                .map_err(recovery_denial)?;
            match outcome {
                RecoveryOutcome::NoEffect => {}
                RecoveryOutcome::ProductUnpublished {
                    next,
                    prior_cleanup_failure,
                } => {
                    self.state = PublicationState::Unpublished(next);
                    self.prior_cleanup = prior_cleanup_failure;
                }
                RecoveryOutcome::Performed(outcome) => {
                    self.state = PublicationState::Performed { outcome, carrier };
                }
            }
            // World may have moved D. Reenter the new D before keyed receipt
            // inspection and delivery on a later invocation.
            return Ok(None);
        }
        let PublicationState::Performed { outcome, carrier } = &mut self.state else {
            unreachable!()
        };
        let binding = outcome.bind_original_definition(self.idempotency);
        let receipt = runtime
            .resolve_admitted_application_idempotency(phase, &prepared.operation, binding)
            .map_err(|stop| {
                denial(
                    WorthQueryOutputDemandDenialKind::PublicationReceipt(stop.kind()),
                    B::IDENTITY,
                )
            })?;
        let Resolution::AlreadyCommitted(receipt) = receipt.into_resolution() else {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                B::IDENTITY,
            ));
        };
        let original = crate::domain_computation::primary_graph::application_output_demand::OriginalProducingPublication::for_successor(
            &execution, self.original_publication.clone(), &receipt,
        )?;
        if let Some(failure) = outcome.take_original_cleanup_failure() {
            self.prior_cleanup = Some(failure);
            return Ok(None);
        }
        if let Some(recovery) = outcome.take_original_cleanup()
            && let Err(failure) = runtime
                .application_runtime()
                .close_product_publication_recovery_in_advancement(phase, recovery)
        {
            self.prior_cleanup = Some(failure);
            return Ok(None);
        }
        let actual = carrier.take().ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                B::IDENTITY,
            )
        })?;
        let (change, observation, retired_carrier) = match actual.into_producer_delivery(&receipt) {
            Ok(parts) => parts,
            Err(actual) => {
                *carrier = Some(actual);
                return Err(denial(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    B::IDENTITY,
                ));
            }
        };
        self.state = PublicationState::Closed;
        Ok(Some(RecoveredProducerPublication {
            receipt,
            change,
            observation,
            retired_carrier,
            original,
        }))
    }
}

fn retryable(kind: WorthQueryOutputDemandDenialKind) -> WorthQueryOutputDemandDenial {
    denial(kind, "producer publication recovery").with_recovery_posture(
        crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandRecoveryPosture::Retryable,
    )
}
fn recovery_denial(stop: RecoveryDenial) -> WorthQueryOutputDemandDenial {
    use worth_runtime_world::facade::RuntimeWorldSettledRelationalAdoptionDenial as Adoption;
    let kind = match stop {
        RecoveryDenial::Demand(stop) => return stop,
        RecoveryDenial::ExecutionDenied(stop) => {
            WorthQueryOutputDemandDenialKind::ExecutionRequest(stop)
        }
        RecoveryDenial::Authorization(stop) => {
            return retryable(WorthQueryOutputDemandDenialKind::of_request_authorization(
                stop.kind(),
            ));
        }
        RecoveryDenial::World(stop) => WorthQueryOutputDemandDenialKind::WorldRecovery(stop),
        RecoveryDenial::Adoption(stop) => match *stop {
            Adoption::Recovery(stop) => WorthQueryOutputDemandDenialKind::WorldRecovery(stop),
            Adoption::Publication(stop) => {
                WorthQueryOutputDemandDenialKind::RecoveryPublication(stop.cause())
            }
        },
        RecoveryDenial::ForeignAdmission | RecoveryDenial::BindingMismatch => {
            return denial(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                "producer publication recovery",
            )
        }
        RecoveryDenial::ProviderAdmission(stop) => {
            return readmission_denial(
                source_readmission::ready_resource_denial("producer publication recovery", stop)
                    .into_output_denial(),
            )
        }
        RecoveryDenial::ProviderCapacity => {
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
        }
        RecoveryDenial::ProviderBusy => WorthQueryOutputDemandDenialKind::SchedulingDeferred,
        RecoveryDenial::ProductUnavailable => WorthQueryOutputDemandDenialKind::ProducerUnavailable,
    };
    readmission_denial(denial(kind, "producer publication recovery"))
}

fn readmission_denial(stop: WorthQueryOutputDemandDenial) -> WorthQueryOutputDemandDenial {
    use {
        crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial as Resource,
        crate::domain_computation::request_execution::WorthQueryAdvancementDenial as Advance,
    };
    if matches!(
        stop.kind(),
        WorthQueryOutputDemandDenialKind::Cancelled
            | WorthQueryOutputDemandDenialKind::TimedOut
            | WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
            | WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
            | WorthQueryOutputDemandDenialKind::ExecutionRequest(
                Advance::Interrupted(_)
                    | Advance::Resource(
                        Resource::WorkExhausted
                            | Resource::RetainedBytesExhausted
                            | Resource::ScratchCapacityExceeded
                            | Resource::ResultCapacityExceeded
                            | Resource::WorkerLimit
                            | Resource::PolicyMemoryLimit
                            | Resource::WorkLimit
                            | Resource::MemoryLimit { .. }
                            | Resource::NativeAllocationRefused { .. }
                    )
            )
    ) {
        stop.with_recovery_posture(
            crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandRecoveryPosture::Retryable)
    } else {
        stop
    }
}
