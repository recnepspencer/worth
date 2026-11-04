//! Install the proof's declared application rule and seed its authorized account.
use crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::ledger_rule::LedgerOnly;
use crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::ledger_rule::LedgerRule;
use crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::rule::First;
use crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::rule::Rule;
use crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::rule::Second;
use crate::domain_computation::primary_graph::tests::application_attempt::typed_invariant_refusal::schema::*;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityKey;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntitySeed;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantFactories;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationInvariantProjectionAuthority;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationPrincipalKey;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationRelationSeed;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement;
use worth_query_declaration::facade::application_schema::*;
use worth_query_declaration::facade::authentication::*;
use worth_query_installation::facade::*;

pub struct World {
    pub application: WorthQueryPrimaryGraphApplicationRuntime<RefusalSchema>,
    pub binding: WorthQueryInstalledPrincipalBinding<
        RefusalSchema,
        IdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    >,
    pub invariant: WorthQueryApplicationInvariantProjectionAuthority<RefusalSchema>,
}

pub fn install(first: Rule, second: Rule) -> World {
    with_test_advancement(|phase| {
        let declaration = RefusalSchema::declaration().unwrap();
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            "typed_refusal_test",
            1,
            0,
        ))
        .application_schema(declaration.clone())
        .validate()
        .unwrap();
        let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
            .admit(package)
            .unwrap();
        let (runtime, authority) = WorthQueryExecutionRuntimeInstaller::new()
            .install(WorthQueryInstallationGeneration::initial(), [admitted])
            .unwrap()
            .into_parts();
        let schema = runtime
            .installed_packages()
            .bind_application_schema(declaration)
            .unwrap();
        let binding = schema
            .principal_binding(IdentityBinding::reference())
            .unwrap();
        let invariant = schema
            .installed_invariant(
                First::reference(),
                ApplicationInvariantExecutionPoint::CommitBoundary,
            )
            .unwrap();
        let mut factories = WorthQueryApplicationInvariantFactories::for_installed_schema(&schema);
        factories
            .bind(&invariant, move |resolver| first.resolve(resolver))
            .unwrap();
        let second_handle = schema
            .installed_invariant(
                Second::reference(),
                ApplicationInvariantExecutionPoint::CommitBoundary,
            )
            .unwrap();
        factories
            .bind(&second_handle, move |resolver| second.resolve(resolver))
            .unwrap();
        let ledger = schema
            .installed_invariant(
                LedgerOnly::reference(),
                ApplicationInvariantExecutionPoint::CommitBoundary,
            )
            .unwrap();
        factories
            .bind(&ledger, |_| Ok::<_, String>(LedgerRule))
            .unwrap();
        let mut bootstrap = authority.prepare_primary_graph_with_relational_runtime_and_invariants_for_test(
            &phase.bootstrap_for_test(), &runtime, &schema,
            worth_relational::facade::runtime::RelationalRuntimeApi::builder().build().expect("valid native runtime configuration"),
            worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()), factories,
        ).unwrap();
        bootstrap
            .bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("principal-0").unwrap(),
                1,
                WorthQueryExternalPrincipalIdentity::new("https://issuer.example", "alice")
                    .unwrap(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
            .unwrap();
        bootstrap
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Account::reference(),
                    WorthQueryApplicationEntityKey::new("account-1").unwrap(),
                )
                .field(AccountIdentity::reference(), "account-1".to_owned())
                .field(AccountStatus::reference(), "open".to_owned()),
            )
            .unwrap();
        bootstrap
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                AccountOwner::reference(),
                "owner-1",
                WorthQueryApplicationEntityKey::new("principal-0").unwrap(),
                WorthQueryApplicationEntityKey::new("account-1").unwrap(),
            ))
            .unwrap();
        let invariant = bootstrap.retain_invariant_projection_authority();
        let application = bootstrap
            .publish_application_runtime(
                &phase.bootstrap_for_test(),
                runtime,
                authority,
                schema,
                worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
            )
            .unwrap();
        World {
            application,
            binding,
            invariant,
        }
    })
}
