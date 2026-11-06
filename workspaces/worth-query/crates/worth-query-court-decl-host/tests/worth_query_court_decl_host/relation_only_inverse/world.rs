//! One installed program world with a seeded scope node and the operator.
use super::*;
pub struct Program;
pub struct Feature;
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "relation-only-inverse.feature.v1";
}
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("relation-only-inverse.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Schema, Feature>()
            .native_action::<SpawnBinding>()
            .finish_action()
            .native_action::<DespawnBinding>()
            .finish_action()
            .finish()]
    }
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<SpawnBinding, _>(handlers::Handler)?;
        setup.handler::<DespawnBinding, _>(handlers::Handler)
    }
}
pub type Host = WorthQueryProgramApplicationRuntime<Schema, Program>;
pub fn install() -> Host {
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, Program>::begin()
            .validated_program()
            .unwrap(),
        Schema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("919b0068-0000-0000-0000-000000000001").unwrap(),
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        |_, graph, installed| {
            graph.bind_principal(
                &installed.principal_binding(Actor::reference()).unwrap(),
                WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
                1,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Node::reference(),
                    WorthQueryApplicationEntityKey::new("scope").unwrap(),
                )
                .field(Id::at("Id"), SCOPE),
            )?;
            Ok(())
        },
    )
    .unwrap()
}
