//! A genuine fixed refusing program with explicit public bootstrap calls.
use super::super::super::{authentication, resources};
use super::{directory, mutation, prefix, query, rule, schema::*};
use crate::application_invariant_acceptance::managed_index::prefix_observation;
use std::sync::Arc;
use worth_query_decl::facade::{
    application_program::*, application_query::*, application_schema::*,
};
use worth_query_host::facade::{
    application_contribution::*, application_installation::*, primary_graph::*,
};

pub struct Contribution;
impl ApplicationSchemaContribution<Catalog> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("managed-catalog.refusing-contribution.v1");
    fn register_members(
        schema: ApplicationSchemaDeclarationBuilder<Catalog>,
    ) -> ApplicationSchemaDeclarationBuilder<Catalog> {
        super::schema::declare(schema).invariant(rule::definition())
    }
}
pub struct Configuration {
    pub(super) provider: Arc<prefix_observation::Calls>,
    pub(super) rules: Arc<rule::Calls>,
}
impl WorthQueryApplicationContribution<Catalog> for Contribution {
    type Configuration = Configuration;
    fn configure(
        configuration: Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.managed_index::<prefix::PrefixIndex, _>(prefix::Provider(configuration.provider))?;
        setup.handler::<mutation::Binding, _>(mutation::Handler)?;
        setup.handler::<directory::Binding, _>(directory::Handler)?;
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
    const IDENTITY: &'static str = "managed-catalog.refusing-feature.v1";
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
        ApplicationProgramIdentity::new("managed-catalog.refusing-program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Catalog, Feature>()
            .native_action::<mutation::Binding>()
            .finish_action()
            .native_action::<directory::Binding>()
            .finish_action()
            .finish()]
    }
}
pub(super) type Host = WorthQueryProgramApplicationRuntime<Catalog, Program>;
pub(super) type Initialize = fn(
    &WorthQueryBootstrapAdvancementPhase<'_>,
    &mut WorthQueryPrimaryGraphBootstrap<Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>;
pub(super) fn initialize(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    graph: &mut WorthQueryPrimaryGraphBootstrap<Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    // compiled-example: begin
    graph.initialize_managed_index::<prefix::PrefixIndex>(phase)?;
    // compiled-example: end
    Ok(())
}
pub(super) fn omit(
    _: &WorthQueryBootstrapAdvancementPhase<'_>,
    _: &mut WorthQueryPrimaryGraphBootstrap<Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    Ok(())
}
pub(super) fn empty_attempt(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    graph: &mut WorthQueryPrimaryGraphBootstrap<Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    initialize(phase, graph)?;
    graph.commit_seed_batch(phase)
}
pub(super) fn unrelated_attempt(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    graph: &mut WorthQueryPrimaryGraphBootstrap<Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    initialize(phase, graph)?;
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Principal::reference(),
            WorthQueryApplicationEntityKey::new("unrelated-seed").unwrap(),
        )
        .field(PrincipalIdentityField::reference(), 999u64),
    )?;
    graph.commit_seed_batch(phase)?;
    // The following ordinary batch must not inherit the unused selection.
    Ok(())
}
struct SameMeaningDifferentBinding;
impl ApplicationManagedIndexBinding<Catalog> for SameMeaningDifferentBinding {
    type SourceBinding = query::SourceBinding;
    type ProbeBinding = query::ProbeBinding;
    type Entity = Item;
    const NAME: &'static str = prefix::PrefixIndex::NAME;
    const PROVIDER: &'static str = prefix::PrefixIndex::PROVIDER;
    const PROVIDER_VERSION: u32 = prefix::PrefixIndex::PROVIDER_VERSION;
}
pub(super) fn foreign_binding(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    graph: &mut WorthQueryPrimaryGraphBootstrap<Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.initialize_managed_index::<SameMeaningDifferentBinding>(phase)
}
pub(super) fn install(
    configuration: Configuration,
    initialize: Initialize,
    names: [&str; 3],
) -> Result<Host, WorthQueryInMemoryApplicationDenial> {
    let declaration = Catalog::declaration().unwrap();
    in_memory_program(
        ApplicationProgramAuthoring::<Catalog, Program>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        declaration,
        (configuration,),
        ApplicationBuildIdentity::parse("19191919-1919-4919-8919-191919191922").unwrap(),
        resources::application_limits(),
        |phase, graph, installed| {
            initialize(phase, graph)?;
            graph.bind_principal(&installed.principal_binding(CatalogPrincipal::reference()).unwrap(),
                WorthQueryApplicationPrincipalKey::new("reader").unwrap(), 1u64,
                authentication::external_identity(), worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled)?;
            for name in ["catalog", "A", "B"] {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Directory::reference(),
                        WorthQueryApplicationEntityKey::new(name).unwrap(),
                    )
                    .field(DirectoryIdentity::reference(), name.to_owned()),
                )?;
            }
            for (identity, name) in ["shared", "a-only", "b-only"].into_iter().zip(names) {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Item::reference(),
                        WorthQueryApplicationEntityKey::new(identity).unwrap(),
                    )
                    .field(ItemIdentity::reference(), identity.to_owned())
                    .field(ItemName::reference(), name.to_owned())
                    .field(ItemEnabled::reference(), true),
                )?;
            }
            for (directory, item) in [
                ("A", "shared"),
                ("B", "shared"),
                ("A", "a-only"),
                ("B", "b-only"),
            ] {
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    Contains::reference(),
                    format!("{directory}:{item}"),
                    WorthQueryApplicationEntityKey::new(directory).unwrap(),
                    WorthQueryApplicationEntityKey::new(item).unwrap(),
                ))?;
            }
            Ok(())
        },
    )
}
