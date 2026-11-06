//! Effect-free correspondence for an original guarded request. Never mutation authority.
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationScopeBinding,
        ApplicationMutationSourceExpectation,
    },
    application_schema::ApplicationStructuredValueBinding,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use {
    crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact,
    crate::domain_computation::primary_graph::WorthQueryObservedResultSet,
    crate::domain_computation::primary_graph::WorthQueryObservedSource,
    crate::domain_computation::primary_graph::WorthQuerySourceExpectationDenial,
};
mod inventory;
mod support;
pub(in crate::domain_computation::primary_graph) use inventory::inventory_identity;
pub use inventory::WorkflowCapturedInputs;
pub(in crate::domain_computation::primary_graph) use inventory::WorkflowCapturedTarget;
pub(in crate::domain_computation::primary_graph) use support::WorkflowCapturedInputSupport;

pub(in crate::domain_computation::primary_graph) type Scope<S, B> = <<B as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope;
pub(in crate::domain_computation::primary_graph) type Query<S, B> = <<B as ApplicationMutationBinding<S>>::SourceExpectation as ApplicationMutationSourceExpectation<S>>::Query;

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
    pub(in crate::domain_computation::primary_graph) basis: crate::basis::WorthQueryProductBranchReadIdentity,
    pub(in crate::domain_computation::primary_graph) runtime: u64,
    pub(in crate::domain_computation::primary_graph) request_work:
        worth_query_declaration::facade::application_operation::ApplicationCanonicalWork,
    pub(in crate::domain_computation::primary_graph) precondition_work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
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
