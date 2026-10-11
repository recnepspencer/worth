//! Resolve real authentication and authorization before building the effect program.
use super::schema::*;
use super::world::World;
use crate::{Count, Parameter};
use worth_query_consumer_values::PositiveCount;
use worth_query_decl::facade::application_schema::TypedMutationPreconditions;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::primary_graph::*;

pub(super) fn prepare(
    world: &World,
    scope: &WorthQueryRequestScope,
) -> WorthQueryApplicationEffectProgram<LeaseSchema, SetCount, SetCountInput, Parameter> {
    world
        .application
        .with_host_advancement(|phase| {
            let selected = world
                .application
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap();
            let external = super::authentication::block_on(
                super::authentication::admit(world.application.installed_schema()).authenticate(
                    super::authentication::LocalCredential::issued_for_model_owner(),
                    scope,
                ),
            )
            .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let account = selected
                .resolve_entity(
                    &phase,
                    Count::reference(),
                    PositiveCount::new(7).unwrap(),
                    scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = world
                .application
                .installed_schema()
                .installed_operation(SetCount::reference())
                .unwrap();
            let admission = selected
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    TypedMutationPreconditions::new(),
                    scope,
                )
                .unwrap();
            let (_, projection, _) = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, projected| {
                        reader
                            .require_decision_field(projected, Count::reference())
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
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .begin_effect_program();
            let account = effects.existing_entity(&account).unwrap();
            effects
                .write_field(&account, Count::reference(), PositiveCount::new(8).unwrap())
                .unwrap();
            effects.finish().unwrap()
        })
        .unwrap()
}
