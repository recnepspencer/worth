//! A completed chain may acquire a fresh instance through its own native succession owner.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyBuilder,
    ApplicationOperationBodyProgram,
};
use crate::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationOutputPosture as Posture,
        ApplicationMutationOutputRoleCardinality as Cardinality, ApplicationMutationScopeBinding,
        WorthQueryApplicationDeclaredOutputRole, WorthQueryApplicationOutputRole,
        WorthQueryPreserveOutput,
    },
    application_program::ApplicationWorkflowSpec,
    application_schema::ApplicationSchema,
};

/// Pure meaning. A predecessor reference or decoded attachment grants no succession authority.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodySuccession {
    workflow_spec: String,
    binding: String,
    role: String,
}
impl ApplicationOperationBodySuccession {
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
        let mut record = String::from("workflow-succession-v1;");
        for part in [&self.workflow_spec, &self.binding, &self.role] {
            write!(&mut record, "{}:{part};", part.len()).expect("String write");
        }
        record
    }
}
impl<Schema: ApplicationSchema, Operation, Input>
    ApplicationOperationBodyBuilder<Schema, Operation, Input>
{
    /// Co-publish this body with a fresh instance on its exact existing scope.
    /// Native admission additionally requires an actually completed, childless
    /// predecessor of this subject under a supported definition pin.
    ///
    /// ```compile_fail,E0271
    /// use worth_query_declaration::facade::{application_operation::*, application_schema::*, application_program::*};
    /// fn wrong_action<S, B, R, W>(body: ApplicationOperationBodyBuilder<S, B::Operation, B::Input>)
    /// where S: ApplicationSchema, B: ApplicationMutationBinding<S>, W: ApplicationWorkflowSpec<Schema=S>,
    /// B::ScopeBinding: ApplicationMutationScopeBinding<S, Scope=R::Entity>,
    /// R: WorthQueryApplicationOutputRole<Schema=S, Contract=B::Output, Action=WorthQueryCreateOutput> {
    ///     let _ = body.succeed_workflow::<W, B, R>();
    /// }
    /// ```
    pub fn succeed_workflow<Spec, Binding, Role>(mut self) -> Result<Self, Denial>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Binding::ScopeBinding: ApplicationMutationScopeBinding<Schema, Scope = Role::Entity>,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryPreserveOutput,
        >,
    {
        if self.succession.is_some()
            || <Role as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR.cardinality()
                != Cardinality::ExactlyOne
        {
            return Err(Denial::InvalidWorkflowSuccession);
        }
        self = self.preserve_scope_output::<Binding, Role>()?;
        self.succession = Some(ApplicationOperationBodySuccession::from_untrusted(
            Spec::IDENTITY.as_str().into(),
            Binding::IDENTITY.into(),
            Role::NAME.into(),
        ));
        Ok(self)
    }
}
pub(super) fn valid(body: &ApplicationOperationBodyProgram) -> bool {
    let Some(succession) = body.succession() else {
        return true;
    };
    if body.creation().is_some_and(|creation| {
        creation.enrollment().is_some()
            || creation.signing().is_some()
            || creation.signing_lifecycle().is_some()
    }) || [
        succession.workflow_spec(),
        succession.binding(),
        succession.role(),
    ]
    .iter()
    .any(|part| part.is_empty() || part.trim() != *part)
    {
        return false;
    }
    body.outputs().iter().any(|output| {
        output.binding == succession.binding()
            && output.role.name == succession.role()
            && Some(output.role.entity.as_str()) == body.scope_entity()
            && output.role.posture == Posture::Preserve
            && output.role.cardinality == Cardinality::ExactlyOne
            && output.target == super::ApplicationOperationBodyOutputTarget::Scope
    })
}

#[cfg(test)]
mod tests;
