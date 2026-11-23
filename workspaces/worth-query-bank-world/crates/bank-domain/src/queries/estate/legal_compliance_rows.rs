//! The legal-compliance result rows: one estate, its legal authorities and
//! each authority's holder, keeping the slot identities already on the wire.
use worth_query_decl::facade::WorthQueryProjection;

use crate::{
    estate::{EstateCaseId, LegalAuthorityId, LegalAuthorityKind},
    model::BankPrincipalId,
    queries::UnitQueryResultBinding,
    reads::EstateLegalAuthorityView,
    schema::{
        BankSchema, EstateCase, EstateCaseIdentityField, LegalAuthority, LegalAuthorityEstate,
        LegalAuthorityHolder, LegalAuthorityIdentityField, LegalAuthorityKindField,
        LegalAuthorityRecognizedField, Principal, PrincipalIdentityField,
    },
};

#[derive(WorthQueryProjection)]
#[worth_query(schema = BankSchema, root = EstateCase)]
pub(super) struct EstateRow {
    #[worth_query(field = EstateCaseIdentityField, slot = "EstateIdentitySlot")]
    pub(super) estate: EstateCaseId,
    #[worth_query(relation = LegalAuthorityEstate, reverse_from = LegalAuthority,
        nested = UnitQueryResultBinding, name = "legal_authorities", slot = "AuthoritiesSlot")]
    pub(super) authorities: Vec<AuthorityRow>,
}

#[derive(WorthQueryProjection)]
#[worth_query(schema = BankSchema, root = LegalAuthority)]
pub(super) struct AuthorityRow {
    #[worth_query(field = LegalAuthorityIdentityField, slot = "AuthorityIdentitySlot")]
    authority: LegalAuthorityId,
    #[worth_query(field = LegalAuthorityKindField, slot = "AuthorityKindSlot")]
    kind: LegalAuthorityKind,
    #[worth_query(field = LegalAuthorityRecognizedField, slot = "AuthorityRecognizedSlot")]
    recognized: bool,
    #[worth_query(relation = LegalAuthorityHolder, nested = UnitQueryResultBinding,
        slot = "AuthorityHolderSlot")]
    holder: HolderRow,
}

#[derive(WorthQueryProjection)]
#[worth_query(schema = BankSchema, root = Principal)]
pub(super) struct HolderRow {
    #[worth_query(field = PrincipalIdentityField, slot = "AuthorityHolderIdentitySlot")]
    holder: BankPrincipalId,
}

impl From<AuthorityRow> for EstateLegalAuthorityView {
    fn from(row: AuthorityRow) -> Self {
        Self::from_projection(row.authority, row.holder.holder, row.kind, row.recognized)
    }
}
