//! Ordinary contribution authoring/install, reusing the existing catalog and provider.
use super::{
    authoring, generated, geometry,
    handler::{DecisionRead, Handler},
    mutation, prefix, protected, query, rule,
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
        geometry::generated::declare(generated::declare(super::schema::declare(builder)))
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
    geometry: geometry::Configuration,
}
impl WorthQueryApplicationContribution<Catalog> for Contribution {
    type Configuration = Configuration;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<Catalog>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        generated::contracts(contracts)?;
        geometry::generated::contracts(contracts)
    }
    fn configure(
        configuration: Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        geometry::configure(configuration.geometry, setup)?;
        generated::configure(setup)?;
        geometry::generated::configure(setup)?;
        #[cfg(test)]
        let omit_refinement = configuration
            .prefix
            .omit_refinement
            .load(std::sync::atomic::Ordering::SeqCst);
        #[cfg(not(test))]
        let omit_refinement = false;
        let registration = setup
            .managed_index_provider::<prefix::PrefixIndex, _>(prefix::Provider(
                configuration.prefix,
            ))
            .with_predicate_routing();
        let registration = if omit_refinement {
            registration
        } else {
            registration.with_refinement()
        };
        registration.install()?;
        setup.handler::<mutation::Binding, _>(Handler(configuration.decision_read))?;
        setup.handler::<super::refinement::mutation::Binding, _>(mutation::Handler)?;
        setup.handler::<authoring::Binding, _>(authoring::Handler)?;
        setup.handler::<protected::status::Binding, _>(protected::status::Handler)?;
        setup.invariant(
            super::refinement::rule::ExactName::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            super::refinement::rule::resolve,
        )?;
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
    type Outputs = ApplicationProgramOutputs<(
        generated::program::Root,
        (
            geometry::generated::program::Root,
            generated::indexed::program::Root,
        ),
    )>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationSharedRuleRef<Catalog, rule::ReservedName>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleList<
            ApplicationRuleAt<
                ApplicationSharedRuleRef<Catalog, geometry::rule::ProbeContact>,
                ApplicationCommitBoundary,
            >,
            ApplicationRuleList<
                ApplicationRuleAt<
                    ApplicationSharedRuleRef<Catalog, super::refinement::rule::ExactName>,
                    ApplicationCommitBoundary,
                >,
                ApplicationRuleLeaf,
            >,
        >,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("managed-catalog.maintained-program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<Catalog, Feature>()
                .provides::<generated::indexed::program::ItemsOutput>()
                .action::<generated::indexed::request::Binding, generated::indexed::program::Delivery>()
                .finish_action()
                .provides::<generated::program::DirectoryOutput>()
                .provides::<geometry::generated::program::DirectoryOutput>()
                .action::<geometry::generated::request::Binding, geometry::generated::program::Delivery>()
                .finish_action()
                .action::<generated::source::operation::Binding, generated::program::Delivery>()
                .finish_action()
                .native_action::<mutation::Binding>()
                .finish_action()
                .native_action::<super::refinement::mutation::Binding>()
                .finish_action()
                .native_action::<protected::status::Binding>()
                .finish_action()
                .operation::<protected::access::InspectBoxes>()
                .native_action::<authoring::Binding>()
                .finish_action()
                .native_action::<geometry::authoring::Binding>()
                .finish_action()
                .native_action::<geometry::refresh::Binding>()
                .finish_action()
                .finish(),
            generated::program::feature(),
            generated::indexed::program::feature(),
            geometry::generated::program::feature(),
        ]
    }
}
pub(super) type Host = WorthQueryProgramApplicationRuntime<Catalog, Program>;
pub(super) fn install(prefix: Arc<prefix::Calls>, rules: Arc<rule::Calls>) -> Host {
    install_decision_read(prefix, rules, DecisionRead::Declared)
}
#[cfg(test)]
pub(super) fn install_watch(prefix: Arc<prefix::Calls>, rules: Arc<rule::Calls>) -> Host {
    let resources = super::super::super::resources::application_limits_with_observations(
        super::super::super::resources::EXECUTION_POLICY,
        320,
    );
    install_configured_with_limits(
        prefix,
        rules,
        DecisionRead::Declared,
        geometry::Configuration::default(),
        resources,
    )
}
pub(super) fn install_decision_read(
    prefix: Arc<prefix::Calls>,
    rules: Arc<rule::Calls>,
    decision_read: DecisionRead,
) -> Host {
    install_configured(
        prefix,
        rules,
        decision_read,
        geometry::Configuration::default(),
    )
}
pub(super) fn install_geometry(
    prefix: Arc<prefix::Calls>,
    rules: Arc<rule::Calls>,
    geometry: geometry::Configuration,
) -> Host {
    install_configured(prefix, rules, DecisionRead::Declared, geometry)
}
fn install_configured(
    prefix: Arc<prefix::Calls>,
    rules: Arc<rule::Calls>,
    decision_read: DecisionRead,
    geometry: geometry::Configuration,
) -> Host {
    install_configured_with_limits(
        prefix,
        rules,
        decision_read,
        geometry,
        super::super::super::resources::application_limits(),
    )
}

