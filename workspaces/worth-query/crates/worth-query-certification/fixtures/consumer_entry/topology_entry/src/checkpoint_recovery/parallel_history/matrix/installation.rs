//! Fresh authentic application worlds and credentials, separate from the reference.
use super::{application::*, control::Control};
use crate::principal::*;
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::application_installation::{
    ApplicationBuildIdentity, WorthQueryApplicationLimits,
};
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
pub(super) fn install_with_limits(
    control: std::sync::Arc<Control>,
    limits: WorthQueryApplicationLimits,
) -> Application {
    let world = &control.world;
    let program = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .validated_program()
        .unwrap();
    let declaration =
        <Schema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration()
            .unwrap();
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
                    .field(NodeKey::at("NodeKey"), format!("member-{:03}", key))
                    .field(Revision::at("Revision"), 0)
                    .field(Value::at("Value"), 0)
                    .field(Applied::at("Applied"), 0),
                )?;
            }
            for (child, parents) in world
                .members
                .iter()
                .map(|member| (member.key, &member.upstream))
                .chain(std::iter::once((100, &world.requested)))
            {
                for parent in parents {
                    let parent = world.members[*parent].key;
                    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                        Prerequisite::reference::<Schema>(),
                        format!("member-{child}-requires-{parent}"),
                        WorthQueryApplicationEntityKey::new(format!("member-{child}")).unwrap(),
                        WorthQueryApplicationEntityKey::new(format!("member-{parent}")).unwrap(),
                    ))?;
                }
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
    authenticate_in_realm(
        application,
        FixtureAuthenticationRealm::NeutralHistory,
        authentication::WorthQueryAdmissionClock::system(),
    )
}
