use worth_query_decl::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};
use worth_query_decl::facade::{
    worth_query_aspect, worth_query_field, worth_query_principal_binding,
};

use crate::model::BankPrincipalId;

use super::entities::{ExternalPrincipalMapping, Principal};
use super::relations::ExternalPrincipal;
use super::{BankPrincipalIdBinding, BankSchema};

worth_query_aspect!(pub ExternalPrincipalIdentity for BankSchema,
    ExternalPrincipalMapping; identity = AspectIdentity(0x91611001), revision = AspectContractRevision(1),);
worth_query_field!(
    pub ExternalIdentityKey for BankSchema,
    ExternalPrincipalMapping,
    ExternalPrincipalIdentity:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality
);
worth_query_field!(
    pub ExternalMappingStatus for BankSchema,
    ExternalPrincipalMapping,
    ExternalPrincipalIdentity:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality
);
worth_query_aspect!(pub PrincipalIdentity for BankSchema, Principal; identity = AspectIdentity(0x91611002), revision = AspectContractRevision(1),);
worth_query_field!(
    pub PrincipalIdentityField for BankSchema,
    Principal,
    PrincipalIdentity:
    BankPrincipalId => BankPrincipalIdBinding, read_only, equality
);
worth_query_principal_binding!(
    pub BankPrincipalBinding in BankSchema,
    mapping ExternalPrincipalMapping {
        identity: ExternalIdentityKey,
        status: ExternalMappingStatus,
        target: ExternalPrincipal => Principal,
        principal_identity: PrincipalIdentityField
    }
);
