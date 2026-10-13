//! Install the proof's declared application rule and seed its authorized account.
use super::accounting_rule::{AccountingRule, HookAccounting};
use super::schema::*;
use crate::domain_computation::execution_runtime::*;
use crate::domain_computation::primary_graph::*;
use worth_query_declaration::facade::application_schema::*;
use worth_query_declaration::facade::authentication::*;
use worth_query_installation::facade::*;

pub(super) struct World {
    pub(super) application: WorthQueryPrimaryGraphApplicationRuntime<HookSchema>,
    pub(super) binding: WorthQueryInstalledPrincipalBinding<
        HookSchema,
        IdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    >,
    pub(super) invariant: WorthQueryApplicationInvariantProjectionAuthority<HookSchema>,
}

pub(super) fn install(rule: AccountingRule) -> World {
    try_install(rule).expect("checked rule permits principal bootstrap publication")
}

pub(super) fn try_install(
    rule: AccountingRule,
) -> Result<World, WorthQueryPrimaryGraphInstallationDenial> {
    with_test_advancement(|phase| {
        let declaration = HookSchema::declaration().unwrap();
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            "hook_accounting_test",
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
                HookAccounting::reference(),
                ApplicationInvariantExecutionPoint::CommitBoundary,
            )
            .unwrap();
        let mut factories = WorthQueryApplicationInvariantFactories::for_installed_schema(&schema);
        factories.bind(&invariant, move |_| Ok(rule)).unwrap();
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
        let application = bootstrap.publish_application_runtime(
            &phase.bootstrap_for_test(),
            runtime,
            authority,
            schema,
            worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
        )?;
        Ok(World {
            application,
            binding,
            invariant,
        })
    })
}
