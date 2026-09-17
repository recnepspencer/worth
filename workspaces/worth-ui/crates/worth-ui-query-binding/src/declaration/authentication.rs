use worth_query_decl::facade::application_schema::U64ApplicationValueBinding;
use worth_query_decl::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};
use worth_query_decl::facade::{
    worth_query_aspect, worth_query_entity, worth_query_field, worth_query_principal_binding,
    worth_query_relation,
};

use super::WorthUiApplicationSchema;

worth_query_entity!(pub WorthUiExternalPrincipalMapping for WorthUiApplicationSchema);
worth_query_entity!(pub WorthUiPrincipal for WorthUiApplicationSchema);
worth_query_aspect!(pub WorthUiExternalIdentity for WorthUiApplicationSchema,
    WorthUiExternalPrincipalMapping; identity = AspectIdentity(0x9161105c), revision = AspectContractRevision(1),);
worth_query_aspect!(pub WorthUiPrincipalIdentity for WorthUiApplicationSchema,
    WorthUiPrincipal; identity = AspectIdentity(0x9161105d), revision = AspectContractRevision(1),);
worth_query_field!(pub WorthUiExternalIdentityKey for WorthUiApplicationSchema,
    WorthUiExternalPrincipalMapping, WorthUiExternalIdentity:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding,
    read_only, equality);
worth_query_field!(pub WorthUiMappingStatus for WorthUiApplicationSchema,
    WorthUiExternalPrincipalMapping, WorthUiExternalIdentity:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding,
    read_write, equality);
worth_query_field!(pub WorthUiPrincipalId for WorthUiApplicationSchema,
    WorthUiPrincipal, WorthUiPrincipalIdentity:
    u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_relation!(pub WorthUiExternalPrincipal in WorthUiApplicationSchema,
    WorthUiExternalPrincipalMapping => WorthUiPrincipal;
    integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(
    pub WorthUiPrincipalBinding in WorthUiApplicationSchema,
    mapping WorthUiExternalPrincipalMapping {
        identity: WorthUiExternalIdentityKey,
        status: WorthUiMappingStatus,
        target: WorthUiExternalPrincipal => WorthUiPrincipal,
        principal_identity: WorthUiPrincipalId
    }
);
