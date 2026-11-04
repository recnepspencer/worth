use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_installation::facade::ApplicationSchema;

#[path = "idempotency_resolution/denial.rs"]
mod denial;
#[path = "idempotency_resolution/external_settlement.rs"]
mod external_settlement;

pub use denial::{
    WorthQueryApplicationIdempotencyResolutionDenial,
    WorthQueryApplicationIdempotencyResolutionDenialKind,
};

use super::{WorthQueryApplicationCommitReceipt, WorthQueryApplicationIdempotencyBinding};

use crate::domain_computation::application_aftermath::WorthQueryAdmittedIdempotencyRead;
use crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolution;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryPrimaryGraphApplicationRuntime,
};

/// What an idempotency key already means on the current product branch, read
/// through `resolve_admitted_application_idempotency` without committing
/// anything.
#[derive(Debug)]
pub enum WorthQueryApplicationIdempotencyResolution {
    ProductPreparationPending(crate::domain_computation::WorthQueryProductPreparationPending),
    /// No commit is recorded under this key.
    Unseen,
    RecoveredCommitted(crate::domain_computation::primary_graph::provider::WorthQueryRecoveredApplicationCommitReceipt),
    /// The key was already used for a different intent.
    IntentDrift,
}

/// Owner custody for one admitted guarded workflow mutation. Only `Committed`
/// carries authority to settle the workflow's operation transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryGuardedWorkflowOperationCustody {
    Unseen,
    Committed(WorthQueryApplicationCommitReceipt),
    RecoveredCommitted(crate::domain_computation::primary_graph::provider::WorthQueryRecoveredApplicationCommitReceipt),
    /// The original dispatch has a matching World-performed terminal at its
    /// external-effect owner. Acceptance still rechecks that owner at commit.
    ExternallySettled(WorthQueryApplicationCommitReceipt),
    DispatchPending(WorthQueryApplicationCommitReceipt),
    IntentDrift,
    PublicationPending,
    ProductUnpublished(worth_runtime_world::facade::ProductUnpublishedRecoveryHandle),
    Indeterminate(WorthQueryApplicationIdempotencyResolutionDenial),
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn resolve_admitted_guarded_workflow_operation_custody<
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        binding: WorthQueryApplicationIdempotencyBinding,
        transition_identity: &[u8; 32],
        completed: Option<&WorthQueryApplicationCommitReceipt>,
    ) -> Result<
        WorthQueryGuardedWorkflowOperationCustody,
        WorthQueryApplicationIdempotencyResolutionDenial,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        if !binding.matches_guarded_workflow_effect(transition_identity) {
            return Err(WorthQueryApplicationIdempotencyResolutionDenial::foreign_admission());
        }
        admission
            .refuse_stale_home_authority(self.product_runtime.owner.owner_identity())
            .map_err(WorthQueryApplicationIdempotencyResolutionDenial::from_authorization)?;
        admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationIdempotencyResolutionDenial::from_authorization)?;
        if !admission.belongs_to(
            self.runtime.authority_identity(),
            &self.installed_schema.binding_identity(),
        ) {
            return Err(WorthQueryApplicationIdempotencyResolutionDenial::foreign_admission());
        }
        let scope = admission.operation_scope_binding();
        let original_scope = if let Some(receipt) = completed {
            let original = receipt.principal_scope();
            // Completed custody keeps its actual executor. The fresh actor
            // must independently admit the same installed operation/subject,
            // but cannot rewrite the original effect's principal-bound key.
            if receipt.runtime_authority() != self.runtime.authority_identity()
                || receipt.installed_operation() != admission.installed_operation_identity_bytes()
                || original.binding_identity() != scope.binding_identity()
                || original.operation_authority_identity() != scope.operation_authority_identity()
                || original.scope() != scope.scope()
                || receipt.product_branch()
                    != admission
                        .graph_work()
                        .mutation_product()
                        .ok_or_else(
                            WorthQueryApplicationIdempotencyResolutionDenial::foreign_admission,
                        )?
                        .product_branch()
            {
                return Err(WorthQueryApplicationIdempotencyResolutionDenial::foreign_admission());
            }
            original
        } else {
            scope
        };
        let bound = binding
            .bind_operation(admission.operation_definition_identity())
            .bind_operation_scope(original_scope)
            .bind_preconditions(admission.mutation_preconditions().identity())
            .bind_governed_input(admission.governed_input_identity())
            .bind_governed_proposal(admission.governed_proposal_identity());
        if completed
            .is_some_and(|receipt| receipt.authority_binding().idempotency_binding() != bound)
        {
            return Ok(WorthQueryGuardedWorkflowOperationCustody::IntentDrift);
        }
        let product = admission
            .graph_work()
            .mutation_product()
            .ok_or_else(WorthQueryApplicationIdempotencyResolutionDenial::foreign_admission)?
            .publication_binding();
        let command_lane = self.primary_provider.correction_commit_lane(&bound);
        let _command = command_lane.as_ref().map(|lane| lane.enter());
        let commit_lane = self
            .primary_provider
            .application_branch_commit_lane(product.observation());
        let coordination = commit_lane.enter();
        let proof = self
            .authorize_idempotency_inspection(execution, admission, &coordination)
            .map_err(WorthQueryApplicationIdempotencyResolutionDenial::from_authorization)?;
        let custody = proof
            .govern((), |()| {
                self.primary_provider
                    .resolve_guarded_workflow_operation_custody(execution, bound, &product)
            })
            .map_err(|(_, denial)| {
                WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(denial)
            })?;
        use crate::domain_computation::primary_graph::provider::WorthQueryProviderGuardedWorkflowOperationCustody as Owner;
        Ok(match custody {
            Owner::Readmitted(receipt) => {
                if let Some(original) = completed {
                    if original.commit_reference() != receipt.commit_reference() {
                        return Ok(WorthQueryGuardedWorkflowOperationCustody::IntentDrift);
                    }
                    let original = original.clone();
                    if original.dispatch_outbox().and_then(|record| record.inbound()).is_some() {
                        match self.resolve_guarded_workflow_external_settlement(&original) {
                            Ok(true) => WorthQueryGuardedWorkflowOperationCustody::ExternallySettled(original),
                            Ok(false) => WorthQueryGuardedWorkflowOperationCustody::DispatchPending(original),
                            Err(_) => WorthQueryGuardedWorkflowOperationCustody::Indeterminate(WorthQueryApplicationIdempotencyResolutionDenial::provider_unavailable()),
                        }
                    } else if super::workflow_transition_program::operation_receipt_requires_recovery(&original) {
                        WorthQueryGuardedWorkflowOperationCustody::DispatchPending(original)
                    } else {
                        WorthQueryGuardedWorkflowOperationCustody::Committed(original)
                    }
                } else {
                    WorthQueryGuardedWorkflowOperationCustody::RecoveredCommitted(receipt)
                }
            }
            Owner::Unseen => WorthQueryGuardedWorkflowOperationCustody::Unseen,
            Owner::IntentDrift => WorthQueryGuardedWorkflowOperationCustody::IntentDrift,
            Owner::PublicationPending => {
                WorthQueryGuardedWorkflowOperationCustody::PublicationPending
            }
            Owner::ProductUnpublished(handle) => {
                WorthQueryGuardedWorkflowOperationCustody::ProductUnpublished(handle)
            }
            Owner::Indeterminate(denial) => {
                WorthQueryGuardedWorkflowOperationCustody::Indeterminate(
                    WorthQueryApplicationIdempotencyResolutionDenial::from_provider(denial),
                )
            }
        })
    }

    pub fn resolve_admitted_application_idempotency<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<WorthQueryAdmittedIdempotencyRead, WorthQueryApplicationIdempotencyResolutionDenial>
    where
        Input: Clone + Send + Sync + 'static,
    {
        let execution = phase.execution_request_for(&self.product_runtime).map_err(|_| {
            WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(
                crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(
                    crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase),
                    "idempotency-resolution",
                ),
            )
        })?;
        self.resolve_admitted_application_idempotency_with_request(&execution, admission, binding)
    }

    pub(in crate::domain_computation) fn resolve_admitted_application_idempotency_with_request<
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<WorthQueryAdmittedIdempotencyRead, WorthQueryApplicationIdempotencyResolutionDenial>
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.resolve_admitted_application_idempotencies(execution, admission, [binding])
            .map(|mut reads| {
                reads
                    .pop()
                    .expect("one requested idempotency binding yields one read")
            })
    }

    pub(in crate::domain_computation::primary_graph) fn resolve_admitted_application_idempotencies<
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        bindings: impl IntoIterator<Item = WorthQueryApplicationIdempotencyBinding>,
    ) -> Result<
        Vec<WorthQueryAdmittedIdempotencyRead>,
        WorthQueryApplicationIdempotencyResolutionDenial,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        admission
            .refuse_stale_home_authority(self.product_runtime.owner.owner_identity())
            .map_err(WorthQueryApplicationIdempotencyResolutionDenial::from_authorization)?;
        admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationIdempotencyResolutionDenial::from_authorization)?;
        if !admission.belongs_to(
            self.runtime.authority_identity(),
            &self.installed_schema.binding_identity(),
        ) {
            return Err(WorthQueryApplicationIdempotencyResolutionDenial::foreign_admission());
        }
        let bindings = bindings
            .into_iter()
            .map(|read_for| {
                let binding = read_for
                    .bind_operation(admission.operation_definition_identity())
                    .bind_operation_scope(admission.operation_scope_binding())
                    .bind_preconditions(admission.mutation_preconditions().identity())
                    .bind_governed_input(admission.governed_input_identity())
                    .bind_governed_proposal(admission.governed_proposal_identity());
                (read_for, binding)
            })
            .collect::<Vec<_>>();
        let command_lanes = self
            .primary_provider
            .correction_commit_lanes(bindings.iter().map(|(_, binding)| *binding));
        let _commands = command_lanes
            .iter()
            .map(|lane| lane.enter())
            .collect::<Vec<_>>();
        let product = admission
            .graph_work()
            .mutation_product()
            .ok_or_else(WorthQueryApplicationIdempotencyResolutionDenial::foreign_admission)?
            .publication_binding();
        let commit_lane = self
            .primary_provider
            .application_branch_commit_lane(product.observation());
        let coordination = commit_lane.enter();
        let proof = self
            .authorize_idempotency_inspection(execution, admission, &coordination)
            .map_err(WorthQueryApplicationIdempotencyResolutionDenial::from_authorization)?;
        let resolutions = proof
            .govern((), |()| {
                bindings
                    .iter()
                    .map(|(_, binding)| {
                        self.primary_provider
                            .resolve_idempotency_binding_at_product(execution, *binding, &product)
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(|(_, denial)| {
                WorthQueryApplicationIdempotencyResolutionDenial::from_authorization(denial)
            })?;
        bindings
            .into_iter()
            .zip(resolutions)
            .map(|((read_for, _binding), resolution)| match resolution {
                Ok(WorthQueryProviderIdempotencyResolution::Readmitted(receipt)) => Ok(WorthQueryAdmittedIdempotencyRead::mint(read_for, WorthQueryApplicationIdempotencyResolution::RecoveredCommitted(receipt))),
                Ok(WorthQueryProviderIdempotencyResolution::Absent) => {
                    Ok(WorthQueryAdmittedIdempotencyRead::mint(
                        read_for,
                        WorthQueryApplicationIdempotencyResolution::Unseen,
                    ))
                }
                Ok(WorthQueryProviderIdempotencyResolution::Drift) => {
                    Ok(WorthQueryAdmittedIdempotencyRead::mint(
                        read_for,
                        WorthQueryApplicationIdempotencyResolution::IntentDrift,
                    ))
                }
                Ok(WorthQueryProviderIdempotencyResolution::PreparationPending {
                    handle,
                    stage,
                }) => Ok(WorthQueryAdmittedIdempotencyRead::mint(
                    read_for,
                    WorthQueryApplicationIdempotencyResolution::ProductPreparationPending(
                        self.pending_preparation_resolution(handle, stage),
                    ),
                )),
                Ok(WorthQueryProviderIdempotencyResolution::PreparationBusy) => Err(WorthQueryApplicationIdempotencyResolutionDenial::from_provider(
                    crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial::PreparationBusy)),
                Ok(WorthQueryProviderIdempotencyResolution::Unpublished) => {
                    Err(WorthQueryApplicationIdempotencyResolutionDenial::provider_unavailable())
                }
                Err(denial) => {
                    Err(WorthQueryApplicationIdempotencyResolutionDenial::from_provider(denial))
                }
            })
            .collect()
    }
}
