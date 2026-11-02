//! Ordinary program installation; only the new capability opts into this Source.
use super::{authoring, history, mutation, plain, prefix, query, rule, schema::*};
use std::sync::Arc;
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::{application_contribution::*, primary_graph::*};
pub struct Contribution;
impl ApplicationSchemaContribution<Catalog> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("event-catalog.members.v1");
    fn register_members(
        schema: ApplicationSchemaDeclarationBuilder<Catalog>,
    ) -> ApplicationSchemaDeclarationBuilder<Catalog> {
        super::schema::declare(schema)
    }
}
pub struct Configuration {
    pub provider: Arc<prefix::Calls>,
    pub records: Arc<rule::Calls>,
}
impl WorthQueryApplicationContribution<Catalog> for Contribution {
    type Configuration = Configuration;
    fn configure(
        config: Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup
            .managed_index_provider::<prefix::PrefixIndex, _>(prefix::Provider(config.provider))
            .with_predicate_routing()
            .install()?;
        setup.handler::<authoring::Binding, _>(authoring::Handler)?;
        setup.handler::<mutation::Binding, _>(mutation::Handler)?;
        setup.handler::<history::Binding, _>(history::Handler)?;
        setup.handler::<plain::Binding, _>(plain::Handler)?;
        setup.invariant(
            rule::CompleteNestedSource::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            move |resolver| rule::resolve(resolver, Arc::clone(&config.records)),
        )
    }
}
pub(in super::super) struct Program;
pub(in super::super) struct Feature;
impl ApplicationFeature<Catalog> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "event-catalog.feature.v1";
}
impl ApplicationProgramDefinition<Catalog> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationSharedRuleRef<Catalog, rule::CompleteNestedSource>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("event-catalog.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Catalog, Feature>()
            .native_action::<authoring::Binding>()
            .finish_action()
            .native_action::<plain::Binding>()
            .finish_action()
            .native_action::<history::Binding>()
            .finish_action()
            .native_action::<mutation::Binding>()
            .finish_action()
            .finish()]
    }
}
