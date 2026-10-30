//! A single declared schema/program installs every ordinary application handler.
use super::{
    access, attempts, authentication::Actor, creation, mutation_access, recording, resources,
    retention, schema::*,
};
use worth_query_decl::facade::application_schema::ApplicationInvariantMarkerIdentity;
use worth_query_decl::facade::{application_program::*, authentication::*};
use worth_query_host::facade::{
    application_contribution::*, application_installation::*, primary_graph::*,
};

pub struct Program;
pub struct Feature;
pub struct RetentionDelivery;
impl ApplicationActionDelivery<Schema, retention::Binding> for RetentionDelivery {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "proposal-application.retention.delivery.v1";
}
pub type RetentionAction = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    Feature,
    retention::Binding,
    RetentionDelivery,
>;
impl ApplicationFeature<Schema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "proposal-application.feature.v1";
}
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationLocalRuleRef<Schema, Feature, retention::rule::MaximumRetention>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("proposal-application.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Schema, Feature>()
            .native_action::<creation::Binding>()
            .finish_action()
            .native_action::<recording::Binding>()
            .finish_action()
            .native_action::<attempts::claim::Binding>()
            .finish_action()
            .native_action::<attempts::recording::Binding>()
            .finish_action()
            .native_action::<access::status::Binding>()
            .finish_action()
            .native_action::<mutation_access::status::Binding>()
            .finish_action()
            .action::<retention::Binding, RetentionDelivery>()
            .finish_action()
            .operation::<access::InspectDomain>()
            .finish()]
    }
}
impl WorthQueryApplicationContribution<Schema> for Contribution {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<creation::Binding, _>(creation::Handler)?;
        setup.handler::<recording::Binding, _>(recording::Handler)?;
        setup.handler::<attempts::claim::Binding, _>(attempts::claim::Handler)?;
        setup.handler::<attempts::recording::Binding, _>(attempts::recording::Handler)?;
        setup.handler::<access::status::Binding, _>(access::status::Handler)?;
        setup.handler::<mutation_access::status::Binding, _>(mutation_access::status::Handler)?;
        setup.invariant(retention::rule::MaximumRetention::reference(),
            worth_query_decl::facade::application_schema::ApplicationInvariantExecutionPoint::CommitBoundary,retention::rule::resolve)?;
        setup.handler::<retention::Binding, _>(retention::Handler)
    }
}
pub type Host = WorthQueryProgramApplicationRuntime<Schema, Program>;
pub fn install() -> Host {
    let declaration = Schema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    in_memory_program(
        program,
        declaration,
        ((),),
        ApplicationBuildIdentity::parse("17171717-1717-1717-1717-171717171717").unwrap(),
        resources::limits(),
        |_, graph, installed| {
            for (actor, identity) in [(Actor::Author, 1u64), (Actor::Reviewer, 2u64)] {
                graph.bind_principal(
                    &installed
                        .principal_binding(PrincipalBinding::reference())
                        .unwrap(),
                    WorthQueryApplicationPrincipalKey::new(actor.subject()).unwrap(),
                    identity,
                    actor.identity(),
                    WorthQueryPrincipalMappingStatus::Enabled,
                )?;
            }
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Registry::reference(),
                    WorthQueryApplicationEntityKey::new("models").unwrap(),
                )
                .field(RegistryName::reference(), "models".to_owned()),
            )?;
            Ok(())
        },
    )
    .expect("the complete public model/proposal declaration installs")
}
