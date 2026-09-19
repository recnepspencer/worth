use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef, ExactlyOneResult,
        ForwardResultTraversal, ManyResults, ReverseResultTraversal,
    },
    application_schema::{EqualityPredicate, NoApplicationUnit, ReadOnly, ReadWrite},
};

use crate::{
    estate::{EstateCaseId, LegalAuthorityId, LegalAuthorityKind},
    model::BankPrincipalId,
    schema::{
        BankSchema, EstateCase, EstateCaseIdentityField, EstateCaseRecord, LegalAuthority,
        LegalAuthorityEstate, LegalAuthorityHolder, LegalAuthorityIdentityField,
        LegalAuthorityKindField, LegalAuthorityRecognizedField, LegalAuthorityRecord, Principal,
        PrincipalIdentity, PrincipalIdentityField,
    },
};

use super::legal_compliance::EstateLegalComplianceQuery;

pub(super) struct EstateIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(EstateIdentitySlot => "EstateIdentitySlot");
pub(super) struct AuthoritiesSlot;
worth_query_decl::facade::worth_query_portable_type!(AuthoritiesSlot => "AuthoritiesSlot");
pub(super) struct AuthorityIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(AuthorityIdentitySlot => "AuthorityIdentitySlot");
pub(super) struct AuthorityKindSlot;
worth_query_decl::facade::worth_query_portable_type!(AuthorityKindSlot => "AuthorityKindSlot");
pub(super) struct AuthorityRecognizedSlot;
worth_query_decl::facade::worth_query_portable_type!(AuthorityRecognizedSlot => "AuthorityRecognizedSlot");
pub(super) struct AuthorityHolderSlot;
worth_query_decl::facade::worth_query_portable_type!(AuthorityHolderSlot => "AuthorityHolderSlot");
pub(super) struct AuthorityHolderIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(AuthorityHolderIdentitySlot => "AuthorityHolderIdentitySlot");

macro_rules! selector {
    ($name:ident, $slot:ty, $entity:ty, $aspect:ty, $field:ty, $value:ty, $write:ty, $alias:literal) => {
        pub(super) fn $name() -> ApplicationQueryResultFieldRef<
            EstateLegalComplianceQuery,
            $slot,
            BankSchema,
            $entity,
            $aspect,
            $field,
            $value,
            $write,
            EqualityPredicate,
            NoApplicationUnit,
        > {
            ApplicationQueryResultFieldRef::new($alias, <$field>::reference())
        }
    };
}

selector!(
    estate_identity,
    EstateIdentitySlot,
    EstateCase,
    EstateCaseRecord,
    EstateCaseIdentityField,
    EstateCaseId,
    ReadOnly,
    "estate"
);
selector!(
    authority_identity,
    AuthorityIdentitySlot,
    LegalAuthority,
    LegalAuthorityRecord,
    LegalAuthorityIdentityField,
    LegalAuthorityId,
    ReadOnly,
    "authority"
);
selector!(
    authority_kind,
    AuthorityKindSlot,
    LegalAuthority,
    LegalAuthorityRecord,
    LegalAuthorityKindField,
    LegalAuthorityKind,
    ReadWrite,
    "kind"
);
selector!(
    authority_recognized,
    AuthorityRecognizedSlot,
    LegalAuthority,
    LegalAuthorityRecord,
    LegalAuthorityRecognizedField,
    bool,
    ReadWrite,
    "recognized"
);
selector!(
    authority_holder_identity,
    AuthorityHolderIdentitySlot,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityField,
    BankPrincipalId,
    ReadOnly,
    "holder"
);

pub(super) fn estate_authorities() -> ApplicationQueryResultRelationRef<
    EstateLegalComplianceQuery,
    AuthoritiesSlot,
    BankSchema,
    LegalAuthorityEstate,
    LegalAuthority,
    EstateCase,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many(
        "legal_authorities",
        LegalAuthorityEstate::reference(),
    )
}

pub(super) fn authority_holder() -> ApplicationQueryResultRelationRef<
    EstateLegalComplianceQuery,
    AuthorityHolderSlot,
    BankSchema,
    LegalAuthorityHolder,
    LegalAuthority,
    Principal,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("holder", LegalAuthorityHolder::reference())
}
