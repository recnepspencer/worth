//! Installed authorization-world fixture compiler and its population phases.

use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::publication::publish_authorization_world;
pub use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::reclaiming::install_reclaiming_authorization_world;

use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::preparation::prepare_authorization_world;

use crate::domain_computation::primary_graph::tests::fixture::account_seed::bind_account;
use crate::domain_computation::primary_graph::tests::fixture::account_seed::AccountSeedSpec;
use crate::domain_computation::primary_graph::tests::fixture::*;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationInvariantProjectionAuthority;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionInstallationAuthority;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphBootstrap;

#[derive(Clone, Copy)]
pub enum CapabilityGrantPopulation {
    None,
    Current,
    CurrentAndFutureReplacement,
    CurrentWithSameResourceUnrelated(usize),
    ExactPairPopulation(usize),
    Composed(crate::domain_computation::primary_graph::tests::fixture::capability_seed::composition::CapabilityCompositionScenario),
    Delegated { links: usize, unrelated: usize },
    Elevated(crate::domain_computation::primary_graph::tests::fixture::capability_elevation_seed::CapabilityElevationScenario),
}

pub struct AuthorizationWorldSpec<'a> {
    pub owner_bindings: &'a [(&'a str, &'a str)],
    pub blocked: bool,
    pub principal_count: usize,
    pub primary_label: &'a str,
    pub resources: WorthQueryApplicationQueryResourceProfile,
    pub capability_grants: CapabilityGrantPopulation,
}

pub struct AuthorizationWorld {
    pub application:
        WorthQueryPrimaryGraphApplicationRuntime<IdentityExecutionSchema>,
    pub binding: InstalledIdentityBinding,
    pub invariant:
        WorthQueryApplicationInvariantProjectionAuthority<IdentityExecutionSchema>,
    pub authorization_time:
        AuthorizationTimeController,
    pub faults:
        std::sync::Arc<worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fault_controller::PrimaryGraphFaultController>,
}

impl AuthorizationWorld {
    pub fn declare_output_contract<Contract: 'static>(&self, identity: &str) {(*
        (*self.application
            .primary_provider_for_test())
            .graph_for_test())
            .output_lineage
            .lock()
            .unwrap()
            .fixture_binding(TypeId::of_for_test::<Contract>(), identity);
    }

    pub fn selected_product(
        &self,
        phase: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    ) -> worth_query_execution::facade::domain_computation::WorthQuerySelectedProductOperation<
        '_,
        IdentityExecutionSchema,
    > {
        let execution = phase
            .execution_request_for_for_test(self.application.product_runtime())
            .expect("fixture lends its installed phase");
        self.application
            .select_product_branch_for_test(
                &execution,
                self.application.product_runtime().default_branch_for_test(),
            )
            .expect("the fixture product branch remains admitted")
    }
}

pub struct PreparedAuthorizationWorld {
    pub runtime: WorthQueryExecutionRuntime,
    pub authority: WorthQueryExecutionInstallationAuthority,
    pub schema: WorthQueryInstalledApplicationSchema<IdentityExecutionSchema>,
    pub binding: InstalledIdentityBinding,
    pub bootstrap: WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
}

pub fn install_authorization_world(spec: AuthorizationWorldSpec<'_>) -> AuthorizationWorld {
    install_authorization_world_with_product_resources(
        spec,
        worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources(),
    )
}

pub fn install_authorization_world_with_product_resources(
    spec: AuthorizationWorldSpec<'_>,
    product_resources: worth_query_execution::facade::integration::WorthQueryProductWorldResources,
) -> AuthorizationWorld {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_bootstrap_advancement(
        128_000_000,
        |phase| install_authorization_world_in_advancement(&phase, spec, product_resources),
    )
}

/// Fixture installation borrows the test's active request when setup is a phase.
pub fn install_authorization_world_in_advancement(
    phase: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    spec: AuthorizationWorldSpec<'_>,
    product_resources: worth_query_execution::facade::integration::WorthQueryProductWorldResources,
) -> AuthorizationWorld {
    let mut prepared = prepare_authorization_world(phase, spec.resources, None, product_resources);
    populate_authorization_world(&mut prepared, &spec);
    publish_authorization_world(phase, prepared)
}

pub fn populate_authorization_world(
    prepared: &mut PreparedAuthorizationWorld,
    spec: &AuthorizationWorldSpec<'_>,
) {
    bind_principals(
        &mut prepared.bootstrap,
        &prepared.binding,
        spec.principal_count,
    );
    bind_query_graph(
        &mut prepared.bootstrap,
        spec.primary_label,
        spec.capability_grants,
    );
    bind_authorization_relations(&mut prepared.bootstrap, spec.owner_bindings, spec.blocked);
    bind_capability_population(&mut prepared.bootstrap, spec.capability_grants);
}

