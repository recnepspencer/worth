//! Successor-owned schema for the additive B application.
use super::native_contribution::ExtendedSurface;
use worth_query_decl::facade::{
    application_schema::{
        ApplicationSchemaContributionAuthoring, StringApplicationValueBinding,
        U64ApplicationValueBinding,
    },
    authentication::{
        WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
        WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
    },
    worth_query_aspect, worth_query_entity, worth_query_field, worth_query_principal_binding,
    worth_query_relation,
};
use worth_query_host::facade::{
    domain::WorthQueryInstalledApplicationSchema,
    primary_graph::{
        WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
        WorthQueryApplicationPrincipalKey, WorthQueryPrimaryGraphBootstrap,
        WorthQueryPrimaryGraphInstallationDenial,
    },
};
// A distinct Rust schema owns B's complete static declaration. Its stable
// application header and explicit aspect identities preserve the old roles.
pub struct SuccessorSchema;
impl worth_query_decl::facade::application_schema::ApplicationSchema for SuccessorSchema {
    const OWNER: &'static str = "host_home_test";
    const NAME: &'static str = "HomeIdentitySchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration() -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclaration<Self>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder::<Self>::for_schema().contributions().register::<ExtendedSurface>()?.build()
    }
}
impl worth_query_decl::facade::application_schema::ApplicationSchemaComposition
    for SuccessorSchema
{
    type Contributions = (ExtendedSurface,);
}
worth_query_entity!(pub ExternalMapping for SuccessorSchema);
worth_query_entity!(pub Principal for SuccessorSchema);
worth_query_entity!(pub Account for SuccessorSchema);
worth_query_aspect!(pub ExternalIdentity for SuccessorSchema, ExternalMapping; identity = AspectIdentity(0x9161103e), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PrincipalIdentity for SuccessorSchema, Principal; identity = AspectIdentity(0x9161103f), revision = AspectContractRevision(1),);
worth_query_aspect!(pub AccountIdentity for SuccessorSchema, Account; identity = AspectIdentity(0x91611040), revision = AspectContractRevision(1),);
worth_query_field!(pub ExternalIdentityField for SuccessorSchema, ExternalMapping, ExternalIdentity: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for SuccessorSchema, ExternalMapping, ExternalIdentity: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentityField for SuccessorSchema, Principal, PrincipalIdentity: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub AccountNumber for SuccessorSchema, Account, AccountIdentity: String => StringApplicationValueBinding, read_only, equality);
worth_query_principal_binding!(pub IdentityBinding in SuccessorSchema, mapping ExternalMapping { identity: ExternalIdentityField, status: MappingStatusField, target: MappingTarget => Principal, principal_identity: PrincipalIdentityField });
worth_query_relation!(pub MappingTarget in SuccessorSchema, ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
// Deliberately never seeded: physical meaning must survive without record usage.
worth_query_relation!(pub UnusedAccountPeer in SuccessorSchema, Account => Account; integrity = same_context_no_self_edges_unbounded_retain_dangling);

worth_query_entity!(pub LaterRecord for SuccessorSchema);
worth_query_aspect!(pub LaterIdentity for SuccessorSchema, LaterRecord; identity = AspectIdentity(0xda160001), revision = AspectContractRevision(1),);
worth_query_field!(pub Label for SuccessorSchema, LaterRecord, LaterIdentity: String => StringApplicationValueBinding, read_write, equality);

pub(super) fn declare(
    schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<
        SuccessorSchema,
    >,
) -> worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<
    SuccessorSchema,
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
        .relation(
            UnusedAccountPeer::reference(),
            Account::reference(),
            Account::reference(),
        )
        .principal_binding(IdentityBinding::reference())
        .entity(LaterRecord::reference())
        .aspect(LaterRecord::reference(), LaterIdentity::reference())
        .field(LaterRecord::reference(), Label::reference())
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
                Account::reference(),
                WorthQueryApplicationEntityKey::new("account-row").unwrap(),
            )
            .field(AccountNumber::reference(), "account-001".to_string())
            .field(super::native_mutation::Balance::reference(), 7),
        )
        .unwrap();
    Ok(())
}
