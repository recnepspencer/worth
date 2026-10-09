use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::{
    RuntimeWorldPublicationOutcome, RuntimeWorldRecoveryDenial,
    RuntimeWorldSettledRelationalAdoptionDenial,
};

use super::{publish_recovered, unpublished_idempotency::ManagedUnpublishedRecoveryStop};
use crate::domain_computation::application_aftermath::WorthQueryAdmittedIdempotencyRead;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationIdempotencyBinding,
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationIdempotencyResolutionDenial,
    WorthQueryOperationAuthorizationDenial, WorthQueryOutputDemandDenial,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::{
    WorthQueryProductUnpublishedApplication, WorthQueryProductUnpublishedRecovery,
    WorthQueryProductUnpublishedRecoveryReleaseFailure, WorthQueryProviderSessionFailure,
};

enum RecoveredPublicationDelivery {
    Ordinary,
    ProgramSource,
}

/// A performed World successor has been handed to the normal Query pending
/// publication owner before this result is returned. A failed reply or pending
/// projection remains visible in the admitted idempotency read's posture.
pub struct WorthQueryManagedApplicationRecoveryPerformed {
    read:
        Result<WorthQueryAdmittedIdempotencyRead, WorthQueryApplicationIdempotencyResolutionDenial>,
    publication_failure: Option<WorthQueryProviderSessionFailure>,
    prior_cleanup_failure: Option<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
}

impl WorthQueryManagedApplicationRecoveryPerformed {
    pub fn into_parts(
        self,
    ) -> (
        Result<WorthQueryAdmittedIdempotencyRead, WorthQueryApplicationIdempotencyResolutionDenial>,
        Option<WorthQueryProviderSessionFailure>,
        Option<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
    ) {
        (
            self.read,
            self.publication_failure,
            self.prior_cleanup_failure,
        )
    }
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
    ConditionalDefinitionRequired,
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
            ManagedUnpublishedRecoveryStop::Admission(stop) => Self::ProviderAdmission(stop),
        }
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Continues the exact retained ordinary application partial with a fresh
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
        self.recover_admitted_unpublished_application_delivery(
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
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<(WorthQueryManagedApplicationRecoveryOutcome, Option<crate::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource>), WorthQueryManagedApplicationRecoveryDenial>
    where Input: Clone + Send + Sync + 'static,
    {
        self.recover_admitted_unpublished_application_delivery(
            recovery,
            admission,
            idempotency,
            RecoveredPublicationDelivery::ProgramSource,
        )
    }

    fn recover_admitted_unpublished_application_delivery<Operation, Input, Scope>(
        &self,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        delivery: RecoveredPublicationDelivery,
    ) -> Result<(WorthQueryManagedApplicationRecoveryOutcome, Option<crate::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource>), WorthQueryManagedApplicationRecoveryDenial>
    where Input: Clone + Send + Sync + 'static,
    {
        self.with_application_advancement(admission.publication_request(), |phase| {
            use WorthQueryManagedApplicationRecoveryDenial as Denial;
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
            let lane = self
                .primary_provider
                .application_branch_commit_lane(product.observation())
                .map_err(|_| Denial::ProviderCapacity)?;
            let coordination = lane.enter();
            let mut guard = self
                .primary_provider
                .begin_managed_unpublished_recovery(recovery.record_handle())
                .map_err(Denial::from)?;
            if !guard.attempt_mut().matches_fresh_binding(
                bound,
                admission.operation_scope_binding(),
                product.observation(),
            ) {
                return Err(Denial::BindingMismatch);
            }
            if !guard.attempt_mut().ordinary_adoption_supported() {
                return Err(Denial::ConditionalDefinitionRequired);
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
            let prepared = binding
                .prepare_settled_relational_adoption(&unpublished, admission.publication_request())
                .map_err(Denial::Adoption)?;
            let successor = prepared.unpublished_recovery_handle();
            guard
                .attempt_mut()
                .prepare_fresh_slot(&self.primary_provider, prepared.planned_successor())
                .map_err(Denial::Demand)?;
            guard.reserve_successor(&successor).map_err(Denial::from)?;
            guard
                .attempt_mut()
                .prepare_fresh_terminal(successor.clone())
                .map_err(Denial::ProviderAdmission)?;
            let world_recovery = binding.recovery();
            drop(unpublished);
            let outcome = prepared.execute(
                phase
                    .execution_request_for(&self.product_runtime)
                    .expect("private progression uses its admitted runtime phase"),
            );
            drop(binding);
            drop(lease);
            match outcome {
                RuntimeWorldPublicationOutcome::NoEffect(_) => {
                    Ok((WorthQueryManagedApplicationRecoveryOutcome::NoEffect, None))
                }
                RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => {
                    let next = WorthQueryProductUnpublishedApplication::new(
                        effects,
                        world_recovery,
                        self.primary_provider.unpublished_idempotency_disposition(),
                    )
                    .into_recovery();
                    guard.retain_successor(next.record_handle());
                    let prior_cleanup_failure = self
                        .release_product_publication_recovery(recovery.clone(), 0)
                        .err();
                    Ok((
                        WorthQueryManagedApplicationRecoveryOutcome::ProductUnpublished {
                            next,
                            prior_cleanup_failure,
                        },
                        None,
                    ))
                }
                RuntimeWorldPublicationOutcome::Performed(publication) => {
                    let session = guard.take_attempt().complete(publication, None);
                    let source = match delivery {
                        RecoveredPublicationDelivery::Ordinary => None,
                        RecoveredPublicationDelivery::ProgramSource => {
                            Some(session.take_recovered_output_source())
                        }
                    };
                    let publication_failure =
                        publish_recovered(&self.primary_provider, session).err();
                    guard.finish();
                    let prior_cleanup_failure = self
                        .release_product_publication_recovery(recovery.clone(), 0)
                        .err();
                    drop(coordination);
                    let read =
                        self.resolve_admitted_application_idempotency(admission, idempotency);
                    Ok((
                        WorthQueryManagedApplicationRecoveryOutcome::Performed(
                            WorthQueryManagedApplicationRecoveryPerformed {
                                read,
                                publication_failure,
                                prior_cleanup_failure,
                            },
                        ),
                        source,
                    ))
                }
            }
        })
        .map_err(WorthQueryManagedApplicationRecoveryDenial::ExecutionDenied)?
    }
}