pub fn bind_principals(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    binding: &InstalledIdentityBinding,
    principal_count: usize,
) {
    let principals = [
        ("principal-0", 1_u64, "alice"),
        ("principal-1", 2_u64, "bob"),
        ("principal-2", 3_u64, "carol"),
    ];
    assert!(
        principal_count <= principals.len(),
        "fixture supports at most three principals"
    );
    for (key, identity, subject) in principals.into_iter().take(principal_count) {
        bootstrap
            .bind_principal(
                binding,
                WorthQueryApplicationPrincipalKey::new(key).unwrap(),
                identity,
                external_identity(subject),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
            .unwrap();
    }
}

pub fn bind_query_graph(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    primary_label: &str,
    capability_grants: CapabilityGrantPopulation,
) {
    bind_accounts(bootstrap, primary_label, capability_grants);
    bind_activities(bootstrap);
    bind_activity_relations(bootstrap);
}

pub fn bind_accounts(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    primary_label: &str,
    capability_grants: CapabilityGrantPopulation,
) {
    bind_account(
        bootstrap,
        AccountSeedSpec {
            key: "account-1",
            status: "open",
            label: primary_label,
            note: Some("reviewed"),
        },
    );
    let secondary_status = match capability_grants {
        CapabilityGrantPopulation::Elevated(
            crate::domain_computation::primary_graph::tests::fixture::capability_elevation_seed::CapabilityElevationScenario::DistinctCommandResource,
        ) => "open",
        _ => "unrelated",
    };
    bind_account(
        bootstrap,
        AccountSeedSpec {
            key: "account-2",
            status: secondary_status,
            label: "secondary",
            note: None,
        },
    );
}

pub fn bind_activities(bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>) {
    bind_activity(bootstrap, "activity-primary", 11);
    bind_activity(bootstrap, "activity-secondary", 22);
}

pub fn bind_activity_relations(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
) {
    bind_account_activity(
        bootstrap,
        AccountPrimaryActivity::reference(),
        "primary-activity-1",
        "activity-primary",
    );
    bind_account_activity(
        bootstrap,
        AccountSecondaryActivity::reference(),
        "secondary-activity-1",
        "activity-secondary",
    );
    for (relation, activity) in [
        ("all-activity-2", "activity-secondary"),
        ("all-activity-1", "activity-primary"),
    ] {
        bind_account_activity(
            bootstrap,
            AccountAllActivity::reference(),
            relation,
            activity,
        );
    }
    for (relation, activity) in [
        ("reverse-activity-1", "activity-primary"),
        ("reverse-activity-2", "activity-secondary"),
    ] {
        bootstrap
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                ActivityAccount::reference(),
                relation,
                WorthQueryApplicationEntityKey::new(activity).unwrap(),
                WorthQueryApplicationEntityKey::new("account-1").unwrap(),
            ))
            .unwrap();
    }
}

pub fn bind_account_activity<Relation>(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    relation_reference: worth_query_declaration::facade::application_schema::ApplicationRelationRef<
        IdentityExecutionSchema,
        Relation,
        Account,
        Activity,
    >,
    relation: &str,
    activity: &str,
) {
    bootstrap
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            relation_reference,
            relation,
            WorthQueryApplicationEntityKey::new("account-1").unwrap(),
            WorthQueryApplicationEntityKey::new(activity).unwrap(),
        ))
        .unwrap();
}

pub fn bind_authorization_relations(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    owner_bindings: &[(&str, &str)],
    blocked: bool,
) {
    for (ordinal, (principal, account)) in owner_bindings.iter().enumerate() {
        bootstrap
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                AccountOwner::reference(),
                format!("owner-{}", ordinal + 1),
                WorthQueryApplicationEntityKey::new(*principal).unwrap(),
                WorthQueryApplicationEntityKey::new(*account).unwrap(),
            ))
            .unwrap();
    }
    if blocked {
        bootstrap
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                AccountBlocked::reference(),
                "blocked-1",
                WorthQueryApplicationEntityKey::new("principal-0").unwrap(),
                WorthQueryApplicationEntityKey::new("account-1").unwrap(),
            ))
            .unwrap();
    }
}

pub fn bind_capability_population(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    population: CapabilityGrantPopulation,
) {
    match population {
        CapabilityGrantPopulation::None => {}
        CapabilityGrantPopulation::Current => crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_grant(bootstrap),
        CapabilityGrantPopulation::CurrentAndFutureReplacement => {
            crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_grant(bootstrap);
            crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_future_replacement_grant(bootstrap);
        }
        CapabilityGrantPopulation::CurrentWithSameResourceUnrelated(unrelated) => {
            crate::domain_computation::primary_graph::tests::fixture::capability_population_seed::bind_same_resource_unrelated_grants(
                bootstrap, unrelated,
            )
        }
        CapabilityGrantPopulation::ExactPairPopulation(count) => {
            crate::domain_computation::primary_graph::tests::fixture::capability_population_seed::bind_exact_pair_grants(bootstrap, count)
        }
        CapabilityGrantPopulation::Composed(scenario) => {
            crate::domain_computation::primary_graph::tests::fixture::capability_seed::composition::bind_composed_grant(bootstrap, scenario)
        }
        CapabilityGrantPopulation::Delegated { links, unrelated } => {
            crate::domain_computation::primary_graph::tests::fixture::capability_seed::bind_delegated_grants(bootstrap, links, unrelated)
        }
        CapabilityGrantPopulation::Elevated(scenario) => {
            crate::domain_computation::primary_graph::tests::fixture::capability_elevation_seed::bind_elevated_capability(bootstrap, scenario)
        }
    }
}

pub fn bind_activity(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    key: &str,
    sequence: u64,
) {
    bootstrap
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Activity::reference(),
                WorthQueryApplicationEntityKey::new(key).unwrap(),
            )
            .field(ActivityIdentity::reference(), key.to_owned())
            .field(ActivitySequence::reference(), sequence),
        )
        .unwrap();
}

pub fn portable_package(
    declaration: worth_query_declaration::facade::application_schema::ApplicationSchemaDeclaration<
        IdentityExecutionSchema,
    >,
) -> WorthQueryValidatedPortableDomainPackage {
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "identity_execution_test",
        1,
        0,
    ))
    .application_schema(declaration)
    .validate()
    .unwrap()
}

#[path = "preparation/mod.rs"]
pub mod preparation;
#[path = "publication/mod.rs"]
pub mod publication;
#[path = "reclaiming/mod.rs"]
pub mod reclaiming;
