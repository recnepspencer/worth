//! Suspending the current generated output of a selected product.
use crate::domain_computation::execution_runtime::product_world::publication_denial::{
    WorthQueryProductPublicationDenial as PublicationDenial,
    WorthQueryProductPublicationOutcome as PublicationOutcome,
};
use crate::domain_computation::primary_graph::application_contribution::advancement_denial;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::ObservedSourceInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductPublicationBindingInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;

use crate::equality_type_id::EqualityTypeId as TypeId;
use std::sync::Arc;

use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::branch::RelationalMaterializationSuspensionCompletion;

use super::super::context::WorthQuerySelectedProductOperation;
use super::{
    ProducerQualification, WorthQueryGeneratedOutputSuspensionDenial,
    WorthQueryGeneratedOutputSuspensionFailure, WorthQueryGeneratedOutputSuspensionRecovery,
    WorthQuerySuspendedGeneratedOutput,
};
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationProducerBinding,
    WorthQueryApplicationProducerProvider,
};

pub(in crate::domain_computation::primary_graph::product_operation) struct ExpectedSourceQualification {
    runtime_authority: u64,
    schema: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    runtime_source_identity:
        crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity,
    checkpoint_source_identity:
        crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity,
    selection: crate::basis::WorthQueryProductBranchReadIdentity,
}

impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema>
where
    Schema: ApplicationSchema,
{
    pub fn suspend_current_generated_output<Producer>(
        self,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        source: crate::domain_computation::primary_graph::WorthQueryObservedSource<
            <<Producer::OutputFamily as crate::domain_computation::primary_graph::WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::Query,
        >,
    ) -> Result<WorthQuerySuspendedGeneratedOutput, WorthQueryGeneratedOutputSuspensionFailure>
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        let mut outcome = None;
        let admission = self
            .application()
            .with_application_advancement(request, |phase| {
                outcome = Some(
                    self.suspend_observed_generated_output::<Producer>(&phase, request, source),
                );
            });
        // Once the callback ran, its complete owned result must survive a final
        // outer poll, including an actual unpublished-effect recovery carrier.
        if let Some(outcome) = outcome {
            return outcome;
        }
        admission.map_err(WorthQueryGeneratedOutputSuspensionFailure::ExecutionDenied)?;
        unreachable!("advancement invokes its callback")
    }

    fn suspend_observed_generated_output<Producer>(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        source: crate::domain_computation::primary_graph::WorthQueryObservedSource<
            <<Producer::OutputFamily as crate::domain_computation::primary_graph::WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::Query,
        >,
    ) -> Result<WorthQuerySuspendedGeneratedOutput, WorthQueryGeneratedOutputSuspensionFailure>
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        let source_identity = source.idempotency_identity();
        let checkpoint_source_identity = source.checkpoint_identity();
        let source_root = source.source_root();
        let crate::domain_computation::primary_graph::application_query::WorthQueryApplicationBasisSelectionIdentity::Product(selection) = source.selection().clone() else {
            return Err(WorthQueryGeneratedOutputSuspensionFailure::Qualification(
                WorthQueryGeneratedOutputSuspensionDenial::SourceMismatch,
            ));
        };
        self.suspend_qualified_generated_output::<Producer>(
            phase,
            request,
            ExpectedSourceQualification {
                runtime_authority: source.runtime_authority(),
                schema: source.schema_binding(),
                scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(source_root),
                runtime_source_identity: source_identity,
                checkpoint_source_identity,
                selection,
            },
        )
    }

    pub(in crate::domain_computation::primary_graph::product_operation) fn suspend_qualified_generated_output<
        Producer,
    >(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        source: ExpectedSourceQualification,
    ) -> Result<WorthQuerySuspendedGeneratedOutput, WorthQueryGeneratedOutputSuspensionFailure>
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        let (application, product, application_basis) = self.into_parts();
        let execution_request = phase
            .execution_request_for(application.inner.runtime.product_runtime_ref())
            .map_err(|cause| {
                WorthQueryGeneratedOutputSuspensionFailure::ExecutionDenied(cause.into())
            })?;
        let execution = &execution_request;
        drop(application_basis);
        if application
            .inner
            .installed_producers
            .provider::<Producer>()
            .is_none()
        {
            return Err(WorthQueryGeneratedOutputSuspensionFailure::Qualification(
                WorthQueryGeneratedOutputSuspensionDenial::ProducerUnavailable,
            ));
        }
        let branch = product.product_branch();
        let gate = application
            .inner
            .runtime
            .product_runtime_ref()
            .activations_ref()
            .gate(product.branch_identity())
            .map_err(|_| {
                WorthQueryGeneratedOutputSuspensionFailure::ProductActivationUnavailable
            })?;
        let _publication_admission = gate.begin_publication().map_err(|_| {
            WorthQueryGeneratedOutputSuspensionFailure::ProductActivationUnavailable
        })?;
        let observation = product.observation();
        if source.selection
            != crate::basis::WorthQueryProductBranchReadIdentity::from_observation(observation)
        {
            return Err(WorthQueryGeneratedOutputSuspensionFailure::Qualification(
                WorthQueryGeneratedOutputSuspensionDenial::SourceMismatch,
            ));
        }
        let output_occurrence = observation.lifecycle_incarnation();
        let output_generation = observation.reference_generation().get();
        let lineage = application
            .inner
            .runtime
            .primary_provider_ref()
            .graph_ref()
            .output_lineage_ref()
            .lock()
            .expect("application output lineage lock is available");
        let exact = lineage
            .qualified_output::<Producer::Operation>(
                source.runtime_authority,
                &source.schema,
                source.scope,
                observation.lifecycle_incarnation(),
                observation.reference_generation().get(),
                source.runtime_source_identity,
                source.checkpoint_source_identity,
            )
            .ok_or_else(|| {
                let denial = if lineage.has_output_at_or_before::<Producer::Operation>(
                    observation.lifecycle_incarnation(),
                    observation.reference_generation().get(),
                ) {
                    WorthQueryGeneratedOutputSuspensionDenial::SourceMismatch
                } else {
                    WorthQueryGeneratedOutputSuspensionDenial::MissingQualifiedOutput
                };
                WorthQueryGeneratedOutputSuspensionFailure::Qualification(denial)
            })?;
        drop(lineage);
        if exact.runtime_authority != application.inner.runtime.authority_identity().as_u64() {
            return Err(WorthQueryGeneratedOutputSuspensionFailure::Qualification(
                WorthQueryGeneratedOutputSuspensionDenial::ForeignRuntime,
            ));
        }
        if exact.schema != application.inner.installed_schema.binding_identity() {
            return Err(WorthQueryGeneratedOutputSuspensionFailure::Qualification(
                WorthQueryGeneratedOutputSuspensionDenial::ForeignSchema,
            ));
        }
        if source.runtime_authority != exact.runtime_authority || source.schema != exact.schema {
            return Err(WorthQueryGeneratedOutputSuspensionFailure::Qualification(
                WorthQueryGeneratedOutputSuspensionDenial::SourceMismatch,
            ));
        }
        let generated_entities = exact
            .correspondence
            .created_entity_ids()
            .collect::<Vec<_>>();
        if generated_entities.is_empty() {
            return Err(WorthQueryGeneratedOutputSuspensionFailure::Qualification(
                WorthQueryGeneratedOutputSuspensionDenial::NoGeneratedPayload,
            ));
        }
        let correspondence = exact.correspondence;
        let producer = ProducerQualification {
            binding_type: TypeId::of::<Producer>(),
            output_binding_type: TypeId::of::<Producer::Operation>(),
            binding_identity: Producer::IDENTITY,
            provider_identity: Producer::Provider::SEMANTIC_IDENTITY,
            runtime_source_identity: source.runtime_source_identity,
            checkpoint_source_identity: source.checkpoint_source_identity,
            recorded_source_identity: exact.source_identity,
            source_partition_identity: exact.source_partition_identity,
            producer_dependency_identity: exact.producer_dependency_identity,
            producing_implementation: exact.producing_implementation,
            idempotency_key_identity: exact.idempotency_key_identity,
            runtime_authority: exact.runtime_authority,
            schema: exact.schema,
            scope: exact.scope,
            output_occurrence,
            output_generation,
            observed_source_facts: exact.observed_source_facts,
        };
        let pending_suspension = application
            .inner
            .runtime
            .primary_provider_ref()
            .graph_ref()
            .with_runtime(execution, |runtime| {
                runtime
                    .owner_component_services()
                    .materialization_port()
                    .validate_generated_materialization_suspension(
                        execution,
                        product.relational_basis(),
                        &generated_entities,
                    )
            })
            .map_err(WorthQueryGeneratedOutputSuspensionFailure::Preparation)?;
        let (proposal, pending_completion) = pending_suspension.into_parts();
        let open = super::durable_publication::open_if_home(
            &product.publication_binding(),
            request,
            execution,
        )
        .map_err(super::durable_publication::map_open_suspension_denial)?;
        match open {
            super::durable_publication::DurableMaterializationOpen::Memory => {
                let candidate = application
                    .inner
                    .runtime
                    .primary_provider_ref()
                    .graph_ref()
                    .with_runtime(execution, |runtime| {
                        runtime
                            .prepare_validated_proposal(execution, proposal)
                            .map_err(worth_relational::facade::branch::RelationalMaterializationError::Commit)
                    })
                    .map_err(WorthQueryGeneratedOutputSuspensionFailure::Preparation)?;
                let completion = pending_completion.finish(&candidate);
                let prepared = product
                    .publication_binding()
                    .prepare_relational_candidate(candidate, request, true)
                    .map_err(super::durable_publication::map_suspension_denial)?;
                execute_suspension_publication(
                    application,
                    phase,
                    &product,
                    prepared,
                    branch,
                    correspondence,
                    producer,
                    completion,
                )
            }
            super::durable_publication::DurableMaterializationOpen::Retained(effects) => {
                super::durable_publication::finish_suspension_outcome(
                    application,
                    &product,
                    Ok(PublicationOutcome::ProductUnpublished(effects)),
                    branch,
                    correspondence,
                    producer,
                    None,
                )
            }
            super::durable_publication::DurableMaterializationOpen::Opened { opened, control } => {
                let pub_exec = phase
                    .execution_request_for(application.inner.runtime.product_runtime_ref())
                    .map_err(|cause| {
                        WorthQueryGeneratedOutputSuspensionFailure::ExecutionDenied(cause.into())
                    })?;
                match super::durable_publication::prepare_and_execute_opened(
                    application,
                    &product.publication_binding(),
                    opened,
                    &control,
                    request,
                    pub_exec,
                    proposal,
                    |candidate| pending_completion.finish(candidate),
                ) {
                    Ok((outcome, completion)) => super::durable_publication::finish_suspension_outcome(
                        application,
                        &product,
                        Ok(outcome),
                        branch,
                        correspondence,
                        producer,
                        Some(completion),
                    ),
                    Err((PublicationOutcome::ProductUnpublished(effects), completion)) => {
                        super::durable_publication::finish_suspension_outcome(
                            application,
                            &product,
                            Ok(PublicationOutcome::ProductUnpublished(effects)),
                            branch,
                            correspondence,
                            producer,
                            completion,
                        )
                    }
                    Err(_) => Err(WorthQueryGeneratedOutputSuspensionFailure::PublicationNoEffect),
                }
            }
        }
    }
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn execute_suspension_publication<Schema: ApplicationSchema>(
    application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    product: &crate::basis::WorthQueryProductBranchLease,
    prepared: crate::domain_computation::execution_runtime::product_world::WorthQueryPreparedProductPublication,
    branch: crate::basis::WorthQueryProductBranch,
    correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    producer: ProducerQualification,
    completion: RelationalMaterializationSuspensionCompletion,
) -> Result<WorthQuerySuspendedGeneratedOutput, WorthQueryGeneratedOutputSuspensionFailure> {
    let outcome = prepared.execute(
        phase
            .execution_request_for(application.inner.runtime.product_runtime_ref())
            .map_err(|cause| {
                WorthQueryGeneratedOutputSuspensionFailure::ExecutionDenied(cause.into())
            })?,
    );
    super::durable_publication::finish_suspension_outcome(
        application,
        product,
        outcome,
        branch,
        correspondence,
        producer,
        Some(completion),
    )
}

pub(in crate::domain_computation::primary_graph::product_operation) fn suspended_output(
    publication: WorthQueryProductPublicationBinding,
    branch: crate::basis::WorthQueryProductBranch,
    correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    producer: ProducerQualification,
    completion: RelationalMaterializationSuspensionCompletion,
    commit: worth_relational::facade::transactions::CommitResult,
) -> WorthQuerySuspendedGeneratedOutput {
    let suspension = completion
        .complete(commit)
        .expect("World returns the prepared relational transaction result");
    WorthQuerySuspendedGeneratedOutput {
        publication,
        branch,
        custody: suspension.custody,
        correspondence,
        producer,
    }
}
#[cfg(test)]
#[path = "suspend_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
