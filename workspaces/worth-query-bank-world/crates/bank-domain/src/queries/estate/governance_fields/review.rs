use worth_query_decl::facade::{
    application_query::ApplicationQueryResultFieldRef,
    application_schema::{EqualityPredicate, NoApplicationUnit, ReadOnly, ReadWrite},
};

use crate::{
    estate::{EstateCaseId, MandatoryReviewId, MandatoryReviewKind, MandatoryReviewStatus},
    model::BankPrincipalId,
    schema::{
        BankSchema, EstateCase, EstateCaseIdentityField, EstateCaseRecord, MandatoryReview,
        MandatoryReviewIdentityField, MandatoryReviewKindField, MandatoryReviewRecord,
        MandatoryReviewStatusField, Principal, PrincipalIdentity, PrincipalIdentityField,
    },
};

use super::super::governance::EstateGovernanceQuery;

macro_rules! slots {
    ($($name:ident),+ $(,)?) => {
        $(
            pub(in crate::queries::estate) struct $name;
            impl worth_query_decl::facade::portable_identity::WorthQueryPortableType for $name {
                const PORTABLE_TYPE_NAME: &'static str = stringify!($name);
            }
        )+
    };
}

slots!(
    ReviewIdSlot,
    ReviewKindSlot,
    ReviewStatusSlot,
    ReviewEstateIdentitySlot,
    ReviewReviewerSlot,
);

macro_rules! selector {
    ($name:ident, $slot:ty, $entity:ty, $aspect:ty, $field:ty, $value:ty, $write:ty, $alias:literal) => {
        pub(in crate::queries::estate) fn $name() -> ApplicationQueryResultFieldRef<
            EstateGovernanceQuery,
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
    review_id,
    ReviewIdSlot,
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
    review_estate_identity,
    ReviewEstateIdentitySlot,
    EstateCase,
    EstateCaseRecord,
    EstateCaseIdentityField,
    EstateCaseId,
    ReadOnly,
    "estate"
);
selector!(
    review_reviewer_identity,
    ReviewReviewerSlot,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityField,
    BankPrincipalId,
    ReadOnly,
    "reviewer"
);
