//! Capability rows of the bank model.
use super::*;
pub struct CapabilityGrantRecord;
impl fragment::Aspect for CapabilityGrantRecord {
    const NAME: &'static str = "CapabilityGrantRecord";
    type Row = CapabilityGrantRecordValue;
}
impl CapabilityGrantRecord {
    pub fn reference() -> AspectSelection<EstateRecords, CapabilityGrant, Self> {
        aspect(EstateRecords, CapabilityGrant, Self)
    }
}
pub struct CapabilityGrantRecordValue {
    pub capability_grant_identity_field: CapabilityGrantId,
    pub capability_operation_field: EstateCapabilityOperation,
    pub capability_purpose_field: EstateCapabilityPurpose,
    pub capability_disclosure_field: Option<RestrictedBankField>,
    pub capability_amount_ceiling_field: Option<Money<USD>>,
    pub capability_valid_from_field: EstateMoment,
    pub capability_valid_through_field: EstateMoment,
    pub capability_delegation_limit_field: DelegationLimit,
    pub capability_workflow_stage_field: EstateWorkflowStage,
    pub capability_grant_status_field: CapabilityGrantStatus,
}
impl fragment::Record<CapabilityGrantRecordValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<CapabilityGrantRecordValue, Self> {
        fragment::field(
            "CapabilityGrantIdentityField",
            |row: &CapabilityGrantRecordValue| row.capability_grant_identity_field,
        )
        .equality_queryable(true)
        .unique()
        .and(
            "CapabilityOperationField",
            |row: &CapabilityGrantRecordValue| row.capability_operation_field,
        )
        .equality_queryable(true)
        .and(
            "CapabilityPurposeField",
            |row: &CapabilityGrantRecordValue| row.capability_purpose_field,
        )
        .equality_queryable(true)
        .and(
            "CapabilityDisclosureField",
            |row: &CapabilityGrantRecordValue| row.capability_disclosure_field,
        )
        .equality_queryable(true)
        .and(
            "CapabilityAmountCeilingField",
            |row: &CapabilityGrantRecordValue| row.capability_amount_ceiling_field,
        )
        .and(
            "CapabilityValidFromField",
            |row: &CapabilityGrantRecordValue| row.capability_valid_from_field,
        )
        .equality_queryable(true)
        .and(
            "CapabilityValidThroughField",
            |row: &CapabilityGrantRecordValue| row.capability_valid_through_field,
        )
        .equality_queryable(true)
        .and(
            "CapabilityDelegationLimitField",
            |row: &CapabilityGrantRecordValue| row.capability_delegation_limit_field,
        )
        .equality_queryable(true)
        .and(
            "CapabilityWorkflowStageField",
            |row: &CapabilityGrantRecordValue| row.capability_workflow_stage_field,
        )
        .equality_queryable(true)
        .and(
            "CapabilityGrantStatusField",
            |row: &CapabilityGrantRecordValue| row.capability_grant_status_field,
        )
        .equality_queryable(true)
        .build(
            |capability_grant_identity_field,
             capability_operation_field,
             capability_purpose_field,
             capability_disclosure_field,
             capability_amount_ceiling_field,
             capability_valid_from_field,
             capability_valid_through_field,
             capability_delegation_limit_field,
             capability_workflow_stage_field,
             capability_grant_status_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(CapabilityGrantRecordValue {
                    capability_grant_identity_field,
                    capability_operation_field,
                    capability_purpose_field,
                    capability_disclosure_field,
                    capability_amount_ceiling_field,
                    capability_valid_from_field,
                    capability_valid_through_field,
                    capability_delegation_limit_field,
                    capability_workflow_stage_field,
                    capability_grant_status_field,
                })
            },
        )
    }
}
pub struct EmergencyAccessRecord;
impl fragment::Aspect for EmergencyAccessRecord {
    const NAME: &'static str = "EmergencyAccessRecord";
    type Row = EmergencyAccessRecordValue;
}
impl EmergencyAccessRecord {
    pub fn reference() -> AspectSelection<EstateRecords, EmergencyAccess, Self> {
        aspect(EstateRecords, EmergencyAccess, Self)
    }
}
pub struct EmergencyAccessRecordValue {
    pub emergency_access_identity_field: EmergencyAccessId,
    pub emergency_access_reason_field: EmergencyAccessReason,
    pub emergency_access_status_field: EmergencyAccessStatus,
    pub emergency_access_issued_at_field: EstateMoment,
    pub emergency_access_expires_at_field: EstateMoment,
    pub emergency_access_closed_at_field: Option<EstateMoment>,
}
impl fragment::Record<EmergencyAccessRecordValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<EmergencyAccessRecordValue, Self> {
        fragment::field(
            "EmergencyAccessIdentityField",
            |row: &EmergencyAccessRecordValue| row.emergency_access_identity_field,
        )
        .equality_queryable(true)
        .and(
            "EmergencyAccessReasonField",
            |row: &EmergencyAccessRecordValue| row.emergency_access_reason_field,
        )
        .equality_queryable(true)
        .and(
            "EmergencyAccessStatusField",
            |row: &EmergencyAccessRecordValue| row.emergency_access_status_field,
        )
        .equality_queryable(true)
        .and(
            "EmergencyAccessIssuedAtField",
            |row: &EmergencyAccessRecordValue| row.emergency_access_issued_at_field,
        )
        .equality_queryable(true)
        .and(
            "EmergencyAccessExpiresAtField",
            |row: &EmergencyAccessRecordValue| row.emergency_access_expires_at_field,
        )
        .equality_queryable(true)
        .and(
            "EmergencyAccessClosedAtField",
            |row: &EmergencyAccessRecordValue| row.emergency_access_closed_at_field,
        )
        .equality_queryable(true)
        .build(
            |emergency_access_identity_field,
             emergency_access_reason_field,
             emergency_access_status_field,
             emergency_access_issued_at_field,
             emergency_access_expires_at_field,
             emergency_access_closed_at_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(EmergencyAccessRecordValue {
                    emergency_access_identity_field,
                    emergency_access_reason_field,
                    emergency_access_status_field,
                    emergency_access_issued_at_field,
                    emergency_access_expires_at_field,
                    emergency_access_closed_at_field,
                })
            },
        )
    }
}
pub struct MandatoryReviewRecord;
impl fragment::Aspect for MandatoryReviewRecord {
    const NAME: &'static str = "MandatoryReviewRecord";
    type Row = MandatoryReviewRecordValue;
}
impl MandatoryReviewRecord {
    pub fn reference() -> AspectSelection<EstateRecords, MandatoryReview, Self> {
        aspect(EstateRecords, MandatoryReview, Self)
    }
}
pub struct MandatoryReviewRecordValue {
    pub mandatory_review_identity_field: MandatoryReviewId,
    pub mandatory_review_status_field: MandatoryReviewStatus,
    pub mandatory_review_reviewed_at_field: Option<EstateMoment>,
    pub mandatory_review_kind_field: MandatoryReviewKind,
}
impl fragment::Record<MandatoryReviewRecordValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<MandatoryReviewRecordValue, Self> {
        fragment::field(
            "MandatoryReviewIdentityField",
            |row: &MandatoryReviewRecordValue| row.mandatory_review_identity_field,
        )
        .equality_queryable(true)
        .and(
            "MandatoryReviewStatusField",
            |row: &MandatoryReviewRecordValue| row.mandatory_review_status_field,
        )
        .equality_queryable(true)
        .and(
            "MandatoryReviewReviewedAtField",
            |row: &MandatoryReviewRecordValue| row.mandatory_review_reviewed_at_field,
        )
        .equality_queryable(true)
        .and(
            "MandatoryReviewKindField",
            |row: &MandatoryReviewRecordValue| row.mandatory_review_kind_field,
        )
        .equality_queryable(true)
        .build(
            |mandatory_review_identity_field,
             mandatory_review_status_field,
             mandatory_review_reviewed_at_field,
             mandatory_review_kind_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MandatoryReviewRecordValue {
                    mandatory_review_identity_field,
                    mandatory_review_status_field,
                    mandatory_review_reviewed_at_field,
                    mandatory_review_kind_field,
                })
            },
        )
    }
}
