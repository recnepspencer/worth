//! Ordinary contribution authoring/install, reusing the existing catalog and provider.
use super::{
    authoring, correction, generated, geometry,
    handler::{DecisionRead, Handler},
    mutation, prefix, protected, query, rule,
    schema::*,
    source_authorization,
};
use crate::application_invariant_acceptance::managed_index::prefix_observation;
use std::sync::Arc;
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_host::facade::{
    application_contribution::*, application_installation::*, primary_graph::*,
};

mod catalog;
pub use catalog::{Configuration, Contribution};
pub(super) use catalog::{Feature, Program};

pub(super) type Host = WorthQueryProgramApplicationRuntime<Catalog, Program>;
pub(super) fn install(prefix: Arc<prefix_observation::Calls>, rules: Arc<rule::Calls>) -> Host {
    install_decision_read(prefix, rules, DecisionRead::Declared)
}
#[cfg(test)]
pub(super) fn install_watch(
    prefix: Arc<prefix_observation::Calls>,
    rules: Arc<rule::Calls>,
) -> Host {
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
        WorthQueryAdmissionClock::system(),
    )
}
pub(super) fn install_decision_read(
    prefix: Arc<prefix_observation::Calls>,
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
    prefix: Arc<prefix_observation::Calls>,
    rules: Arc<rule::Calls>,
    geometry: geometry::Configuration,
) -> Host {
    install_configured(prefix, rules, DecisionRead::Declared, geometry)
}
fn install_configured(
    prefix: Arc<prefix_observation::Calls>,
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
        WorthQueryAdmissionClock::system(),
    )
}

fn install_configured_with_limits(
    prefix: Arc<prefix_observation::Calls>,
    rules: Arc<rule::Calls>,
    decision_read: DecisionRead,
    geometry: geometry::Configuration,
    limits: WorthQueryApplicationLimits,
    clock: WorthQueryAdmissionClock,
) -> Host {
    let world = try_install_root(
        prefix,
        Arc::clone(&rules),
        decision_read,
        geometry,
        limits,
        clock.clone(),
    )
    .expect("the genuine maintained catalog installs before public use");
    authoring::populate(&world, &rules, clock);
    rules.records.lock().unwrap().clear();
    world
}
#[cfg(test)]
pub(super) fn install_growth(
    prefix: Arc<prefix_observation::Calls>,
    rules: Arc<rule::Calls>,
    geometry: geometry::Configuration,
    clock: worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock,
) -> Host {
    let policy = super::super::super::resources::POPULATION_BOOTSTRAP_POLICY;
    install_configured_with_limits(
        prefix,
        rules,
        DecisionRead::Declared,
        geometry,
        super::super::super::resources::application_limits_with_clock(policy, clock.clone()),
        clock,
    )
}

fn try_install_root(
    prefix: Arc<prefix_observation::Calls>,
    rules: Arc<rule::Calls>,
    decision_read: DecisionRead,
    geometry: geometry::Configuration,
    limits: WorthQueryApplicationLimits,
    clock: WorthQueryAdmissionClock,
) -> Result<Host, WorthQueryApplicationPreparationDenial> {
    try_install_root_with_sources(
        prefix,
        rules,
        decision_read,
        geometry,
        source_authorization::Configuration::default(),
        limits,
        clock,
    )
}
#[cfg(test)]
pub(super) fn install_source_authorization(sources: source_authorization::Configuration) -> Host {
    let prefix = Arc::new(prefix_observation::Calls::default());
    let rules = Arc::new(rule::Calls::new(Arc::clone(&prefix)));
    let world = try_install_root_with_sources(
        prefix,
        Arc::clone(&rules),
        DecisionRead::Declared,
        geometry::Configuration::default(),
        sources,
        super::super::super::resources::application_limits(),
        WorthQueryAdmissionClock::system(),
    )
    .unwrap();
    authoring::populate(&world, &rules, WorthQueryAdmissionClock::system());
    rules.records.lock().unwrap().clear();
    world
}
fn try_install_root_with_sources(
    prefix: Arc<prefix_observation::Calls>,
    rules: Arc<rule::Calls>,
    decision_read: DecisionRead,
    geometry: geometry::Configuration,
    sources: source_authorization::Configuration,
    limits: WorthQueryApplicationLimits,
    clock: WorthQueryAdmissionClock,
) -> Result<Host, WorthQueryApplicationPreparationDenial> {
    let declaration = Catalog::declaration().unwrap();
    in_memory_program_with_authorization_time_source(
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
        super::super::super::resources::RuntimeTime(clock),
    )
}

#[cfg(test)]
pub(super) fn try_install_unpopulated(
    prefix: Arc<prefix_observation::Calls>,
    rules: Arc<rule::Calls>,
) -> Result<Host, WorthQueryApplicationPreparationDenial> {
    try_install_root(
        prefix,
        rules,
        DecisionRead::Declared,
        geometry::Configuration::default(),
        super::super::super::resources::application_limits(),
        WorthQueryAdmissionClock::system(),
    )
}

// This registered construction owner also installs the explicit nested text capability.
pub(super) type NestedHost = WorthQueryProgramApplicationRuntime<
    super::nested::schema::Catalog,
    super::nested::composition::Program,
>;
pub(super) fn install_nested(
    provider: Arc<prefix_observation::Calls>,
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

// This registered construction owner also installs the explicit nested text capability.
pub(super) type EventHost = WorthQueryProgramApplicationRuntime<
    super::events::schema::Catalog,
    super::events::composition::Program,
>;
pub(super) fn install_events(
    provider: Arc<prefix_observation::Calls>,
    records: Arc<super::events::rule::Calls>,
) -> EventHost {
    use super::events::{
        composition::{Configuration as EventConfiguration, Program as EventProgram},
        schema::*,
    };
    let declaration = Catalog::declaration().unwrap();
    in_memory_program(ApplicationProgramAuthoring::<Catalog,EventProgram>::begin().with_schema(&declaration).validated_program().unwrap(),declaration,(EventConfiguration {provider,records},),
        ApplicationBuildIdentity::parse("19191919-1919-4919-8919-191919191927").unwrap(),
        super::super::super::resources::application_limits_with_observations(super::super::super::resources::EXECUTION_POLICY,320),
        |_,graph,installed| {
            graph.bind_principal(&installed.principal_binding(CatalogPrincipal::reference()).unwrap(),WorthQueryApplicationPrincipalKey::new("event-reader").unwrap(),1u64,
                super::super::super::authentication::external_identity(),worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled)?;
            graph.bind_entity(WorthQueryApplicationEntitySeed::new(Directory::reference(),WorthQueryApplicationEntityKey::new("catalog").unwrap()).field(DirectoryIdentity::reference(),"catalog".to_owned()))?;
            Ok(())
        }).expect("the declared event capability installs through the ordinary host")
}
