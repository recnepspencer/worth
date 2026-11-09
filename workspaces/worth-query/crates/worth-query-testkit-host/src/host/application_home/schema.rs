//! Native Home declaration; A and B share the same base model path and contracts.
use super::native_contribution::HomeNativeSurface;
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
#[path = "schema/records.rs"]
mod records;
pub use records::*;

pub struct HomeIdentitySchema;
impl Fragment for HomeIdentitySchema {
    const NAME: &'static str = "HomeIdentitySchema";
    type Bindings = NoBindings;
    type Members = (HomeNativeSurface,);
}
impl ApplicationSchema for HomeIdentitySchema {
    const OWNER: &'static str = "host_home_test";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for HomeIdentitySchema {
    type Contributions = <Self as Fragment>::Members;
}
pub struct IdentityBinding;
impl IdentityBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        HomeIdentitySchema,
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
impl Registration<HomeIdentitySchema> for IdentityBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<HomeIdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<HomeIdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.principal_binding(Self::reference()))
    }
}
pub(super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<HomeIdentitySchema>,
    schema: &WorthQueryInstalledApplicationSchema<HomeIdentitySchema>,
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

/// Two accepted seed batches, with either a flushed or pending final batch.
pub fn seed_batches(
    phase: &worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase<
        '_,
    >,
    graph: &mut WorthQueryPrimaryGraphBootstrap<HomeIdentitySchema>,
    schema: &WorthQueryInstalledApplicationSchema<HomeIdentitySchema>,
    flush_final: bool,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    seed(graph, schema)?;
    graph.commit_seed_batch(phase)?;
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            fragment::entity(HomeRecords, Account),
            WorthQueryApplicationEntityKey::new("second-account-row").unwrap(),
        )
        .field(
            FieldSelection::<HomeRecords, Account, AccountIdentity, String>::at("AccountNumber"),
            "account-002".to_owned(),
        )
        .field(
            FieldSelection::<HomeRecords, Account, AccountIdentity, u64>::at("Balance"),
            11,
        ),
    )?;
    if flush_final {
        graph.commit_seed_batch(phase)?;
    }
    Ok(())
}
