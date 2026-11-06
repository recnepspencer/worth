//! Select the actual candidate-owned Create target, never a reconstructed ID.
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, WorthQueryApplicationOutputRole, WorthQueryCreateOutput,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::transactions::CreatedEntityRef;
use {super::denial, super::ApplicationMutationOutputRoleCardinality, super::EntityReference, super::TypeId, crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial, super::WorthQueryApplicationOutputCorrespondenceCandidate, super::WorthQueryApplicationOutputPosture, crate::domain_computation::primary_graph::vocabulary::application_attempt::role_use::OutputRoleUse, crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind};

impl WorthQueryApplicationOutputCorrespondenceCandidate {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn birth_subject<
        Schema,
        Binding,
        Role,
    >(
        &self,
    ) -> Result<CreatedEntityRef, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryCreateOutput,
        >,
    {
        let role = OutputRoleUse::fixed::<Role>();
        let denied = || {
            denial(
                WorthQueryApplicationAttemptDenialKind::ForeignOutputRole,
                role.name.as_str(),
            )
        };
        if self.binding_type != Some(TypeId::of::<Binding>())
            || self.contract_type != Some(TypeId::of::<Binding::Output>())
        {
            return Err(denied());
        }
        let binding = self.roles.get(&role.name).ok_or_else(denied)?;
        if role.cardinality != ApplicationMutationOutputRoleCardinality::ExactlyOne
            || binding.posture != WorthQueryApplicationOutputPosture::Create
            || binding.entity_name != role.entity_name
            || binding.entity_type != role.entity_type
        {
            return Err(denied());
        }
        match &binding.entity {
            EntityReference::Created(created) => Ok(created.clone()),
            EntityReference::Existing(_) => Err(denied()),
        }
    }
}
