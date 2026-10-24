use crate::domain_computation::execution_runtime::product_world::publication_denial::{
    WorthQueryProductPublicationDenial as PublicationDenial,
    WorthQueryProductPublicationOutcome as PublicationOutcome,
};
use crate::domain_computation::primary_graph::application_contribution::advancement_denial;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::{
    RuntimeWorldRecoveryDenial, RuntimeWorldSettledRelationalAdoptionDenial,
};

use super::{publish_recovered, unpublished_idempotency::ManagedUnpublishedRecoveryStop};
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationIdempotencyBinding,
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationIdempotencyResolutionDenial,
    WorthQueryOperationAuthorizationDenial, WorthQueryOutputDemandDenial,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::{
    WorthQueryProductUnpublishedRecovery, WorthQueryProductUnpublishedRecoveryReleaseFailure,
};

mod performed;
mod publication;
pub use performed::WorthQueryManagedApplicationRecoveryPerformed;

enum RecoveredPublicationDelivery {
    Ordinary,
    ProgramSource,
    OwningSource,
}

/// Owner result of an admitted recovery: performed publication, no effect, or
/// a new exact partial recovery replacing the previously retained handle.
pub enum WorthQueryManagedApplicationRecoveryOutcome {
    Performed(WorthQueryManagedApplicationRecoveryPerformed),
    /// World made no movement. The supplied exact recovery remains retryable.
    NoEffect,
    /// World issued a new partial; its provider claim was staged before the
    /// owner effect and now replaces the original exact recovery handle.
    ProductUnpublished {
        next: WorthQueryProductUnpublishedRecovery,
        prior_cleanup_failure: Option<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
    },
}

/// A refusal before another World owner effect. The borrowed original
/// recovery remains retained by its provider entry for a later admitted try.
#[derive(Debug)]
pub enum WorthQueryManagedApplicationRecoveryDenial {
    ExecutionDenied(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
    ForeignAdmission,
    BindingMismatch,
    ProviderBusy,
    ProviderCapacity,
    ProviderAdmission(worth_relational::facade::mvcc::CompanionPreflightStop),
    World(RuntimeWorldRecoveryDenial),
    ProductUnavailable,
    Adoption(RuntimeWorldSettledRelationalAdoptionDenial),
    Demand(WorthQueryOutputDemandDenial),
}

impl From<ManagedUnpublishedRecoveryStop> for WorthQueryManagedApplicationRecoveryDenial {
    fn from(stop: ManagedUnpublishedRecoveryStop) -> Self {
        match stop {
            ManagedUnpublishedRecoveryStop::Missing | ManagedUnpublishedRecoveryStop::Busy => {
                Self::ProviderBusy
            }
            ManagedUnpublishedRecoveryStop::Capacity => Self::ProviderCapacity,
        }
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Continues the exact retained application partial with a fresh
    /// admitted operation. It never reruns the Relational candidate or handler.
    pub fn recover_admitted_unpublished_application<Operation, Input, Scope>(
        &self,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        WorthQueryManagedApplicationRecoveryOutcome,
        WorthQueryManagedApplicationRecoveryDenial,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.with_application_advancement(admission.publication_request(), |phase| {
            self.recover_admitted_unpublished_application_in_advancement(
                &phase,
                recovery,
                admission,
                idempotency,
            )
        })
        .map_err(WorthQueryManagedApplicationRecoveryDenial::ExecutionDenied)?
    }

    /// Continues recovery inside the Publication call that already admitted its readers.
    pub fn recover_admitted_unpublished_application_in_advancement<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        WorthQueryManagedApplicationRecoveryOutcome,
        WorthQueryManagedApplicationRecoveryDenial,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.recover_admitted_unpublished_application_delivery(
            phase,
            recovery,
            admission,
            idempotency,
            RecoveredPublicationDelivery::Ordinary,
        )
        .map(|(outcome, _)| outcome)
    }

    /// Retains the original performed output carrier before optional Query
    /// projection. Only the program-source owner uses this handoff.
    pub fn recover_admitted_unpublished_program_source<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<(WorthQueryManagedApplicationRecoveryOutcome, Option<crate::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource>), WorthQueryManagedApplicationRecoveryDenial>
    where Input: Clone + Send + Sync + 'static,
    {
        self.recover_admitted_unpublished_application_delivery(
            phase,
            recovery,
            admission,
            idempotency,
            RecoveredPublicationDelivery::ProgramSource,
        )
    }

    /// Owning no-output recovery retains cleanup without constructing an output carrier.
    pub fn recover_admitted_unpublished_source_operation<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        WorthQueryManagedApplicationRecoveryOutcome,
        WorthQueryManagedApplicationRecoveryDenial,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.recover_admitted_unpublished_application_delivery(
            phase,
            recovery,
            admission,
            idempotency,
            RecoveredPublicationDelivery::OwningSource,
        )
        .map(|(outcome, _)| outcome)
    }

