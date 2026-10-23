//! Install the proof's declared application rule and seed its authorized account.
use super::rule::{First, Rule, Second};
use super::schema::*;
use crate::domain_computation::execution_runtime::*;
use crate::domain_computation::primary_graph::*;
use worth_query_declaration::facade::application_schema::*;
use worth_query_declaration::facade::authentication::*;
use worth_query_installation::facade::*;

pub(super) struct World {
    pub(super) application: WorthQueryPrimaryGraphApplicationRuntime<RefusalSchema>,
    pub(super) binding: WorthQueryInstalledPrincipalBinding<
        RefusalSchema,
        IdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    >,
    pub(super) invariant: WorthQueryApplicationInvariantProjectionAuthority<RefusalSchema>,
}

pub(super) fn install(first: Rule, second: Rule) -> World {
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
        let mut bootstrap = authority.prepare_primary_graph_with_relational_runtime_and_invariants(
            &phase.bootstrap_for_test(), &runtime, &schema,
            worth_relational::facade::runtime::RelationalRuntimeApi::builder().build(),
            crate::domain_computation::execution_runtime::product_world::test_product_world_resources(), factories,
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
