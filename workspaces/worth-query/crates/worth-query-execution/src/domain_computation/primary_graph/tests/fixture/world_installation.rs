//! Identity-world installation and authorization-world scenario entry points.

use super::account_seed::{bind_account, AccountSeedSpec};
pub(super) use super::authorization_world_installation::AuthorizationWorld;
use super::authorization_world_installation::{
    install_authorization_world, AuthorizationWorldSpec, CapabilityGrantPopulation,
};
use super::*;

const PRINCIPAL_ZERO_ACCOUNTS: &[(&str, &str)] =
    &[("principal-0", "account-1"), ("principal-0", "account-2")];

pub(in crate::domain_computation::primary_graph::tests) fn installed_world(
    rows: &[(&str, WorthQueryPrincipalMappingStatus)],
) -> IdentityWorld {
    installed_world_with_policy_fact(rows, false)
}

pub(in crate::domain_computation::primary_graph::tests) fn installed_world_with_policy_fact(
    rows: &[(&str, WorthQueryPrincipalMappingStatus)],
    include_policy_fact: bool,
) -> IdentityWorld {
    let declaration = IdentityExecutionSchema::declaration().unwrap();
    let package = portable_package(declaration.clone());
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    let installation = WorthQueryExecutionRuntimeInstaller::new()
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
    let mut bootstrap = authority.prepare_primary_graph(&runtime, &schema, crate::domain_computation::execution_runtime::product_world::test_product_world_resources()).unwrap();
    for (ordinal, (subject, status)) in rows.iter().enumerate() {
        bootstrap
            .bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new(format!("principal-{ordinal}")).unwrap(),
                u64::try_from(ordinal + 1).unwrap(),
                external_identity(subject),
                *status,
            )
            .unwrap();
    }
    if include_policy_fact {
        bind_account(
            &mut bootstrap,
            AccountSeedSpec {
                key: "account-1",
                status: "open",
                label: "primary",
                note: Some("reviewed"),
            },
        );
        bootstrap
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                AccountOwner::reference(),
                "owner-1",
                WorthQueryApplicationEntityKey::new("principal-0").unwrap(),
                WorthQueryApplicationEntityKey::new("account-1").unwrap(),
            ))
            .unwrap();
    }
    let application = bootstrap
        .publish_application_runtime(
            runtime,
            authority,
            schema,
            worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
        )
        .unwrap();
    IdentityWorld {
        application,
        binding,
    }
}

pub(in crate::domain_computation::primary_graph) fn installed_authorization_world(
    include_owner_relation: bool,
) -> AuthorizationWorld {
    install_authorization_world(AuthorizationWorldSpec {
        owner_bindings: owner_bindings(include_owner_relation),
        ..standard_spec()
    })
}

pub(in crate::domain_computation::primary_graph) fn installed_authorization_world_with_label(
    label: &str,
) -> AuthorizationWorld {
    install_authorization_world(AuthorizationWorldSpec {
        owner_bindings: PRINCIPAL_ZERO_ACCOUNTS,
        primary_label: label,
        ..standard_spec()
    })
}

pub(in crate::domain_computation::primary_graph) fn installed_authorization_world_with_resource_profile(
    resources: WorthQueryApplicationQueryResourceProfile,
) -> AuthorizationWorld {
    install_authorization_world(AuthorizationWorldSpec {
        owner_bindings: PRINCIPAL_ZERO_ACCOUNTS,
        resources,
        ..standard_spec()
    })
}

pub(in crate::domain_computation::primary_graph) fn installed_two_principal_authorization_world(
    include_owner_relation: bool,
) -> AuthorizationWorld {
    install_authorization_world(AuthorizationWorldSpec {
        owner_bindings: owner_bindings(include_owner_relation),
        principal_count: 2,
        ..standard_spec()
    })
}

pub(in crate::domain_computation::primary_graph) fn installed_blocked_authorization_world(
) -> AuthorizationWorld {
    install_authorization_world(AuthorizationWorldSpec {
        owner_bindings: PRINCIPAL_ZERO_ACCOUNTS,
        blocked: true,
        ..standard_spec()
    })
}

fn standard_spec<'a>() -> AuthorizationWorldSpec<'a> {
    AuthorizationWorldSpec {
        owner_bindings: &[],
        blocked: false,
        principal_count: 1,
        primary_label: "primary",
        resources: WorthQueryApplicationQueryResourceProfile::default(),
        capability_grants: CapabilityGrantPopulation::None,
    }
}

fn owner_bindings(include: bool) -> &'static [(&'static str, &'static str)] {
    if include {
        PRINCIPAL_ZERO_ACCOUNTS
    } else {
        &[]
    }
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
