//! Native Home declaration; A and B share the same base model path and contracts.
use super::native_contribution::ExtendedSurface;
use worth_query_decl::facade::{
    application_schema::{
        fragment::{self, *},
        *,
    },
    authentication::{WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus},
};
use worth_query_host::facade::{
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
#[path = "schema/later_records.rs"]
mod later_records;
pub use later_records::*;

pub struct SuccessorSchema;
impl Fragment for SuccessorSchema {
    const NAME: &'static str = "HomeIdentitySchema";
    type Bindings = NoBindings;
    type Members = (ExtendedSurface,);
}
impl ApplicationSchema for SuccessorSchema {
    const OWNER: &'static str = "host_home_test";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for SuccessorSchema {
    type Contributions = <Self as Fragment>::Members;
}
pub struct IdentityBinding;
impl IdentityBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        SuccessorSchema,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "IdentityBinding",
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
impl Registration<SuccessorSchema> for IdentityBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<SuccessorSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SuccessorSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.principal_binding(Self::reference()))
    }
}
pub(super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<SuccessorSchema>,
    schema: &WorthQueryInstalledApplicationSchema<SuccessorSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let binding = schema
        .principal_binding(IdentityBinding::reference())
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
