//! A creation's workflow enrollment is authored meaning, never a start permit.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyCreationBuilder,
};
use crate::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationOutputRoleCardinality,
        WorthQueryApplicationDeclaredOutputRole, WorthQueryApplicationOutputRole,
        WorthQueryCreateOutput,
    },
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationSchema,
};
#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyEnrollment {
    workflow_spec: String,
    binding: String,
    role: String,
}

impl ApplicationOperationBodyEnrollment {
    /// Decoded meaning only; schema and vocabulary installation must readmit it.
    pub fn from_untrusted(workflow_spec: String, binding: String, role: String) -> Self {
        Self {
            workflow_spec,
            binding,
            role,
        }
    }
    pub fn workflow_spec(&self) -> &str {
        &self.workflow_spec
    }
    pub fn binding(&self) -> &str {
        &self.binding
    }
    pub fn role(&self) -> &str {
        &self.role
    }
    pub fn canonical_record(&self) -> String {
        use std::fmt::Write;
        let mut record = String::from("workflow-birth-v1;");
        for part in [&self.workflow_spec, &self.binding, &self.role] {
            write!(&mut record, "{}:{part};", part.len()).expect("String write");
        }
        record
    }
}

impl<Schema: ApplicationSchema, Operation, Input, Entity>
    ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    /// Declares that this exact created output is born enrolled in the named
    /// workflow. Ordinary creation cannot substitute for the native birth join.
    /// The published definition and fresh capability remain request-time inputs.
    pub fn enroll_in_workflow<Spec, Binding, Role>(mut self) -> Result<Self, Denial>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Entity = Entity,
            Action = WorthQueryCreateOutput,
        >,
    {
        if self.enrollment.is_some()
            || self.signing.is_some()
            || <Role as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR.cardinality()
                != ApplicationMutationOutputRoleCardinality::ExactlyOne
        {
            return Err(Denial::InvalidWorkflowEnrollment);
        }
        self = self.create_output::<Binding, Role>()?;
        self.enrollment = Some(ApplicationOperationBodyEnrollment {
            workflow_spec: Spec::IDENTITY.as_str().to_owned(),
            binding: Binding::IDENTITY.to_owned(),
            role: <Role as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR
                .name()
                .to_owned(),
        });
        Ok(self)
    }
}

pub(super) fn valid(body: &super::ApplicationOperationBodyProgram) -> bool {
    let Some(creation) = body.creation() else {
        return true;
    };
    let Some(enrollment) = creation.enrollment() else {
        return true;
    };
    if creation.signing().is_some()
        || creation.signing_lifecycle().is_some()
        || [
            enrollment.workflow_spec(),
            enrollment.binding(),
            enrollment.role(),
        ]
        .iter()
        .any(|value| value.is_empty() || value.trim() != *value)
    {
        return false;
    }
    body.outputs().iter().any(|output|
        output.binding == enrollment.binding() && output.role.name == enrollment.role()
        && output.role.entity == creation.entity()
        && output.role.posture == crate::application_operation::ApplicationMutationOutputPosture::Create
        && output.role.cardinality == ApplicationMutationOutputRoleCardinality::ExactlyOne
        && matches!(&output.target, super::ApplicationOperationBodyOutputTarget::Creation { key } if key == creation.key()))
}
