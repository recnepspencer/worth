//! Effect-free correspondence for an original guarded request. Never mutation authority.
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationScopeBinding,
        ApplicationMutationSourceExpectation,
    },
    application_schema::{ApplicationStructuredValueBinding, TypedMutationPreconditions},
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

use crate::domain_computation::primary_graph::{
    application_attempt::precondition_binding::bind_mutation_preconditions,
    WorthQueryAdvancementPhase, WorthQueryApplicationEntityIdentity,
    WorthQueryApplicationObservedFact, WorthQueryObservedResultSet, WorthQueryObservedSource,
    WorthQuerySelectedProductOperation, WorthQuerySourceExpectationDenial,
};
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
mod inventory;
mod support;
pub(in crate::domain_computation::primary_graph) use inventory::inventory_identity;
pub use inventory::WorkflowCapturedInputs;
pub(in crate::domain_computation::primary_graph) use inventory::WorkflowCapturedTarget;
pub(in crate::domain_computation::primary_graph) use support::WorkflowCapturedInputSupport;

type Scope<S, B> = <<B as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope;
type Query<S, B> = <<B as ApplicationMutationBinding<S>>::SourceExpectation as ApplicationMutationSourceExpectation<S>>::Query;

/// Borrowed native source custody; a result set is not replaceable with one row.
pub enum WorkflowCapturedInputSource<'a, Q> {
    None,
    Row(&'a WorthQueryObservedSource<Q>),
    ResultSet(&'a WorthQueryObservedResultSet<Q>),
}

/// Owner-derived descriptive request contract. No public raw-digest constructor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowCapturedInputContract {
    pub(in crate::domain_computation::primary_graph) resource: EntityId,
    pub(in crate::domain_computation::primary_graph) binding: String,
    pub(in crate::domain_computation::primary_graph) input_type: String,
    pub(in crate::domain_computation::primary_graph) input: [u8; 32],
    pub(in crate::domain_computation::primary_graph) key: [u8; 32],
    pub(in crate::domain_computation::primary_graph) source: Option<(u64, [u8; 32], [u8; 32])>,
    pub(in crate::domain_computation::primary_graph) preconditions: Option<[u8; 32]>,
}

/// Prepared description and its resource-resolution dependencies. It performs no action.
pub struct PreparedWorkflowCapturedInput {
    pub(in crate::domain_computation::primary_graph) contract: WorkflowCapturedInputContract,
    pub(in crate::domain_computation::primary_graph) facts: Vec<WorthQueryApplicationObservedFact>,
    pub(super) basis: crate::basis::WorthQueryProductBranchReadIdentity,
    pub(super) runtime: u64,
    pub(super) request_work:
        worth_query_declaration::facade::application_operation::ApplicationCanonicalWork,
    pub(super) precondition_work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
}

#[derive(Debug)]
pub enum WorkflowCapturedInputDenial {
    ForeignSelection,
    Binding,
    Preconditions,
    DuplicateTarget,
    TargetMismatch,
    ResourceObservation,
    ResourceRead(Box<crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>),
    Source(WorthQuerySourceExpectationDenial),
}

impl WorkflowCapturedInputContract {
    /// Compare the complete original request after fresh native authorization,
    /// but before its source owner or any effect is consumed. Descriptive equality
    /// does not replace currentness, capability or workflow-slot admission.
    pub fn validate_admission<S: ApplicationSchema, B, C>(
        &self,
        _: &WorthQueryProgramPublicationAccess,
        admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<S, B::Operation, B::Input, C>,
        identities: &ApplicationMutationIdentities<'_, S, B>,
        source: WorkflowCapturedInputSource<'_, Query<S, B>>,
    ) -> Result<(), WorkflowCapturedInputDenial>
    where
        B: ApplicationMutationBinding<S>,
    {
        let source = match source {
            WorkflowCapturedInputSource::None => None,
            WorkflowCapturedInputSource::Row(s) => {
                Some((1, s.idempotency_identity().bytes(), s.partition_identity()))
            }
            WorkflowCapturedInputSource::ResultSet(s) => Some((
                2,
                s.source().idempotency_identity().bytes(),
                s.source().partition_identity(),
            )),
        };
        if self.resource != admission.scope_entity_id()
            || self.binding != B::IDENTITY
            || self.input_type != B::InputBinding::IDENTITY.as_str()
            || self.input != *identities.input_identity()
            || self.key != *identities.key_identity()
            || self.preconditions.as_ref() != admission.mutation_preconditions().identity()
            || self.source != source
        {
            return Err(WorkflowCapturedInputDenial::TargetMismatch);
        }
        Ok(())
    }
}
impl std::fmt::Display for WorkflowCapturedInputDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "captured workflow input refused: {self:?}")
    }
}
impl std::error::Error for WorkflowCapturedInputDenial {}

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
            .execution_request_for(&runtime.product_runtime)
            .map_err(|_| WorkflowCapturedInputDenial::ForeignSelection)?;
        if scope.runtime_authority() != runtime.runtime.authority_identity()
            || scope.binding_identity() != &runtime.installed_schema.binding_identity()
        {
            return Err(WorkflowCapturedInputDenial::ForeignSelection);
        }
        if !B::REQUIRES_WORKFLOW_AUTHORITY || B::WORKFLOW_CONTROL {
            return Err(WorkflowCapturedInputDenial::Binding);
        }
        let binding = runtime
            .installed_schema
            .installed_mutation_binding::<B>()
            .map_err(|_| WorkflowCapturedInputDenial::Binding)?;
        let graph = runtime
            .runtime
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
            runtime: runtime.runtime.authority_identity().as_u64(),
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
