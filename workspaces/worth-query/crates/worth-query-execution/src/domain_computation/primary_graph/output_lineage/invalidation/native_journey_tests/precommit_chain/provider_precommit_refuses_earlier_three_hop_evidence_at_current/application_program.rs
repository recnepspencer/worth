//! An admitted operation prepared at the installed world's current basis.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    Account, IdentityExecutionSchema, TouchAccountInput,
};
pub(super) fn admit(
    world: &AuthorizationWorld,
    replacement: &str,
) -> crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    TouchAccountOperation,
    TouchAccountInput,
    Account,
> {
    world
        .application
        .with_host_advancement(|phase| {
            let request = live_scope();
            let external = world.authenticate("alice", Duration::from_secs(60), &request);
            let selected = world.selected_product(&(phase).test_host_request());
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let account = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "closed".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = world
                .application
                .installed_schema()
                .installed_operation(TouchAccountOperation::reference())
                .unwrap();
            let admission = selected
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap();
            let (_, projection, _) = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, account| {
                        reader
                            .require_decision_field(account, AccountStatus::reference())
                            .unwrap();
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts();
            let reads = world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let mut effects = reads
                .complete_projected_dependencies(
                    &phase,
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .begin_effect_program();
            let account = effects.existing_entity(&account).unwrap();
            effects
                .write_field(&account, AccountStatus::reference(), replacement.to_owned())
                .unwrap();
            effects.finish().unwrap()
        })
        .expect("the fixture invocation owns its installed host call")
}
