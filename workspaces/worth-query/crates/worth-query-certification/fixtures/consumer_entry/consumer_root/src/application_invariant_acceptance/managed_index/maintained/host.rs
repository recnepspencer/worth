//! Ordinary contribution authoring/install, reusing the existing catalog and provider.
use super::{
    authoring, correction, generated, geometry,
    handler::{DecisionRead, Handler},
    mutation, prefix, protected, query, rule,
    schema::*,
    source_authorization,
};
use std::sync::Arc;
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::{
    application_contribution::*, application_installation::*, primary_graph::*,
};

mod catalog;
pub use catalog::{Configuration, Contribution};
pub(super) use catalog::{Feature, Program};

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
    try_install_root_with_sources(
        prefix,
        rules,
        decision_read,
        geometry,
        source_authorization::Configuration::default(),
        limits,
    )
}
#[cfg(test)]
pub(super) fn install_source_authorization(sources: source_authorization::Configuration) -> Host {
    let prefix = Arc::new(prefix::Calls::default());
    let rules = Arc::new(rule::Calls::new(Arc::clone(&prefix)));
    let world = try_install_root_with_sources(
        prefix,
        Arc::clone(&rules),
        DecisionRead::Declared,
        geometry::Configuration::default(),
        sources,
        super::super::super::resources::application_limits(),
    )
    .unwrap();
    authoring::populate(&world, &rules);
    rules.records.lock().unwrap().clear();
    world
}
fn try_install_root_with_sources(
    prefix: Arc<prefix::Calls>,
    rules: Arc<rule::Calls>,
    decision_read: DecisionRead,
    geometry: geometry::Configuration,
    sources: source_authorization::Configuration,
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
            sources,
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

// This registered construction owner also installs the explicit nested text capability.
pub(super) type NestedHost = WorthQueryProgramApplicationRuntime<
    super::nested::schema::Catalog,
    super::nested::composition::Program,
>;
pub(super) fn install_nested(
    provider: Arc<super::nested::prefix::Calls>,
    records: Arc<super::nested::rule::Calls>,
) -> NestedHost {
    use super::nested::{
        composition::{Configuration as NestedConfiguration, Program as NestedProgram},
        schema::*,
    };
    let declaration = Catalog::declaration().unwrap();
    in_memory_program(ApplicationProgramAuthoring::<Catalog,NestedProgram>::begin().with_schema(&declaration).validated_program().unwrap(),declaration,(NestedConfiguration {provider,records},),
        ApplicationBuildIdentity::parse("19191919-1919-4919-8919-191919191926").unwrap(),
        super::super::super::resources::application_limits_with_observations(super::super::super::resources::EXECUTION_POLICY,320),
        |_,graph,installed| {
            graph.bind_principal(&installed.principal_binding(CatalogPrincipal::reference()).unwrap(),WorthQueryApplicationPrincipalKey::new("nested-reader").unwrap(),1u64,
                super::super::super::authentication::external_identity(),worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled)?;
            graph.bind_entity(WorthQueryApplicationEntitySeed::new(Directory::reference(),WorthQueryApplicationEntityKey::new("catalog").unwrap()).field(DirectoryIdentity::reference(),"catalog".to_owned()))?;
            Ok(())
        }).expect("the declared nested capability installs through the ordinary host")
}
