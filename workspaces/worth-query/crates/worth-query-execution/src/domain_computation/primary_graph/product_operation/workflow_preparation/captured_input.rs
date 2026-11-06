//! Captured workflow inputs are described against a selection: a target's contract and
//! a proposal's inventory both bind to the selected runtime and product occurrence. The
//! descriptions, contracts and inventory stay the attempt's.
use crate::domain_computation::primary_graph::application_attempt::precondition_binding::bind_mutation_preconditions;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationEntityIdentity,
    WorthQueryApplicationObservedFact, WorthQuerySelectedProductOperation,
};
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_query_declaration::facade::application_program::ApplicationWorkflowNodeIdentity;
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationSourceExpectation,
    },
    application_schema::{ApplicationStructuredValueBinding, TypedMutationPreconditions},
};
use worth_query_installation::facade::ApplicationSchema;
use crate::domain_computation::primary_graph::application_attempt::workflow::captured_input::{
    PreparedWorkflowCapturedInput, Query, Scope, WorkflowCapturedInputContract,
    WorkflowCapturedInputDenial, WorkflowCapturedInputSource, WorkflowCapturedInputs,
};

impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S> {
    /// Describe a target without admitting it. In particular, a later operation's
    /// current business posture and workflow authority are deliberately not evaluated.
    /// Its actual action-time admission remains mandatory when it is selected.
    pub fn prepare_workflow_captured_input<B>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &WorthQueryProgramPublicationAccess,
        scope: &WorthQueryApplicationEntityIdentity<S, Scope<S, B>>,
        identities: &ApplicationMutationIdentities<'_, S, B>,
        preconditions: &TypedMutationPreconditions<S, B::Operation, Scope<S, B>>,
        source: WorkflowCapturedInputSource<'_, Query<S, B>>,
    ) -> Result<PreparedWorkflowCapturedInput, WorkflowCapturedInputDenial>
    where
        B: ApplicationMutationBinding<S>,
    {
        let runtime = self.application();
        let execution = phase
            .execution_request_for(runtime.product_runtime())
            .map_err(|_| WorkflowCapturedInputDenial::ForeignSelection)?;
        if scope.runtime_authority() != runtime.authority_identity()
            || scope.binding_identity() != &runtime.installed_schema().binding_identity()
        {
            return Err(WorkflowCapturedInputDenial::ForeignSelection);
        }
        if !B::REQUIRES_WORKFLOW_AUTHORITY || B::WORKFLOW_CONTROL {
            return Err(WorkflowCapturedInputDenial::Binding);
        }
        let binding = runtime
            .installed_schema()
            .installed_mutation_binding::<B>()
            .map_err(|_| WorkflowCapturedInputDenial::Binding)?;
        let graph = runtime
            .primary_graph()
            .ok_or(WorkflowCapturedInputDenial::ForeignSelection)?;
        let selection = graph.integration_handle().with_runtime(&execution, |relational| {
            crate::domain_computation::primary_graph::application_attempt::observe_indexed_entity_selection(
                relational, &execution, self.application_basis().snapshot_handle(),
                scope.identity_index_id(), scope.entity_kind(), scope.identity_locator().clone(),
                scope.identity_value().clone(), 2,
            )
        }).map_err(|cause| match cause {
            crate::domain_computation::primary_graph::application_attempt::WorthQueryIndexedSelectionRefusal::Execution(cause) =>
                WorkflowCapturedInputDenial::ResourceRead(Box::new(
                    crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::relational_read_stopped(cause, B::IDENTITY))),
            _ => WorkflowCapturedInputDenial::ResourceObservation,
        })?;
        if !matches!(&selection,
            WorthQueryApplicationObservedFact::IndexedEntitySelection { candidates, .. }
                if candidates.as_slice() == [scope.entity_id()])
        {
            return Err(WorkflowCapturedInputDenial::ResourceObservation);
        }
        let bound = bind_mutation_preconditions(
            preconditions,
            binding.operation().contracts(),
            scope.entity_name(),
            scope.entity_id(),
            graph.layout(),
        )
        .map_err(|()| WorkflowCapturedInputDenial::Preconditions)?;
        let source = match source {
            WorkflowCapturedInputSource::None => {
                if B::SourceExpectation::QUERY_IDENTIFIER.is_some() {
                    return Err(WorkflowCapturedInputDenial::Binding);
                }
                None
            }
            WorkflowCapturedInputSource::Row(source) => {
                let (identity, partition) = self
                    .describe_workflow_source::<B>(scope, identities.mutation_input(), source)
                    .map_err(WorkflowCapturedInputDenial::Source)?;
                Some((1, identity, partition))
            }
            WorkflowCapturedInputSource::ResultSet(source) => {
                let (identity, partition) = self
                    .describe_workflow_source::<B>(
                        scope,
                        identities.mutation_input(),
                        source.source(),
                    )
                    .map_err(WorkflowCapturedInputDenial::Source)?;
                Some((2, identity, partition))
            }
        };
        Ok(PreparedWorkflowCapturedInput {
            basis: crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                self.product().observation(),
            ),
            runtime: runtime.authority_identity().as_u64(),
            request_work: identities.canonical_work(),
            precondition_work: bound.canonical_work(),
            contract: WorkflowCapturedInputContract {
                resource: scope.entity_id(),
                binding: B::IDENTITY.to_owned(),
                input_type: B::InputBinding::IDENTITY.as_str().to_owned(),
                input: *identities.input_identity(),
                key: *identities.key_identity(),
                source,
                preconditions: bound.identity().copied(),
            },
            facts: vec![
                selection,
                WorthQueryApplicationObservedFact::Entity {
                    entity_id: scope.entity_id(),
                    kind: scope.entity_kind(),
                },
                WorthQueryApplicationObservedFact::Field {
                    entity_id: scope.entity_id(),
                    kind: scope.entity_kind(),
                    locator: scope.identity_locator().clone(),
                    value: scope.identity_value().clone(),
                },
            ],
        })
    }
}

impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S> {
    /// One proposal's exact qualified target contracts, bound to this selection.
    pub fn prepare_workflow_captured_inputs<B: ApplicationMutationBinding<S>, C>(
        &self,
        access: &WorthQueryProgramPublicationAccess,
        admission: &mut WorthQueryAdmittedApplicationOperation<S, B::Operation, B::Input, C>,
        inputs: Vec<(
            ApplicationWorkflowNodeIdentity,
            PreparedWorkflowCapturedInput,
        )>,
    ) -> Result<WorkflowCapturedInputs, WorkflowCapturedInputDenial> {
        WorkflowCapturedInputs::prepare::<S, B, C>(
            access,
            &self.workflow_selection_basis(),
            admission,
            inputs,
        )
    }
}
