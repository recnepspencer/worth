//! The Home fixture's actual declared schema, with a minimal genuine principal binding.
use super::native_contribution::HomeNativeSurface;
use worth_query_decl::facade::{
    application_schema::{StringApplicationValueBinding, U64ApplicationValueBinding},
    authentication::{
        WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
        WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
    },
    worth_query_application, worth_query_aspect, worth_query_entity, worth_query_field,
    worth_query_principal_binding, worth_query_relation,
};
use worth_query_host::facade::{
    domain::WorthQueryInstalledApplicationSchema,
    primary_graph::{
        WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
        WorthQueryApplicationPrincipalKey, WorthQueryPrimaryGraphBootstrap,
        WorthQueryPrimaryGraphInstallationDenial,
    },
};
worth_query_application! {
    pub HomeIdentitySchema { owner: "host_home_test", version: (1, 0), contributions: [HomeNativeSurface], }
}
worth_query_entity!(pub ExternalMapping for HomeIdentitySchema);
worth_query_entity!(pub Principal for HomeIdentitySchema);
worth_query_entity!(pub Account for HomeIdentitySchema);
worth_query_aspect!(pub ExternalIdentity for HomeIdentitySchema, ExternalMapping; identity = AspectIdentity(0x9161103e), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PrincipalIdentity for HomeIdentitySchema, Principal; identity = AspectIdentity(0x9161103f), revision = AspectContractRevision(1),);
worth_query_aspect!(pub AccountIdentity for HomeIdentitySchema, Account; identity = AspectIdentity(0x91611040), revision = AspectContractRevision(1),);
worth_query_field!(pub ExternalIdentityField for HomeIdentitySchema, ExternalMapping, ExternalIdentity: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for HomeIdentitySchema, ExternalMapping, ExternalIdentity: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentityField for HomeIdentitySchema, Principal, PrincipalIdentity: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub AccountNumber for HomeIdentitySchema, Account, AccountIdentity: String => StringApplicationValueBinding, read_only, equality);
worth_query_principal_binding!(pub IdentityBinding in HomeIdentitySchema, mapping ExternalMapping { identity: ExternalIdentityField, status: MappingStatusField, target: MappingTarget => Principal, principal_identity: PrincipalIdentityField });
worth_query_relation!(pub MappingTarget in HomeIdentitySchema, ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);

pub(super) fn declare(
    schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<
        HomeIdentitySchema,
    >,
) -> worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<
    HomeIdentitySchema,
> {
    schema
        .entity(ExternalMapping::reference())
        .entity(Principal::reference())
        .entity(Account::reference())
        .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
        .aspect(Principal::reference(), PrincipalIdentity::reference())
        .aspect(Account::reference(), AccountIdentity::reference())
        .field(
            ExternalMapping::reference(),
            ExternalIdentityField::reference(),
        )
        .field(
            ExternalMapping::reference(),
            MappingStatusField::reference(),
        )
        .field(Principal::reference(), PrincipalIdentityField::reference())
        .field(Account::reference(), AccountNumber::reference())
        .relation(
            MappingTarget::reference(),
            ExternalMapping::reference(),
            Principal::reference(),
        )
        .principal_binding(IdentityBinding::reference())
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
                Account::reference(),
                WorthQueryApplicationEntityKey::new("account-row").unwrap(),
            )
            .field(AccountNumber::reference(), "account-001".to_string())
            .field(super::native_mutation::Balance::reference(), 7),
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
            Account::reference(),
            WorthQueryApplicationEntityKey::new("second-account-row").unwrap(),
        )
        .field(AccountNumber::reference(), "account-002".to_owned())
        .field(super::native_mutation::Balance::reference(), 11),
    )?;
    if flush_final {
        graph.commit_seed_batch(phase)?;
    }
    Ok(())
}
