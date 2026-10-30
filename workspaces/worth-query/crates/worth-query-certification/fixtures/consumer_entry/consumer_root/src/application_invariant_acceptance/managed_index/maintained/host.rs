//! Ordinary contribution authoring/install, reusing the existing catalog and provider.
use super::{
    authoring,
    handler::{DecisionRead, Handler},
    mutation, prefix, query, rule,
    schema::*,
};
use std::sync::Arc;
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::{
    application_contribution::*, application_installation::*, primary_graph::*,
};

worth_query_decl::facade::worth_query_operation_reads!(mutation::RenameItem => [ItemEnabled]);

pub struct Contribution;
impl ApplicationSchemaContribution<Catalog> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("managed-catalog.maintained-contribution.v1");
    fn register_members(
        builder: ApplicationSchemaDeclarationBuilder<Catalog>,
    ) -> ApplicationSchemaDeclarationBuilder<Catalog> {
        // Declare this root's typed Source, fields, index and public bindings.
        // Its fixed contribution adds the rule and original decision reads.
        super::schema::declare(builder)
            .operation_managed_index_read::<query::PrefixBinding, _, _>(
                mutation::RenameItem::reference(),
            )
            .operation_read_field(mutation::RenameItem::reference(), ItemEnabled::reference())
            .invariant(rule::definition())
    }
}
pub struct Configuration {
    prefix: Arc<prefix::Calls>,
    rules: Arc<rule::Calls>,
    decision_read: DecisionRead,
}
impl WorthQueryApplicationContribution<Catalog> for Contribution {
    type Configuration = Configuration;
    fn configure(
        configuration: Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.managed_index::<prefix::PrefixIndex, _>(prefix::Provider(configuration.prefix))?;
        setup.handler::<mutation::Binding, _>(Handler(configuration.decision_read))?;
        setup.handler::<authoring::Binding, _>(authoring::Handler)?;
        setup.invariant(
            rule::ReservedName::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            move |resolver| rule::resolve(resolver, Arc::clone(&configuration.rules)),
        )
    }
}
pub(super) struct Program;
pub(super) struct Feature;
impl ApplicationFeature<Catalog> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "managed-catalog.maintained-feature.v1";
}
impl ApplicationProgramDefinition<Catalog> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationSharedRuleRef<Catalog, rule::ReservedName>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("managed-catalog.maintained-program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Catalog, Feature>()
            .native_action::<mutation::Binding>()
            .finish_action()
            .native_action::<authoring::Binding>()
            .finish_action()
            .finish()]
    }
}
pub(super) type Host = WorthQueryProgramApplicationRuntime<Catalog, Program>;
pub(super) fn install(prefix: Arc<prefix::Calls>, rules: Arc<rule::Calls>) -> Host {
    install_decision_read(prefix, rules, DecisionRead::Declared)
}
pub(super) fn install_decision_read(
    prefix: Arc<prefix::Calls>,
    rules: Arc<rule::Calls>,
    decision_read: DecisionRead,
) -> Host {
    let world = install_root(prefix, Arc::clone(&rules), decision_read);
    authoring::populate(&world, &rules);
    rules.records.lock().unwrap().clear();
    world
}
fn install_root(
    prefix: Arc<prefix::Calls>,
    rules: Arc<rule::Calls>,
    decision_read: DecisionRead,
) -> Host {
    let declaration = Catalog::declaration().unwrap();
    in_memory_program(
        ApplicationProgramAuthoring::<Catalog, Program>::begin().with_schema(&declaration).validated_program().unwrap(),
        declaration, (Configuration { prefix, rules, decision_read },),
        ApplicationBuildIdentity::parse("19191919-1919-4919-8919-191919191919").unwrap(),
        super::super::super::resources::application_limits(),
        |_, graph, installed| {
            graph.bind_principal(&installed.principal_binding(CatalogPrincipal::reference()).unwrap(),
                WorthQueryApplicationPrincipalKey::new("catalog-reader").unwrap(), 1u64,
                super::super::super::authentication::external_identity(),
                worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled)?;
            graph.bind_entity(WorthQueryApplicationEntitySeed::new(Directory::reference(),
                WorthQueryApplicationEntityKey::new("catalog").unwrap())
                .field(DirectoryIdentity::reference(), "catalog".to_owned()))?;
            Ok(())
        },
    ).expect("the genuine maintained catalog installs before public use")
}
