//! Suspending the current generated output of a selected product.

use std::any::TypeId;
use std::sync::Arc;

use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::branch::RelationalMaterializationSuspensionCompletion;
use worth_runtime_world::facade::RuntimeWorldPublicationOutcome;

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
        let source_identity = source.idempotency_identity();
        let checkpoint_source_identity = source.checkpoint_identity();
        let source_root = source.source_root();
        let crate::domain_computation::primary_graph::application_query::WorthQueryApplicationBasisSelectionIdentity::Product(selection) = source.selection.clone() else {
            return Err(WorthQueryGeneratedOutputSuspensionFailure::Qualification(
                WorthQueryGeneratedOutputSuspensionDenial::SourceMismatch,
            ));
        };
        self.suspend_qualified_generated_output::<Producer>(
            request,
            ExpectedSourceQualification {
                runtime_authority: source.runtime_authority,
                schema: source.schema_binding,
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
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        source: ExpectedSourceQualification,
    ) -> Result<WorthQuerySuspendedGeneratedOutput, WorthQueryGeneratedOutputSuspensionFailure>
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        let (application, product, application_basis) = self.into_parts();
        drop(application_basis);
        if application
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
            .product_runtime
            .activations
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
            .primary_provider
            .graph
            .output_lineage
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
        if exact.runtime_authority != application.runtime.authority_identity().as_u64() {
            return Err(WorthQueryGeneratedOutputSuspensionFailure::Qualification(
                WorthQueryGeneratedOutputSuspensionDenial::ForeignRuntime,
            ));
        }
        if exact.schema != application.installed_schema.binding_identity() {
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
        let prepared = application.primary_provider.graph.with_runtime(|runtime| {
            runtime
                .owner_component_services()
                .materialization_port()
                .prepare_generated_materialization_suspension(
                    product.relational_basis(),
                    &generated_entities,
                )
        });
        let prepared =
            prepared.map_err(|_| WorthQueryGeneratedOutputSuspensionFailure::Preparation)?;
        let (candidate, completion) = prepared.into_parts();
        let prepared = product
            .publication_binding()
            .prepare_relational_candidate(candidate, request, true)
            .map_err(|_| WorthQueryGeneratedOutputSuspensionFailure::PublicationNoEffect)?;
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
            idempotency_key_identity: exact.idempotency_key_identity,
            runtime_authority: exact.runtime_authority,
            schema: exact.schema,
            scope: exact.scope,
            output_occurrence,
            output_generation,
            observed_source_facts: exact.observed_source_facts,
            resources: exact.resources,
        };
        match prepared.execute() {
            RuntimeWorldPublicationOutcome::Performed(performed) => {
                let commit = performed
                    .component_results()
                    .relational_commit_result()
                    .cloned()
                    .expect("a relational-only publication returns its relational result");
                let mut publication = performed.consume();
                let observation = publication
                    .take_successor_observation()
                    .expect("the requested successor observation is retained");
                Ok(suspended_output(
                    application.materialization_publication_binding(observation),
                    branch,
                    correspondence,
                    producer,
                    completion,
                    commit,
                ))
            }
            RuntimeWorldPublicationOutcome::NoEffect(no_effect) => {
                drop(no_effect);
                Err(WorthQueryGeneratedOutputSuspensionFailure::PublicationNoEffect)
            }
            RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => {
                let product = application.unpublished_materialization(effects, &product);
                Err(
                    WorthQueryGeneratedOutputSuspensionFailure::ProductUnpublished(
                        WorthQueryGeneratedOutputSuspensionRecovery::new(
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
