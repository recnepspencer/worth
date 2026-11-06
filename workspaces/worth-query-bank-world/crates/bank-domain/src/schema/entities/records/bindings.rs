//! Bindings of the bank model.
use super::*;
pub struct BankRecordsBindings;
impl fragment::BindingSet for BankRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<AccountId> for BankRecordsBindings {
    type Binding = AccountIdBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<InstitutionId> for BankRecordsBindings {
    type Binding = InstitutionIdBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<BusinessId> for BankRecordsBindings {
    type Binding = BusinessIdBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<PaymentId> for BankRecordsBindings {
    type Binding = PaymentIdBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<AccountName> for BankRecordsBindings {
    type Binding = AccountNameBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<AccountKind> for BankRecordsBindings {
    type Binding = AccountKindBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<AccountJournalRevision> for BankRecordsBindings {
    type Binding = AccountJournalRevisionBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<AccountStatus> for BankRecordsBindings {
    type Binding = AccountStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<CustomerRole> for BankRecordsBindings {
    type Binding = CustomerRoleBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<AccountAuthorizationId> for BankRecordsBindings {
    type Binding = AccountAuthorizationIdBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<EmployeeAssignmentId> for BankRecordsBindings {
    type Binding = EmployeeAssignmentIdBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<EmployeeRole> for BankRecordsBindings {
    type Binding = EmployeeRoleBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<SignedMoney<USD>> for BankRecordsBindings {
    type Binding = SignedUsdMoneyBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<PostingId> for BankRecordsBindings {
    type Binding = PostingIdBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<PostingPurpose> for BankRecordsBindings {
    type Binding = PostingPurposeBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<PaymentStatus> for BankRecordsBindings {
    type Binding = PaymentStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<JournalEntryId> for BankRecordsBindings {
    type Binding = JournalEntryIdBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<Money<USD>> for BankRecordsBindings {
    type Binding = UsdMoneyBinding;
    const EQUALITY_QUERYABLE: bool = false;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<BranchId> for BankRecordsBindings {
    type Binding = BranchIdBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<DeathNoticeId> for BankRecordsBindings {
    type Binding = DeathNoticeIdBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<DeathNoticeStatus> for BankRecordsBindings {
    type Binding = DeathNoticeStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<EstateCaseId> for BankRecordsBindings {
    type Binding = EstateCaseIdBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<EstateWorkflowStage> for BankRecordsBindings {
    type Binding = EstateWorkflowStageBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<EstateCaseStatus> for BankRecordsBindings {
    type Binding = EstateCaseStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<LegalAuthorityId> for BankRecordsBindings {
    type Binding = LegalAuthorityIdBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<LegalAuthorityKind> for BankRecordsBindings {
    type Binding = LegalAuthorityKindBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<bool> for BankRecordsBindings {
    type Binding = BoolApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<CapabilityGrantId> for BankRecordsBindings {
    type Binding = CapabilityGrantIdBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<EstateCapabilityOperation> for BankRecordsBindings {
    type Binding = EstateCapabilityOperationBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<EstateCapabilityPurpose> for BankRecordsBindings {
    type Binding = EstateCapabilityPurposeBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<RestrictedBankField> for BankRecordsBindings {
    type Binding = RestrictedBankFieldBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<EstateMoment> for BankRecordsBindings {
    type Binding = EstateMomentBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<DelegationLimit> for BankRecordsBindings {
    type Binding = DelegationLimitBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<CapabilityGrantStatus> for BankRecordsBindings {
    type Binding = CapabilityGrantStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<EmergencyAccessId> for BankRecordsBindings {
    type Binding = EmergencyAccessIdBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<EmergencyAccessReason> for BankRecordsBindings {
    type Binding = EmergencyAccessReasonBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<EmergencyAccessStatus> for BankRecordsBindings {
    type Binding = EmergencyAccessStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<MandatoryReviewId> for BankRecordsBindings {
    type Binding = MandatoryReviewIdBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<MandatoryReviewStatus> for BankRecordsBindings {
    type Binding = MandatoryReviewStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<MandatoryReviewKind> for BankRecordsBindings {
    type Binding = MandatoryReviewKindBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for BankRecordsBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for BankRecordsBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<BankPrincipalId> for BankRecordsBindings {
    type Binding = BankPrincipalIdBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
