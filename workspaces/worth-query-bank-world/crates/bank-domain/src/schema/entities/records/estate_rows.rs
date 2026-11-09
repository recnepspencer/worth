//! Estate rows of the bank model.
use super::*;
pub struct BranchIdentity;
impl fragment::Aspect for BranchIdentity {
    const NAME: &'static str = "BranchIdentity";
    type Row = BranchIdentityValue;
}
impl BranchIdentity {
    pub fn reference() -> AspectSelection<EstateRecords, Branch, Self> {
        aspect(EstateRecords, Branch, Self)
    }
}
pub struct BranchIdentityValue {
    pub branch_identity_field: BranchId,
}
impl fragment::Record<BranchIdentityValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<BranchIdentityValue, Self> {
        fragment::field("BranchIdentityField", |row: &BranchIdentityValue| {
            row.branch_identity_field
        })
        .equality_queryable(true)
        .build(|branch_identity_field, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(BranchIdentityValue {
                branch_identity_field,
            })
        })
    }
}
pub struct DeathNoticeRecord;
impl fragment::Aspect for DeathNoticeRecord {
    const NAME: &'static str = "DeathNoticeRecord";
    type Row = DeathNoticeRecordValue;
}
impl DeathNoticeRecord {
    pub fn reference() -> AspectSelection<EstateRecords, DeathNotice, Self> {
        aspect(EstateRecords, DeathNotice, Self)
    }
}
pub struct DeathNoticeRecordValue {
    pub death_notice_identity_field: DeathNoticeId,
    pub death_notice_status_field: DeathNoticeStatus,
}
impl fragment::Record<DeathNoticeRecordValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<DeathNoticeRecordValue, Self> {
        fragment::field(
            "DeathNoticeIdentityField",
            |row: &DeathNoticeRecordValue| row.death_notice_identity_field,
        )
        .equality_queryable(true)
        .and("DeathNoticeStatusField", |row: &DeathNoticeRecordValue| {
            row.death_notice_status_field
        })
        .equality_queryable(true)
        .build(
            |death_notice_identity_field,
             death_notice_status_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(DeathNoticeRecordValue {
                    death_notice_identity_field,
                    death_notice_status_field,
                })
            },
        )
    }
}
pub struct EstateCaseRecord;
impl fragment::Aspect for EstateCaseRecord {
    const NAME: &'static str = "EstateCaseRecord";
    type Row = EstateCaseRecordValue;
}
impl EstateCaseRecord {
    pub fn reference() -> AspectSelection<EstateRecords, EstateCase, Self> {
        aspect(EstateRecords, EstateCase, Self)
    }
}
pub struct EstateCaseRecordValue {
    pub estate_case_identity_field: EstateCaseId,
    pub estate_workflow_stage_field: EstateWorkflowStage,
    pub estate_case_status_field: EstateCaseStatus,
}
impl fragment::Record<EstateCaseRecordValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<EstateCaseRecordValue, Self> {
        fragment::field("EstateCaseIdentityField", |row: &EstateCaseRecordValue| {
            row.estate_case_identity_field
        })
        .equality_queryable(true)
        .and("EstateWorkflowStageField", |row: &EstateCaseRecordValue| {
            row.estate_workflow_stage_field
        })
        .equality_queryable(true)
        .and("EstateCaseStatusField", |row: &EstateCaseRecordValue| {
            row.estate_case_status_field
        })
        .equality_queryable(true)
        .build(
            |estate_case_identity_field,
             estate_workflow_stage_field,
             estate_case_status_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(EstateCaseRecordValue {
                    estate_case_identity_field,
                    estate_workflow_stage_field,
                    estate_case_status_field,
                })
            },
        )
    }
}
pub struct LegalAuthorityRecord;
impl fragment::Aspect for LegalAuthorityRecord {
    const NAME: &'static str = "LegalAuthorityRecord";
    type Row = LegalAuthorityRecordValue;
}
impl LegalAuthorityRecord {
    pub fn reference() -> AspectSelection<EstateRecords, LegalAuthority, Self> {
        aspect(EstateRecords, LegalAuthority, Self)
    }
}
pub struct LegalAuthorityRecordValue {
    pub legal_authority_identity_field: LegalAuthorityId,
    pub legal_authority_kind_field: LegalAuthorityKind,
    pub legal_authority_recognized_field: bool,
}
impl fragment::Record<LegalAuthorityRecordValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<LegalAuthorityRecordValue, Self> {
        fragment::field(
            "LegalAuthorityIdentityField",
            |row: &LegalAuthorityRecordValue| row.legal_authority_identity_field,
        )
        .equality_queryable(true)
        .and(
            "LegalAuthorityKindField",
            |row: &LegalAuthorityRecordValue| row.legal_authority_kind_field,
        )
        .equality_queryable(true)
        .and(
            "LegalAuthorityRecognizedField",
            |row: &LegalAuthorityRecordValue| row.legal_authority_recognized_field,
        )
        .equality_queryable(true)
        .build(
            |legal_authority_identity_field,
             legal_authority_kind_field,
             legal_authority_recognized_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(LegalAuthorityRecordValue {
                    legal_authority_identity_field,
                    legal_authority_kind_field,
                    legal_authority_recognized_field,
                })
            },
        )
    }
}
