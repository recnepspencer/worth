//! Installed authorization-world fixture compiler and its population phases.

use super::account_seed::{bind_account, AccountSeedSpec};
use super::*;
use crate::domain_computation::execution_runtime::WorthQueryExecutionInstallationAuthority;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationInvariantProjectionAuthority, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrimaryGraphBootstrap,
};

#[derive(Clone, Copy)]
pub(super) enum CapabilityGrantPopulation {
    None,
    Current,
    CurrentAndFutureReplacement,
    CurrentWithSameResourceUnrelated(usize),
    ExactPairPopulation(usize),
    Composed(super::capability_seed::CapabilityCompositionScenario),
    Delegated { links: usize, unrelated: usize },
    Elevated(super::capability_elevation_seed::CapabilityElevationScenario),
}

pub(super) struct AuthorizationWorldSpec<'a> {
    pub(super) owner_bindings: &'a [(&'a str, &'a str)],
    pub(super) blocked: bool,
    pub(super) principal_count: usize,
    pub(super) primary_label: &'a str,
    pub(super) resources: WorthQueryApplicationQueryResourceProfile,
    pub(super) capability_grants: CapabilityGrantPopulation,
}

pub(in crate::domain_computation) struct AuthorizationWorld {
    pub(in crate::domain_computation) application:
        WorthQueryPrimaryGraphApplicationRuntime<IdentityExecutionSchema>,
    pub(in crate::domain_computation::primary_graph) binding: InstalledIdentityBinding,
    pub(in crate::domain_computation::primary_graph) invariant:
        WorthQueryApplicationInvariantProjectionAuthority<IdentityExecutionSchema>,
    pub(in crate::domain_computation::primary_graph) authorization_time:
        AuthorizationTimeController,
    pub(in crate::domain_computation::primary_graph) faults:
        std::sync::Arc<crate::domain_computation::primary_graph::tests::fault_controller::PrimaryGraphFaultController>,
}

impl AuthorizationWorld {
    pub(in crate::domain_computation) fn selected_product(
        &self,
    ) -> crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
        '_,
        IdentityExecutionSchema,
    > {
        self.application
            .select_product_branch(self.application.product_runtime().default_branch())
            .expect("the fixture product branch remains admitted")
    }
}

struct PreparedAuthorizationWorld {
    runtime: WorthQueryExecutionRuntime,
    authority: WorthQueryExecutionInstallationAuthority,
    schema: WorthQueryInstalledApplicationSchema<IdentityExecutionSchema>,
    binding: InstalledIdentityBinding,
    bootstrap: WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
}

pub(super) fn install_authorization_world(spec: AuthorizationWorldSpec<'_>) -> AuthorizationWorld {
    let mut prepared = prepare_authorization_world(spec.resources, None);
    populate_authorization_world(&mut prepared, &spec);
    publish_authorization_world(prepared)
}

fn populate_authorization_world(
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

fn prepare_authorization_world(
    resources: WorthQueryApplicationQueryResourceProfile,
    relational: Option<worth_relational::facade::runtime::RelationalRuntime>,
) -> PreparedAuthorizationWorld {
    let declaration = IdentityExecutionSchema::declaration().unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(portable_package(declaration.clone()))
        .unwrap();
    let installation = WorthQueryExecutionRuntimeInstaller::new()
        .application_query_resources(resources)
        .install(WorthQueryInstallationGeneration::initial(), [admitted])
        .unwrap();
    let (runtime, authority) = installation.into_parts();
    let schema = runtime
        .installed_packages()
        .bind_application_schema(declaration)
        .unwrap();
    let binding = schema
        .principal_binding(IdentityBinding::reference())
        .unwrap();
    let bootstrap = match relational {
        Some(relational) => authority
            .prepare_primary_graph_with_relational_runtime(
                &runtime,
                &schema,
                relational,
                crate::domain_computation::execution_runtime::product_world::test_product_world_resources(),
            )
            .unwrap(),
        None => authority.prepare_primary_graph(&runtime, &schema, crate::domain_computation::execution_runtime::product_world::test_product_world_resources()).unwrap(),
    };
    PreparedAuthorizationWorld {
        runtime,
        authority,
        schema,
        binding,
        bootstrap,
    }
}

fn bind_principals(
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

fn bind_query_graph(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    primary_label: &str,
    capability_grants: CapabilityGrantPopulation,
) {
    bind_accounts(bootstrap, primary_label, capability_grants);
    bind_activities(bootstrap);
    bind_activity_relations(bootstrap);
}

fn bind_accounts(
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
            super::capability_elevation_seed::CapabilityElevationScenario::DistinctCommandResource,
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

fn bind_activities(bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>) {
    bind_activity(bootstrap, "activity-primary", 11);
    bind_activity(bootstrap, "activity-secondary", 22);
}

fn bind_activity_relations(
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

fn bind_account_activity<Relation>(
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

fn bind_authorization_relations(
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

fn bind_capability_population(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
    population: CapabilityGrantPopulation,
) {
    match population {
        CapabilityGrantPopulation::None => {}
        CapabilityGrantPopulation::Current => super::capability_seed::bind_grant(bootstrap),
        CapabilityGrantPopulation::CurrentAndFutureReplacement => {
            super::capability_seed::bind_grant(bootstrap);
            super::capability_seed::bind_future_replacement_grant(bootstrap);
        }
        CapabilityGrantPopulation::CurrentWithSameResourceUnrelated(unrelated) => {
            super::capability_population_seed::bind_same_resource_unrelated_grants(
                bootstrap, unrelated,
            )
        }
        CapabilityGrantPopulation::ExactPairPopulation(count) => {
            super::capability_population_seed::bind_exact_pair_grants(bootstrap, count)
        }
        CapabilityGrantPopulation::Composed(scenario) => {
            super::capability_seed::bind_composed_grant(bootstrap, scenario)
        }
        CapabilityGrantPopulation::Delegated { links, unrelated } => {
            super::capability_seed::bind_delegated_grants(bootstrap, links, unrelated)
        }
        CapabilityGrantPopulation::Elevated(scenario) => {
            super::capability_elevation_seed::bind_elevated_capability(bootstrap, scenario)
        }
    }
}

fn publish_authorization_world(prepared: PreparedAuthorizationWorld) -> AuthorizationWorld {
    let PreparedAuthorizationWorld {
        runtime,
        authority,
        schema,
        binding,
        bootstrap,
    } = prepared;
    let invariant = bootstrap.retain_invariant_projection_authority();
    let authorization_time = AuthorizationTimeController::default();
    let faults = std::sync::Arc::new(
        crate::domain_computation::primary_graph::tests::fault_controller::PrimaryGraphFaultController::default(),
    );
    let application = bootstrap
        .publish_application_runtime_with_ports(
            runtime,
            authority,
            schema,
            worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
            authorization_time.clone(),
            faults.clone(),
        )
        .unwrap();
    AuthorizationWorld {
        application,
        binding,
        invariant,
        authorization_time,
        faults,
    }
}

fn bind_activity(
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

fn portable_package(
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
