use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef, ManyResults,
        OptionalOneResult, ReverseResultTraversal,
    },
    application_schema::{EqualityPredicate, NoApplicationUnit, ReadOnly, ReadWrite},
};

use crate::{
    estate::{EstateCaseId, MandatoryReviewId, MandatoryReviewKind, MandatoryReviewStatus},
    model::BankPrincipalId,
    schema::{
        BankSchema, EstateCase, EstateCaseIdentityField, EstateCaseRecord, MandatoryReview,
        MandatoryReviewIdentityField, MandatoryReviewKindField, MandatoryReviewRecord,
        MandatoryReviewStatusField, Principal, PrincipalIdentity, PrincipalIdentityField,
        ReviewEstate, ReviewPrincipal,
    },
};

use super::mandatory_review::EstateMandatoryReviewQuery;

pub(super) struct EstateIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(EstateIdentitySlot => "EstateIdentitySlot");
pub(super) struct ReviewsSlot;
worth_query_decl::facade::worth_query_portable_type!(ReviewsSlot => "ReviewsSlot");
pub(super) struct ReviewIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(ReviewIdentitySlot => "ReviewIdentitySlot");
pub(super) struct ReviewKindSlot;
worth_query_decl::facade::worth_query_portable_type!(ReviewKindSlot => "ReviewKindSlot");
pub(super) struct ReviewStatusSlot;
worth_query_decl::facade::worth_query_portable_type!(ReviewStatusSlot => "ReviewStatusSlot");
pub(super) struct ReviewPrincipalSlot;
worth_query_decl::facade::worth_query_portable_type!(ReviewPrincipalSlot => "ReviewPrincipalSlot");
pub(super) struct ReviewPrincipalIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(ReviewPrincipalIdentitySlot => "ReviewPrincipalIdentitySlot");

macro_rules! selector {
    ($name:ident, $slot:ty, $entity:ty, $aspect:ty, $field:ty, $value:ty, $write:ty, $alias:literal) => {
        pub(super) fn $name() -> ApplicationQueryResultFieldRef<
            EstateMandatoryReviewQuery,
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
    review_identity,
    ReviewIdentitySlot,
    MandatoryReview,
    MandatoryReviewRecord,
    MandatoryReviewIdentityField,
    MandatoryReviewId,
    ReadOnly,
    "review"
);
selector!(
    review_kind,
    ReviewKindSlot,
    MandatoryReview,
    MandatoryReviewRecord,
    MandatoryReviewKindField,
    MandatoryReviewKind,
    ReadOnly,
    "kind"
);
selector!(
    review_status,
    ReviewStatusSlot,
    MandatoryReview,
    MandatoryReviewRecord,
    MandatoryReviewStatusField,
    MandatoryReviewStatus,
    ReadWrite,
    "status"
);
selector!(
    review_principal_identity,
    ReviewPrincipalIdentitySlot,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityField,
    BankPrincipalId,
    ReadOnly,
    "reviewer"
);

pub(super) fn estate_reviews() -> ApplicationQueryResultRelationRef<
    EstateMandatoryReviewQuery,
    ReviewsSlot,
    BankSchema,
    ReviewEstate,
    MandatoryReview,
    EstateCase,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many("reviews", ReviewEstate::reference())
}

pub(super) fn review_principal() -> ApplicationQueryResultRelationRef<
    EstateMandatoryReviewQuery,
    ReviewPrincipalSlot,
    BankSchema,
    ReviewPrincipal,
    Principal,
    MandatoryReview,
    ReverseResultTraversal,
    OptionalOneResult,
> {
    ApplicationQueryResultRelationRef::reverse_optional("reviewer", ReviewPrincipal::reference())
}
