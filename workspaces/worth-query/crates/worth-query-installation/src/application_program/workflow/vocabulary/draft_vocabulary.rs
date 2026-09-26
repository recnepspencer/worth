//! Installed members as declared refs, so an untrusted draft that names them
//! by identifier resolves to exactly what a typed author would have built.
//! A resolved ref carries no authority: the definition built from it still
//! validates and binds like any other.

use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_program::{
        ApplicationWorkflowApprovalRef, ApplicationWorkflowAssessmentRef,
        ApplicationWorkflowConditionRef, ApplicationWorkflowOperationRef, ApplicationWorkflowSpec,
    },
    application_query::{ApplicationQueryBinding, ApplicationQueryMarkerIdentity},
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};

use super::{
    InstalledWorkflowAssessment, InstalledWorkflowCondition, InstalledWorkflowOperation,
    WorthQueryInstalledApplicationWorkflowSpec,
};

impl InstalledWorkflowOperation {
    pub(super) fn declared<Spec, Binding>() -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Binding: ApplicationMutationBinding<Spec::Schema>,
    {
        let reference = ApplicationWorkflowOperationRef::declared_binding::<Spec, Binding>();
        Self {
            marker: reference.operation_type(),
            identifier: reference.identifier(),
            input_type: reference.input_type().clone(),
            binding_type: std::any::TypeId::of::<Binding>(),
            binding_identity: Binding::IDENTITY,
            requires_workflow_authority: Binding::REQUIRES_WORKFLOW_AUTHORITY,
            reference,
        }
    }
}

impl InstalledWorkflowAssessment {
    pub(super) fn declared<Spec, Binding>() -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Binding: ApplicationQueryBinding<Spec::Schema> + 'static,
        Binding::Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
    {
        let reference = ApplicationWorkflowAssessmentRef::declared::<Spec, Binding::Query>();
        Self {
            query_marker: reference.query_type(),
            query_identifier: reference.identifier(),
            parameter_type: reference.parameter_type().clone(),
            result_type: reference.result_type().clone(),
            binding_identity: Binding::IDENTITY,
            reference,
        }
    }
}

impl InstalledWorkflowCondition {
    pub(super) fn declared<Spec, Binding>() -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Binding: ApplicationQueryBinding<Spec::Schema> + 'static,
        Binding::Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
        <Binding::Query as ApplicationQueryMarkerIdentity<Spec::Schema>>::ResultBinding:
            ApplicationStructuredValueBinding<Value = bool>,
    {
        let reference = ApplicationWorkflowConditionRef::declared::<Spec, Binding::Query>();
        Self {
            query_marker: reference.query_type(),
            query_identifier: reference.identifier(),
            parameter_type: reference.parameter_type().clone(),
            result_type: reference.result_type().clone(),
            binding_identity: Binding::IDENTITY,
            reference,
        }
    }
}

impl<Schema, Spec, Program> WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    /// The installed operation a draft names by identifier and, when it names
    /// one, binding identity. Naming no binding builds the unbound ref.
    pub fn draft_operation(
        &self,
        identifier: &str,
        binding: Option<&str>,
    ) -> Option<ApplicationWorkflowOperationRef> {
        let installed = self.operations.iter().find(|operation| {
            operation.identifier == identifier
                && binding.is_none_or(|binding| operation.binding_identity == binding)
        })?;
        Some(match binding {
            Some(_) => installed.reference.clone(),
            None => installed.reference.without_binding(),
        })
    }

    /// The installed assessment query a draft names, read for the resource
    /// and applying always until the draft retargets it.
    pub fn draft_assessment(&self, identifier: &str) -> Option<ApplicationWorkflowAssessmentRef> {
        self.assessments
            .iter()
            .find(|assessment| assessment.query_identifier == identifier)
            .map(|assessment| assessment.reference.clone())
    }

    pub fn draft_condition(&self, identifier: &str) -> Option<ApplicationWorkflowConditionRef> {
        self.conditions
            .iter()
            .find(|condition| condition.query_identifier == identifier)
            .map(|condition| condition.reference.clone())
    }

    pub fn draft_approval(&self, identifier: &str) -> Option<ApplicationWorkflowApprovalRef> {
        self.approvals
            .iter()
            .find(|approval| approval.binding.identifier == identifier)
            .map(|approval| approval.reference.clone())
    }
}
