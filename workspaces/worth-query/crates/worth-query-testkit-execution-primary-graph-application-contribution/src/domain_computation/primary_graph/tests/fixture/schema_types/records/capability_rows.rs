//! Capability rows of the execution fixture.
use super::*;
pub struct CapabilityElevationFacts;
impl fragment::Aspect for CapabilityElevationFacts {
    const NAME: &'static str = "CapabilityElevationFacts";
    type Row = CapabilityElevationFactsValue;
}
impl CapabilityElevationFacts {
    pub fn reference() -> AspectSelection<ElevationRecords, CapabilityElevation, Self> {
        aspect(ElevationRecords, CapabilityElevation, Self)
    }
}
pub struct CapabilityElevationFactsValue {
    pub capability_elevation_identity: ElevationKey,
    pub capability_elevation_reason: String,
    pub capability_elevation_status_field: CapabilityElevationStatus,
    pub capability_elevation_not_before: u64,
    pub capability_elevation_not_after: u64,
    pub capability_elevation_closed_at: Option<u64>,
}
impl fragment::Record<CapabilityElevationFactsValue> for ElevationBindings {
    fn record() -> impl fragment::Shape<CapabilityElevationFactsValue, Self> {
        fragment::field(
            "CapabilityElevationIdentity",
            |row: &CapabilityElevationFactsValue| row.capability_elevation_identity.clone(),
        )
        .equality_queryable(true)
        .read_only()
        .and(
            "CapabilityElevationReason",
            |row: &CapabilityElevationFactsValue| row.capability_elevation_reason.clone(),
        )
        .read_only()
        .and(
            "CapabilityElevationStatusField",
            |row: &CapabilityElevationFactsValue| row.capability_elevation_status_field,
        )
        .and(
            "CapabilityElevationNotBefore",
            |row: &CapabilityElevationFactsValue| row.capability_elevation_not_before,
        )
        .and(
            "CapabilityElevationNotAfter",
            |row: &CapabilityElevationFactsValue| row.capability_elevation_not_after,
        )
        .and(
            "CapabilityElevationClosedAt",
            |row: &CapabilityElevationFactsValue| row.capability_elevation_closed_at,
        )
        .build(
            |capability_elevation_identity,
             capability_elevation_reason,
             capability_elevation_status_field,
             capability_elevation_not_before,
             capability_elevation_not_after,
             capability_elevation_closed_at,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(CapabilityElevationFactsValue {
                    capability_elevation_identity,
                    capability_elevation_reason,
                    capability_elevation_status_field,
                    capability_elevation_not_before,
                    capability_elevation_not_after,
                    capability_elevation_closed_at,
                })
            },
        )
    }
}
pub struct CapabilityReviewFacts;
impl fragment::Aspect for CapabilityReviewFacts {
    const NAME: &'static str = "CapabilityReviewFacts";
    type Row = CapabilityReviewFactsValue;
}
impl CapabilityReviewFacts {
    pub fn reference() -> AspectSelection<ExecutionRecords, CapabilityReview, Self> {
        aspect(ExecutionRecords, CapabilityReview, Self)
    }
}
pub struct CapabilityReviewFactsValue {
    pub capability_review_identity: String,
    pub capability_review_kind_field: CapabilityReviewKind,
    pub capability_review_status_field: CapabilityReviewStatus,
    pub capability_review_reviewed_at: Option<u64>,
}
impl fragment::Record<CapabilityReviewFactsValue> for ExecutionRecordsBindings {
    fn record() -> impl fragment::Shape<CapabilityReviewFactsValue, Self> {
        fragment::field(
            "CapabilityReviewIdentity",
            |row: &CapabilityReviewFactsValue| row.capability_review_identity.clone(),
        )
        .equality_queryable(true)
        .read_only()
        .and(
            "CapabilityReviewKindField",
            |row: &CapabilityReviewFactsValue| row.capability_review_kind_field,
        )
        .equality_queryable(true)
        .and(
            "CapabilityReviewStatusField",
            |row: &CapabilityReviewFactsValue| row.capability_review_status_field,
        )
        .and(
            "CapabilityReviewReviewedAt",
            |row: &CapabilityReviewFactsValue| row.capability_review_reviewed_at,
        )
        .build(
            |capability_review_identity,
             capability_review_kind_field,
             capability_review_status_field,
             capability_review_reviewed_at,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(CapabilityReviewFactsValue {
                    capability_review_identity,
                    capability_review_kind_field,
                    capability_review_status_field,
                    capability_review_reviewed_at,
                })
            },
        )
    }
}
pub struct CapabilityFacts;
impl fragment::Aspect for CapabilityFacts {
    const NAME: &'static str = "CapabilityFacts";
    type Row = CapabilityFactsValue;
}
impl CapabilityFacts {
    pub fn reference() -> AspectSelection<GrantRecords, CapabilityGrant, Self> {
        aspect(GrantRecords, CapabilityGrant, Self)
    }
}
pub struct CapabilityFactsValue {
    pub capability_identity: GrantKey,
    pub capability_action_field: CapabilityAction,
    pub capability_purpose_field: CapabilityPurpose,
    pub capability_disclosure_field: CapabilityDisclosure,
    pub capability_amount_field: u64,
    pub capability_status_field: CapabilityStatus,
    pub capability_workflow_field: String,
    pub capability_not_before_field: u64,
    pub capability_not_after_field: u64,
    pub capability_delegation_limit_field: u64,
}
impl fragment::Record<CapabilityFactsValue> for GrantBindings {
    fn record() -> impl fragment::Shape<CapabilityFactsValue, Self> {
        fragment::field("CapabilityIdentity", |row: &CapabilityFactsValue| {
            row.capability_identity.clone()
        })
        .equality_queryable(true)
        .read_only()
        .and("CapabilityActionField", |row: &CapabilityFactsValue| {
            row.capability_action_field
        })
        .and("CapabilityPurposeField", |row: &CapabilityFactsValue| {
            row.capability_purpose_field
        })
        .and("CapabilityDisclosureField", |row: &CapabilityFactsValue| {
            row.capability_disclosure_field
        })
        .and("CapabilityAmountField", |row: &CapabilityFactsValue| {
            row.capability_amount_field
        })
        .and("CapabilityStatusField", |row: &CapabilityFactsValue| {
            row.capability_status_field
        })
        .and("CapabilityWorkflowField", |row: &CapabilityFactsValue| {
            row.capability_workflow_field.clone()
        })
        .and("CapabilityNotBeforeField", |row: &CapabilityFactsValue| {
            row.capability_not_before_field
        })
        .and("CapabilityNotAfterField", |row: &CapabilityFactsValue| {
            row.capability_not_after_field
        })
        .and(
            "CapabilityDelegationLimitField",
            |row: &CapabilityFactsValue| row.capability_delegation_limit_field,
        )
        .build(
            |capability_identity,
             capability_action_field,
             capability_purpose_field,
             capability_disclosure_field,
             capability_amount_field,
             capability_status_field,
             capability_workflow_field,
             capability_not_before_field,
             capability_not_after_field,
             capability_delegation_limit_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(CapabilityFactsValue {
                    capability_identity,
                    capability_action_field,
                    capability_purpose_field,
                    capability_disclosure_field,
                    capability_amount_field,
                    capability_status_field,
                    capability_workflow_field,
                    capability_not_before_field,
                    capability_not_after_field,
                    capability_delegation_limit_field,
                })
            },
        )
    }
}
pub struct CapabilityActionRecordFacts;
impl fragment::Aspect for CapabilityActionRecordFacts {
    const NAME: &'static str = "CapabilityActionRecordFacts";
    type Row = CapabilityActionRecordFactsValue;
}
impl CapabilityActionRecordFacts {
    pub fn reference() -> AspectSelection<ExecutionRecords, CapabilityActionRecord, Self> {
        aspect(ExecutionRecords, CapabilityActionRecord, Self)
    }
}
pub struct CapabilityActionRecordFactsValue {
    pub capability_action_record_identity: String,
}
impl fragment::Record<CapabilityActionRecordFactsValue> for ExecutionRecordsBindings {
    fn record() -> impl fragment::Shape<CapabilityActionRecordFactsValue, Self> {
        fragment::field(
            "CapabilityActionRecordIdentity",
            |row: &CapabilityActionRecordFactsValue| row.capability_action_record_identity.clone(),
        )
        .equality_queryable(true)
        .read_only()
        .build(
            |capability_action_record_identity, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(CapabilityActionRecordFactsValue {
                    capability_action_record_identity,
                })
            },
        )
    }
}
