//! Output metadata measurements before the binding's candidate exists.

use super::{candidate_representation::checked, DecisionReader};
use crate::domain_computation::primary_graph::application_attempt::{
    candidate_retained_representation as representation, OutputRoleUse,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
    WorthQueryApplicationEntityKey,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, WorthQueryApplicationOutputAction, WorthQueryApplicationOutputRole,
    WorthQueryApplicationOutputRoleFamily,
};
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    DecisionReader<'_, '_, '_, Schema, Binding>
{
    /// Declared role/family-name backing retained once when this binding begins.
    /// Add actual bound-role quotes separately, including for an empty contract.
    pub fn candidate_output_contract_bytes(&self) -> Result<usize, Denial> {
        checked(representation::output_contract_bytes::<
            Schema,
            Binding::Output,
        >())
    }

    /// One actual fixed output binding. `Some(key)` is the original key of a
    /// created target; `None` describes an existing target. No identity is minted.
    pub fn candidate_output_bytes<Role>(
        &self,
        created: Option<&WorthQueryApplicationEntityKey<Schema, Role::Entity>>,
    ) -> Result<usize, Denial>
    where
        Role: WorthQueryApplicationOutputRole<Schema = Schema, Contract = Binding::Output>,
    {
        let role = OutputRoleUse::fixed::<Role>();
        checked(representation::output_binding_bytes(
            &role.name,
            role.entity_name,
            created.map_or(0, |key| key.as_str().len()),
        ))
    }

    /// One actual member output binding, using the writer's declared-family and
    /// suffix validation. Contract prefix backing is counted separately once.
    pub fn candidate_output_member_bytes<Family, Action>(
        &self,
        suffix: &str,
        created: Option<&WorthQueryApplicationEntityKey<Schema, Family::Entity>>,
    ) -> Result<usize, Denial>
    where
        Family: WorthQueryApplicationOutputRoleFamily<Schema = Schema, Contract = Binding::Output>,
        Action: WorthQueryApplicationOutputAction,
    {
        let role = OutputRoleUse::member::<Family, Action>(suffix)
            .map_err(|_| Denial::new(Kind::InvalidOutputRole, suffix))?;
        checked(representation::output_binding_bytes(
            &role.name,
            role.entity_name,
            created.map_or(0, |key| key.as_str().len()),
        ))
    }
}
