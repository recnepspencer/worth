//! The emergency-access activity rows: one estate, its emergency accesses and
//! each access's mandatory review, keeping the slot identities on the wire.
use worth_query_decl::facade::WorthQueryProjection;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

use crate::{
    estate::{
        EmergencyAccessId, EmergencyAccessReason, EmergencyAccessStatus, EstateCaseId,
        EstateMoment, MandatoryReviewId, MandatoryReviewStatus,
    },
    queries::UnitQueryResultBinding,
    schema::{
        BankSchema, EmergencyAccess, EmergencyAccessExpiresAtField, EmergencyAccessIdentityField,
        EmergencyAccessIssuedAtField, EmergencyAccessReasonField, EmergencyAccessStatusField,
        EmergencyEstate, EmergencyReview, EstateCase, EstateCaseIdentityField, MandatoryReview,
        MandatoryReviewIdentityField, MandatoryReviewStatusField,
    },
};

use super::{
    EstateEmergencyAccessActivity, EstateEmergencyAccessActivityItem,
    EstateEmergencyAccessActivityQuery,
};

#[derive(WorthQueryProjection)]
#[worth_query(schema = BankSchema, root = EstateCase)]
pub(super) struct EstateRow {
    #[worth_query(field = EstateCaseIdentityField, slot = "EstateIdSlot")]
    estate: EstateCaseId,
    #[worth_query(relation = EmergencyEstate, reverse_from = EmergencyAccess,
        nested = UnitQueryResultBinding, slot = "EstateAccessesSlot")]
    accesses: Vec<AccessRow>,
}

#[derive(WorthQueryProjection)]
#[worth_query(schema = BankSchema, root = EmergencyAccess)]
pub(super) struct AccessRow {
    #[worth_query(field = EmergencyAccessIdentityField, slot = "AccessIdSlot")]
    access: EmergencyAccessId,
    #[worth_query(field = EmergencyAccessReasonField, slot = "AccessReasonSlot")]
    reason: EmergencyAccessReason,
    #[worth_query(field = EmergencyAccessStatusField, slot = "AccessStatusSlot")]
    status: EmergencyAccessStatus,
    #[worth_query(field = EmergencyAccessIssuedAtField, slot = "AccessIssuedAtSlot")]
    issued_at: EstateMoment,
    #[worth_query(field = EmergencyAccessExpiresAtField, slot = "AccessExpiresAtSlot")]
    expires_at: EstateMoment,
    #[worth_query(relation = EmergencyReview, nested = UnitQueryResultBinding,
        slot = "AccessReviewSlot")]
    review: ReviewRow,
}

#[derive(WorthQueryProjection)]
#[worth_query(schema = BankSchema, root = MandatoryReview)]
pub(super) struct ReviewRow {
    #[worth_query(field = MandatoryReviewIdentityField, slot = "ReviewIdSlot")]
    review: MandatoryReviewId,
    #[worth_query(field = MandatoryReviewStatusField, slot = "ReviewStatusSlot")]
    status: MandatoryReviewStatus,
}

impl WorthQueryApplicationProjection<BankSchema, EstateEmergencyAccessActivityQuery>
    for EstateEmergencyAccessActivity
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<
            '_,
            BankSchema,
            EstateEmergencyAccessActivityQuery,
        >,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let EstateRow { estate, accesses } = EstateRow::project_row(row)?;
        let accesses = accesses
            .into_iter()
            .map(|row| EstateEmergencyAccessActivityItem {
                access: row.access,
                reason: row.reason,
                status: row.status,
                issued_at: row.issued_at,
                expires_at: row.expires_at,
                review: row.review.review,
                review_status: row.review.status,
            });
        Ok(Self::from_projection(estate, accesses.collect()))
    }
}