fn install_configured_with_limits(
    prefix: Arc<prefix::Calls>,
    rules: Arc<rule::Calls>,
    decision_read: DecisionRead,
    geometry: geometry::Configuration,
    limits: WorthQueryInMemoryApplicationLimits,
) -> Host {
    let world = try_install_root(prefix, Arc::clone(&rules), decision_read, geometry, limits)
        .expect("the genuine maintained catalog installs before public use");
    authoring::populate(&world, &rules);
    rules.records.lock().unwrap().clear();
    world
}
#[cfg(test)]
pub(super) fn install_growth(
    prefix: Arc<prefix::Calls>,
    rules: Arc<rule::Calls>,
    geometry: geometry::Configuration,
) -> Host {
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            None,
            Some(20_000_000_000),
        ),
    );
    install_configured_with_limits(
        prefix,
        rules,
        DecisionRead::Declared,
        geometry,
        super::super::super::resources::application_limits_with_policy(policy),
    )
}

fn try_install_root(
    prefix: Arc<prefix::Calls>,
    rules: Arc<rule::Calls>,
    decision_read: DecisionRead,
    geometry: geometry::Configuration,
    limits: WorthQueryInMemoryApplicationLimits,
) -> Result<Host, WorthQueryInMemoryApplicationDenial> {
    let declaration = Catalog::declaration().unwrap();
    in_memory_program(
        ApplicationProgramAuthoring::<Catalog, Program>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        declaration,
        (Configuration {
            prefix,
            rules,
            decision_read,
            geometry,
        },),
        ApplicationBuildIdentity::parse("19191919-1919-4919-8919-191919191919").unwrap(),
        limits,
        |_, graph, installed| {
            graph.bind_principal(&installed.principal_binding(CatalogPrincipal::reference()).unwrap(),
                WorthQueryApplicationPrincipalKey::new("catalog-reader").unwrap(), 1u64,
                super::super::super::authentication::external_identity(),
                worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled)?;
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Directory::reference(),
                    WorthQueryApplicationEntityKey::new("catalog").unwrap(),
                )
                .field(DirectoryIdentity::reference(), "catalog".to_owned())
                .field(DirectoryNote::reference(), String::new()),
            )?;
            protected::populate(graph)?;
            Ok(())
        },
    )
}

#[cfg(test)]
pub(super) fn try_install_unpopulated(
    prefix: Arc<prefix::Calls>,
    rules: Arc<rule::Calls>,
) -> Result<Host, WorthQueryInMemoryApplicationDenial> {
    try_install_root(
        prefix,
        rules,
        DecisionRead::Declared,
        geometry::Configuration::default(),
        super::super::super::resources::application_limits(),
    )
}
