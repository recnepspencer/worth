//! Fresh authentic application worlds and credentials, separate from the reference.
use super::{application::*, control::Control};
use crate::principal::*;
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
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
pub(super) fn install(control: std::sync::Arc<Control>) -> Application {
    let world = &control.world;
    let program = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .validated_program()
        .unwrap();
    let declaration = Schema::declaration().unwrap();
    let limits = crate::checkpoint_recovery::support::limits(
        32,
        crate::checkpoint_recovery::support::invalidation(),
    );
    application_installation::in_memory_program(
        program,
        declaration,
        (control.clone(),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
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
            for key in world.members.iter().map(|member| member.key).chain([100]) {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Node::reference::<Schema>(),
                        WorthQueryApplicationEntityKey::new(format!("member-{}", key)).unwrap(),
                    )
                    .field(NodeKey::reference(), format!("member-{:03}", key))
                    .field(Revision::reference(), 0)
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
    crate::checkpoint_recovery::support::authentication_fixture::FixtureAuthenticationRealm::NeutralHistory
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
    use crate::checkpoint_recovery::support::authentication_fixture::{
        authenticate_in_realm, FixtureAuthenticationRealm,
    };
    authenticate_in_realm(application, FixtureAuthenticationRealm::NeutralHistory)
}