    fn recover_admitted_unpublished_application_delivery<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        delivery: RecoveredPublicationDelivery,
    ) -> Result<(WorthQueryManagedApplicationRecoveryOutcome, Option<crate::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource>), WorthQueryManagedApplicationRecoveryDenial>
    where Input: Clone + Send + Sync + 'static,
    {
        use WorthQueryManagedApplicationRecoveryDenial as Denial;
        let request = phase
            .execution_request_for(&self.product_runtime)
            .map_err(|cause| Denial::ExecutionDenied(cause.into()))?;
        {
            admission
                .validate_current_authority()
                .map_err(Denial::Authorization)?;
            if !admission.belongs_to(
                self.runtime.authority_identity(),
                &self.installed_schema.binding_identity(),
            ) {
                return Err(Denial::ForeignAdmission);
            }
            let product = admission
                .graph_work()
                .mutation_product()
                .ok_or(Denial::ForeignAdmission)?
                .publication_binding();
            let bound = idempotency
                .bind_operation(admission.operation_definition_identity())
                .bind_operation_scope(admission.operation_scope_binding())
                .bind_preconditions(admission.mutation_preconditions().identity())
                .bind_governed_input(admission.governed_input_identity())
                .bind_governed_proposal(admission.governed_proposal_identity());
            let command_lane = self.primary_provider.correction_commit_lane(&bound);
            let command_coordination = command_lane.as_ref().map(|lane| lane.enter());
            let lane = self
                .primary_provider
                .application_branch_commit_lane(product.observation());
            let coordination = lane.enter();
            let mut guard = self
                .primary_provider
                .begin_managed_unpublished_recovery(recovery.record_handle(), product.recovery())
                .map_err(Denial::from)?;
            // The handler already supplied this component to the retained attempt.
            // Fresh authority, scope, input and preconditions still compare in full.
            let definition_identity = guard.attempt_mut().conditional_definition_identity();
            let idempotency = if idempotency.conditional_definition_identity().is_none() {
                idempotency.bind_conditional_definition(definition_identity.as_ref())
            } else {
                idempotency
            };
            let bound = if bound.conditional_definition_identity().is_none() {
                bound.bind_conditional_definition(definition_identity.as_ref())
            } else {
                bound
            };
            if !guard.attempt_mut().matches_fresh_binding(
                bound,
                admission.operation_scope_binding(),
                product.observation(),
            ) {
                return Err(Denial::BindingMismatch);
            }
            let needs_settlement = recovery
                .inspect()
                .map_err(Denial::World)?
                .relational_requires_settlement();
            if needs_settlement {
                recovery
                    .continue_owner_settlement()
                    .map_err(Denial::World)?;
            }
            let unpublished = recovery.inspect().map_err(Denial::World)?;
            let lease = self
                .product_runtime
                .admit_product_occurrence(product.observation().lifecycle_incarnation())
                .map_err(|_| Denial::ProductUnavailable)?;
            let binding = lease.publication_binding();
            if unpublished.expected_product() != binding.observation() {
                return Err(Denial::BindingMismatch);
            }
            let outcome = publication::publish(
                self,
                request,
                admission.publication_request(),
                &binding,
                recovery,
                unpublished,
                &mut guard,
            )?;
            drop(binding);
            drop(lease);
            match outcome {
                Err(PublicationDenial::Stopped(stop)) => {
                    Err(Denial::ExecutionDenied(advancement_denial(stop)))
                }
                Err(PublicationDenial::Refused(_)) => {
                    Ok((WorthQueryManagedApplicationRecoveryOutcome::NoEffect, None))
                }
                Ok(PublicationOutcome::ProductUnpublished(effects)) => {
                    let next = effects.into_recovery();
                    guard.retain_successor(next.record_handle());
                    let prior_cleanup_failure = self
                        .release_product_publication_recovery_in_advancement(
                            phase,
                            recovery.clone(),
                            0,
                        )
                        .err();
                    Ok((
                        WorthQueryManagedApplicationRecoveryOutcome::ProductUnpublished {
                            next,
                            prior_cleanup_failure,
                        },
                        None,
                    ))
                }
                Ok(PublicationOutcome::Performed((publication, definition_generation))) => {
                    let session = guard
                        .take_attempt()
                        .complete(publication, definition_generation);
                    let source = match delivery {
                        RecoveredPublicationDelivery::Ordinary
                        | RecoveredPublicationDelivery::OwningSource => None,
                        RecoveredPublicationDelivery::ProgramSource => {
                            super::recovered_program_source_delivery::consume_first_if_armed(
                                self,
                                || session.take_recovered_output_source(),
                            );
                            Some(session.take_recovered_output_source())
                        }
                    };
                    let publication_failure =
                        publish_recovered(&request, &self.primary_provider, session).err();
                    guard.finish();
                    let (prior_cleanup_failure, pending_cleanup) = match delivery {
                        RecoveredPublicationDelivery::Ordinary => (
                            self.release_product_publication_recovery_in_advancement(
                                phase,
                                recovery.clone(),
                                0,
                            )
                            .err(),
                            None,
                        ),
                        RecoveredPublicationDelivery::ProgramSource
                        | RecoveredPublicationDelivery::OwningSource => {
                            (None, Some(recovery.clone()))
                        }
                    };
                    drop(coordination);
                    // The installed pending/completed handoff now owns visibility.
                    drop(command_coordination);
                    let read = self.resolve_admitted_application_idempotency(
                        phase,
                        admission,
                        idempotency,
                    );
                    Ok((
                        WorthQueryManagedApplicationRecoveryOutcome::Performed(
                            WorthQueryManagedApplicationRecoveryPerformed {
                                conditional_definition_identity: definition_identity,
                                read,
                                publication_failure,
                                prior_cleanup_failure,
                                pending_cleanup,
                            },
                        ),
                        source,
                    ))
                }
            }
        }
    }
}
