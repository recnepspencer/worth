use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef, ExactlyOneResult,
        ForwardResultTraversal, ManyResults, ReverseResultTraversal,
    },
    application_schema::{EqualityPredicate, NoApplicationUnit, ReadOnly, ReadWrite},
};

use crate::{
    estate::{
        EmergencyAccessId, EmergencyAccessReason, EmergencyAccessStatus, EstateCaseId,
        EstateMoment, MandatoryReviewId, MandatoryReviewStatus,
    },
    schema::{
        BankSchema, EmergencyAccess, EmergencyAccessExpiresAtField, EmergencyAccessIdentityField,
        EmergencyAccessIssuedAtField, EmergencyAccessReasonField, EmergencyAccessRecord,
        EmergencyAccessStatusField, EmergencyEstate, EmergencyReview, EstateCase,
        EstateCaseIdentityField, EstateCaseRecord, MandatoryReview, MandatoryReviewIdentityField,
        MandatoryReviewRecord, MandatoryReviewStatusField,
    },
};

use super::EstateEmergencyAccessActivityQuery;

pub(super) struct EstateIdSlot;
worth_query_decl::facade::worth_query_portable_type!(EstateIdSlot => "EstateIdSlot");
pub(super) struct EstateAccessesSlot;
worth_query_decl::facade::worth_query_portable_type!(EstateAccessesSlot => "EstateAccessesSlot");
pub(super) struct AccessIdSlot;
worth_query_decl::facade::worth_query_portable_type!(AccessIdSlot => "AccessIdSlot");
pub(super) struct AccessReasonSlot;
worth_query_decl::facade::worth_query_portable_type!(AccessReasonSlot => "AccessReasonSlot");
pub(super) struct AccessStatusSlot;
worth_query_decl::facade::worth_query_portable_type!(AccessStatusSlot => "AccessStatusSlot");
pub(super) struct AccessIssuedAtSlot;
worth_query_decl::facade::worth_query_portable_type!(AccessIssuedAtSlot => "AccessIssuedAtSlot");
pub(super) struct AccessExpiresAtSlot;
worth_query_decl::facade::worth_query_portable_type!(AccessExpiresAtSlot => "AccessExpiresAtSlot");
pub(super) struct AccessReviewSlot;
worth_query_decl::facade::worth_query_portable_type!(AccessReviewSlot => "AccessReviewSlot");
pub(super) struct ReviewIdSlot;
worth_query_decl::facade::worth_query_portable_type!(ReviewIdSlot => "ReviewIdSlot");
pub(super) struct ReviewStatusSlot;
worth_query_decl::facade::worth_query_portable_type!(ReviewStatusSlot => "ReviewStatusSlot");

macro_rules! selector {
    ($name:ident, $slot:ty, $entity:ty, $aspect:ty, $field:ty, $value:ty, $write:ty, $alias:literal) => {
        pub(super) fn $name() -> ApplicationQueryResultFieldRef<
            EstateEmergencyAccessActivityQuery,
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
    estate_id,
    EstateIdSlot,
    EstateCase,
    EstateCaseRecord,
    EstateCaseIdentityField,
    EstateCaseId,
    ReadOnly,
    "estate"
);
selector!(
    access_id,
    AccessIdSlot,
    EmergencyAccess,
    EmergencyAccessRecord,
    EmergencyAccessIdentityField,
    EmergencyAccessId,
    ReadOnly,
    "access"
);
selector!(
    access_reason,
    AccessReasonSlot,
    EmergencyAccess,
    EmergencyAccessRecord,
    EmergencyAccessReasonField,
    EmergencyAccessReason,
    ReadWrite,
    "reason"
);
selector!(
    access_status,
    AccessStatusSlot,
    EmergencyAccess,
    EmergencyAccessRecord,
    EmergencyAccessStatusField,
    EmergencyAccessStatus,
    ReadWrite,
    "status"
);
selector!(
    access_issued_at,
    AccessIssuedAtSlot,
    EmergencyAccess,
    EmergencyAccessRecord,
    EmergencyAccessIssuedAtField,
    EstateMoment,
    ReadWrite,
    "issued_at"
);
selector!(
    access_expires_at,
    AccessExpiresAtSlot,
    EmergencyAccess,
    EmergencyAccessRecord,
    EmergencyAccessExpiresAtField,
    EstateMoment,
    ReadWrite,
    "expires_at"
);
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
    review_status,
    ReviewStatusSlot,
    MandatoryReview,
    MandatoryReviewRecord,
    MandatoryReviewStatusField,
    MandatoryReviewStatus,
    ReadWrite,
    "status"
);

pub(super) fn estate_accesses() -> ApplicationQueryResultRelationRef<
    EstateEmergencyAccessActivityQuery,
    EstateAccessesSlot,
    BankSchema,
    EmergencyEstate,
    EmergencyAccess,
    EstateCase,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many("accesses", EmergencyEstate::reference())
}

pub(super) fn access_review() -> ApplicationQueryResultRelationRef<
    EstateEmergencyAccessActivityQuery,
    AccessReviewSlot,
    BankSchema,
    EmergencyReview,
    EmergencyAccess,
    MandatoryReview,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("review", EmergencyReview::reference())
}
