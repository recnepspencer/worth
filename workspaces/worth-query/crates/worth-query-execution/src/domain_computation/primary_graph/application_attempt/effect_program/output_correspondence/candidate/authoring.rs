use super::*;
use crate::domain_computation::primary_graph::application_attempt::effect_program::{
    retained_representation, WorthQueryApplicationEffectProgramBuilder,
};
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    /// Expect the roles and families `Contract` declares, in place of the
    /// operation's own output contract.
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn prepare_output_contract_for_test<Contract>(
        &mut self,
    ) where
        Schema: ApplicationSchema,
        Contract: worth_query_declaration::facade::application_operation::ApplicationMutationOutputContract<Schema>,
    {
        self.output_correspondence
            .prepare_test_contract::<Schema, Contract>();
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_output_contract<Binding>(
        &mut self,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let (prepared, families, retained_representation_bytes) =
            self.output_correspondence
                .prepare_contract::<Schema, Binding::Output>()?;
        self.charge_candidate_representation_only(retained_representation_bytes)?;
        self.output_correspondence.binding_type = Some(TypeId::of::<Binding>());
        self.output_correspondence.contract_type = Some(TypeId::of::<Binding::Output>());
        self.output_correspondence.expected_roles.extend(prepared);
        self.output_correspondence
            .expected_families
            .extend(families);
        Ok(())
    }

    /// Bind `target` under `role`. Public writers build `role` from a
    /// declaration marker, which checks it against the contract at compile
    /// time; the runtime checks here guard the erased use.
    pub(in crate::domain_computation::primary_graph) fn bind_output<Entity>(
        &mut self,
        role: OutputRoleUse,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        validate_role_name(&role.name)?;
        let retained_representation_bytes =
            retained_representation::output_binding(&role.name, &target.entity, &target.reference)
                .ok_or_else(|| {
                    denial(
                        WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded,
                        role.name.as_str(),
                    )
                })?;
        self.output_correspondence
            .validate_binding(&role, target, &self.program)?;
        self.charge_candidate_representation_only(retained_representation_bytes)?;
        self.output_correspondence.insert_binding(role, target);
        Ok(())
    }
}
