use super::*;
use crate::domain_computation::primary_graph::application_attempt::effect_program::{
    retained_representation, WorthQueryApplicationEffectProgramBuilder,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn prepare_output_role_for_test<
        Binding,
        Entity,
        Action,
    >(
        &mut self,
        role: WorthQueryApplicationOutputRole<Binding, Entity, Action>,
        entity_name: &'static str,
    ) where
        Binding: 'static,
        Action: action::Sealed,
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
        let (binding_type, prepared, retained_representation_bytes) =
            self.output_correspondence
                .prepare_contract::<Schema, Binding>()?;
        self.charge_candidate_representation_only(retained_representation_bytes)?;
        self.output_correspondence.binding_type = Some(binding_type);
        self.output_correspondence.expected_roles.extend(prepared);
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn bind_output<Binding, Entity, Action>(
        &mut self,
        role: WorthQueryApplicationOutputRole<Binding, Entity, Action>,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Binding: 'static,
        Action: action::Sealed,
    {
        validate_role_name(role.name())?;
        let retained_representation_bytes =
            retained_representation::output_binding(role.name(), &target.entity, &target.reference)
                .ok_or_else(|| {
                    denial(
                        WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded,
                        role.name(),
                    )
                })?;
        self.output_correspondence
            .validate_binding(role, target, &self.program)?;
        self.charge_candidate_representation_only(retained_representation_bytes)?;
        self.output_correspondence.insert_binding(role, target);
        Ok(())
    }
}
