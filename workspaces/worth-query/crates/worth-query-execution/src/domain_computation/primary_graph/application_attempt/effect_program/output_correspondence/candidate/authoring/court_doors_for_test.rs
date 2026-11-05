//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::model::WorthQueryApplicationEffectEntity;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::role_use::OutputRoleUse;

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    #[doc(hidden)]
    pub fn bind_output_for_test<Entity>(
        &mut self,
        role: OutputRoleUse,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Self::bind_output::<Entity>(self, role, target)
    }
}
