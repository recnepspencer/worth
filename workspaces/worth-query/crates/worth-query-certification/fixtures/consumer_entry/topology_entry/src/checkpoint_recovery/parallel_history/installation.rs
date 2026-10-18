//! Fresh authentic application worlds and credentials, separate from the reference.
use super::{application::*, expected_history::World};
use crate::principal::*;
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::{
    admission::authenticated_principal as authentication,
    application_installation,
    declaration::authentication::{
        WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
    },
    primary_graph::*,
};
pub(super) type Application =
    application_installation::WorthQueryProgramApplicationRuntime<Schema, Program>;
pub(super) fn install(world: &World) -> Application {
    let program = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .validated_program()
        .unwrap();
    let declaration = Schema::declaration().unwrap();
    let limits = super::super::support::limits(
        32,
        super::super::support::invalidation(128 * 1024 * 1024, Some(1_000_000), 32),
    );
    application_installation::in_memory_program(
        program,
        declaration,
        ((),),
        limits,
        |_phase, graph, installed| {
            let principal = installed
                .principal_binding(ConsumerPrincipalBinding::reference::<Schema>())
                .unwrap();
            graph.bind_principal(
                &principal,
                WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                1,
                external_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            for member in &world.members {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Node::reference::<Schema>(),
                        WorthQueryApplicationEntityKey::new(format!("member-{}", member.key))
                            .unwrap(),
                    )
                    .field(NodeKey::reference(), member.key)
                    .field(Value::reference(), 0)
                    .field(Applied::reference(), 0),
                )?;
            }
            Ok(())
        },
    )
    .unwrap()
}
fn external_identity() -> WorthQueryExternalPrincipalIdentity {
    super::super::support::authentication_fixture::FixtureAuthenticationRealm::NeutralHistory
        .external_identity()
}

pub(super) fn authenticate<Program>(
    application: &application_installation::WorthQueryProgramApplicationRuntime<Schema, Program>,
) -> (
    authentication::WorthQueryRequestScope,
    authentication::WorthQueryAuthenticatedExternalPrincipal<Schema>,
)
where
    Program: ApplicationProgramDefinition<Schema>,
{
    use super::super::support::authentication_fixture::{
        authenticate_in_realm, FixtureAuthenticationRealm,
    };
    authenticate_in_realm(application, FixtureAuthenticationRealm::NeutralHistory)
}
