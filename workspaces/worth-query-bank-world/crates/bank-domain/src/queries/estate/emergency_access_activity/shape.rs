use worth_query_decl::facade::application_query::{
    ApplicationQueryResultShapeBuilder, TypedApplicationQueryResultShape,
};

use crate::schema::{BankSchema, EmergencyAccess, EstateCase, MandatoryReview};

use super::{
    selectors::{
        access_expires_at, access_id, access_issued_at, access_reason, access_review,
        access_status, estate_accesses, estate_id, review_id, review_status,
    },
    EstateEmergencyAccessActivity, EstateEmergencyAccessActivityQuery,
};

pub(super) fn emergency_access_activity_shape() -> TypedApplicationQueryResultShape<
    BankSchema,
    EstateEmergencyAccessActivityQuery,
    EstateCase,
    EstateEmergencyAccessActivity,
    super::EstateEmergencyAccessActivityQueryResultBinding,
> {
    let review = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        EstateEmergencyAccessActivityQuery,
        MandatoryReview,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(MandatoryReview::reference())
    .field(review_id())
    .field(review_status());
    let access = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        EstateEmergencyAccessActivityQuery,
        EmergencyAccess,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(EmergencyAccess::reference())
    .field(access_id())
    .field(access_reason())
    .field(access_status())
    .field(access_issued_at())
    .field(access_expires_at())
    .relation(access_review(), review);
    ApplicationQueryResultShapeBuilder::new(EstateCase::reference())
        .field(estate_id())
        .relation(estate_accesses(), access)
        .build()
}
