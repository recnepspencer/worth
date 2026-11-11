//! The dispatching Home declaration over the shared Home account records.
use super::notify::{BalanceNoticeEffect, NotifyBalance, NotifyBalanceBinding, NotifyHandler};
pub use crate::facade::host::application_home::schema::{
    Account, AccountIdentity, ExternalIdentity, ExternalMapping, HomeRecords, MappingTarget,
    Principal, PrincipalIdentity,
};
use worth_query_decl::facade::{
    application_schema::{
        fragment::{self, *},
        *,
    },
    authentication::{WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus},
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    domain::WorthQueryInstalledApplicationSchema,
    primary_graph::{
        WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
        WorthQueryApplicationPrincipalKey, WorthQueryPrimaryGraphBootstrap,
        WorthQueryPrimaryGraphInstallationDenial,
    },
};

pub struct DispatchHomeSchema;
impl Fragment for DispatchHomeSchema {
    const NAME: &'static str = "DispatchHomeSchema";
    type Bindings = NoBindings;
    type Members = (DispatchSurface,);
}
impl ApplicationSchema for DispatchHomeSchema {
    const OWNER: &'static str = "host_dispatch_home_test";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for DispatchHomeSchema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct DispatchSurface;
impl ApplicationSchemaContribution<DispatchHomeSchema> for DispatchSurface {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.host.dispatch-home.surface.v1");
    type Members = (
        HomeRecords,
        DispatchIdentityBinding,
        BalanceNoticeEffect,
        NotifyBalance,
        NotifyBalanceBinding,
        super::recoverable::RecoverableNotice,
        super::recoverable::RecoverableNoticeBinding,
    );
}
impl WorthQueryApplicationContribution<DispatchHomeSchema> for DispatchSurface {
    type Configuration = std::sync::Arc<std::sync::atomic::AtomicUsize>;
    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, DispatchHomeSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<NotifyBalanceBinding, _>(NotifyHandler(configuration.clone()))?;
        setup.handler::<super::recoverable::RecoverableNoticeBinding, _>(
            super::recoverable::RecoverableNoticeHandler(configuration),
        )?;
        Ok(())
    }
}

pub struct DispatchIdentityBinding;
impl DispatchIdentityBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        DispatchHomeSchema,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "DispatchIdentityBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    FieldSelection::<
                        HomeRecords,
                        ExternalMapping,
                        ExternalIdentity,
                        WorthQueryExternalPrincipalIdentity,
                    >::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    FieldSelection::<
                        HomeRecords,
                        ExternalMapping,
                        ExternalIdentity,
                        WorthQueryPrincipalMappingStatus,
                    >::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(fragment::relation(
                    HomeRecords,
                    ExternalMapping,
                    MappingTarget,
                )),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    FieldSelection::<HomeRecords, Principal, PrincipalIdentity, u64>::at(
                        "PrincipalIdentityField",
                    ),
                ),
            },
        )
    }
}
impl Registration<DispatchHomeSchema> for DispatchIdentityBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<DispatchHomeSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<DispatchHomeSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.principal_binding(Self::reference()))
    }
}

/// One enabled principal and one account whose balance starts at seven.
pub(super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<DispatchHomeSchema>,
    schema: &WorthQueryInstalledApplicationSchema<DispatchHomeSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let binding = schema
        .principal_binding(DispatchIdentityBinding::reference())
        .unwrap();
    graph
        .bind_principal(
            &binding,
            WorthQueryApplicationPrincipalKey::new("host-principal").unwrap(),
            1_u64,
            WorthQueryExternalPrincipalIdentity::new("https://issuer.example", "subject").unwrap(),
            WorthQueryPrincipalMappingStatus::Enabled,
        )
        .unwrap();
    graph
        .bind_entity(
            WorthQueryApplicationEntitySeed::new(
                fragment::entity(HomeRecords, Account),
                WorthQueryApplicationEntityKey::new("account-row").unwrap(),
            )
            .field(
                FieldSelection::<HomeRecords, Account, AccountIdentity, String>::at(
                    "AccountNumber",
                ),
                "account-001".to_string(),
            )
            .field(
                FieldSelection::<HomeRecords, Account, AccountIdentity, u64>::at("Balance"),
                7,
            ),
        )
        .unwrap();
    Ok(())
}
