//! The mandatory-review result rows: one estate, its reviews and each
//! review's optional reviewer, keeping the slot identities already on the wire.
use worth_query_decl::facade::WorthQueryProjection;

use crate::{
    estate::{EstateCaseId, MandatoryReviewId, MandatoryReviewKind, MandatoryReviewStatus},
    model::BankPrincipalId,
    queries::UnitQueryResultBinding,
    reads::EstateReviewView,
    schema::{
        BankSchema, EstateCase, EstateCaseIdentityField, MandatoryReview,
        MandatoryReviewIdentityField, MandatoryReviewKindField, MandatoryReviewStatusField,
        Principal, PrincipalIdentityField, ReviewEstate, ReviewPrincipal,
    },
};

#[derive(WorthQueryProjection)]
#[worth_query(schema = BankSchema, root = EstateCase)]
pub(super) struct EstateRow {
    #[worth_query(field = EstateCaseIdentityField, slot = "EstateIdentitySlot")]
    pub(super) estate: EstateCaseId,
    #[worth_query(relation = ReviewEstate, reverse_from = MandatoryReview,
        nested = UnitQueryResultBinding, slot = "ReviewsSlot")]
    pub(super) reviews: Vec<ReviewRow>,
}

#[derive(WorthQueryProjection)]
#[worth_query(schema = BankSchema, root = MandatoryReview)]
pub(super) struct ReviewRow {
    #[worth_query(field = MandatoryReviewIdentityField, slot = "ReviewIdentitySlot")]
    review: MandatoryReviewId,
    #[worth_query(field = MandatoryReviewKindField, slot = "ReviewKindSlot")]
    kind: MandatoryReviewKind,
    #[worth_query(field = MandatoryReviewStatusField, slot = "ReviewStatusSlot")]
    status: MandatoryReviewStatus,
    #[worth_query(relation = ReviewPrincipal, reverse_from = Principal,
        nested = UnitQueryResultBinding, slot = "ReviewPrincipalSlot")]
    reviewer: Option<ReviewerRow>,
}

#[derive(WorthQueryProjection)]
#[worth_query(schema = BankSchema, root = Principal)]
pub(super) struct ReviewerRow {
    #[worth_query(field = PrincipalIdentityField, slot = "ReviewPrincipalIdentitySlot")]
    reviewer: BankPrincipalId,
}

impl From<ReviewRow> for EstateReviewView {
    fn from(row: ReviewRow) -> Self {
        Self::from_projection(
            row.review,
            row.kind,
            row.status,
            row.reviewer.map(|reviewer| reviewer.reviewer),
        )
    }
}
