//! Select the real candidate's existing Scope/Preserve output, never a caller ID.
use super::{
    denial, ApplicationMutationOutputRoleCardinality, EntityReference, OutputRoleUse, TypeId,
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationOutputCorrespondenceCandidate, WorthQueryApplicationOutputPosture,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, WorthQueryApplicationOutputRole, WorthQueryPreserveOutput,
};
use worth_query_installation::facade::ApplicationSchema;
impl WorthQueryApplicationOutputCorrespondenceCandidate {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn matches_predecessor_output(
        &self,
        matching: &worth_query_declaration::facade::application_schema::ApplicationOperationBodySuccessionOutputMatch,
        target: worth_relational::facade::identity::EntityId,
    ) -> bool {
        let Some(binding) = self.roles.get(matching.selected_role()) else {
            return false;
        };
        let Some(expected) = self.expected_roles.get(matching.selected_role()) else {
            return false;
        };
        binding.posture == WorthQueryApplicationOutputPosture::Preserve
            && expected.cardinality == ApplicationMutationOutputRoleCardinality::ExactlyOne
            && binding.entity_name == matching.producer_role().entity
            && binding.entity == EntityReference::Existing(target)
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) fn succession_subject<
        Schema,
        Binding,
        Role,
    >(
        &self,
    ) -> Result<worth_relational::facade::identity::EntityId, WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryPreserveOutput,
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
            || binding.posture != WorthQueryApplicationOutputPosture::Preserve
            || binding.entity_name != role.entity_name
            || binding.entity_type != role.entity_type
        {
            return Err(denied());
        }
        match binding.entity {
            EntityReference::Existing(entity) => Ok(entity),
            EntityReference::Created(_) => Err(denied()),
        }
    }
}
