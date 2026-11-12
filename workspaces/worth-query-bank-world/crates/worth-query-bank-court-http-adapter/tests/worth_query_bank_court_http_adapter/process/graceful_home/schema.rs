//! The dispatching Home declaration over the shared Home account records.
use super::operation::{
    EstateDeathNotificationEffect, NotifyRail, NotifyRailBinding, NotifyRailHandler,
};
use super::rejection::{
    UnservedNotice, UnservedNoticeBinding, UnservedNoticeEffect, UnservedNoticeHandler,
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
pub use worth_query_testkit_host::facade::host::application_home::schema::{
    Account, AccountIdentity, ExternalIdentity, ExternalMapping, HomeRecords, MappingTarget,
    Principal, PrincipalIdentity,
};

pub struct RailHomeSchema;
impl Fragment for RailHomeSchema {
    const NAME: &'static str = "RailHomeSchema";
    type Bindings = NoBindings;
    type Members = (RailSurface,);
}
impl ApplicationSchema for RailHomeSchema {
    const OWNER: &'static str = "bank_rail_home_test";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for RailHomeSchema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct RailSurface;
impl ApplicationSchemaContribution<RailHomeSchema> for RailSurface {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.bank.rail-home.surface.v1");
    type Members = (
        HomeRecords,
        RailIdentityBinding,
        EstateDeathNotificationEffect,
        NotifyRail,
        NotifyRailBinding,
        UnservedNoticeEffect,
        UnservedNotice,
        UnservedNoticeBinding,
    );
}
impl WorthQueryApplicationContribution<RailHomeSchema> for RailSurface {
    type Configuration = std::sync::Arc<std::sync::atomic::AtomicUsize>;
    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, RailHomeSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<NotifyRailBinding, _>(NotifyRailHandler(configuration.clone()))?;
        setup.handler::<UnservedNoticeBinding, _>(UnservedNoticeHandler(configuration))
    }
}

pub struct RailIdentityBinding;
impl RailIdentityBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        RailHomeSchema,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "RailIdentityBinding",
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
impl Registration<RailHomeSchema> for RailIdentityBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<RailHomeSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<RailHomeSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.principal_binding(Self::reference()))
    }
}

/// One enabled principal and one account whose balance starts at seven.
pub(super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<RailHomeSchema>,
    schema: &WorthQueryInstalledApplicationSchema<RailHomeSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let binding = schema
        .principal_binding(RailIdentityBinding::reference())
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
