use super::*;
use crate::domain_computation::primary_graph::application_attempt::effect_program::{
    retained_representation, WorthQueryApplicationEffectProgramBuilder,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationFixedOutputRole;
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn prepare_output_role_for_test<Role>(
        &mut self,
        role: &Role,
        entity_name: &'static str,
    ) where
        Role: WorthQueryApplicationFixedOutputRole,
    {
        self.output_correspondence
            .prepare_test_role(role, entity_name);
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_output_contract<Binding>(
        &mut self,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let (binding_type, prepared, families, retained_representation_bytes) = self
            .output_correspondence
            .prepare_contract::<Schema, Binding>()?;
        self.charge_candidate_representation_only(retained_representation_bytes)?;
        self.output_correspondence.binding_type = Some(binding_type);
        self.output_correspondence.expected_roles.extend(prepared);
        self.output_correspondence
            .expected_families
            .extend(families);
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn bind_output<Role>(
        &mut self,
        role: Role,
        target: &WorthQueryApplicationEffectEntity<Schema, Role::Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Role: WorthQueryApplicationFixedOutputRole,
    {
        validate_role_name(role.name(INTERNAL))?;
        let retained_representation_bytes = retained_representation::output_binding(
            role.name(INTERNAL),
            &target.entity,
            &target.reference,
        )
        .ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded,
                role.name(INTERNAL),
            )
        })?;
        self.output_correspondence
            .validate_binding(&role, target, &self.program)?;
        self.charge_candidate_representation_only(retained_representation_bytes)?;
        self.output_correspondence.insert_binding(role, target);
        Ok(())
    }
}
