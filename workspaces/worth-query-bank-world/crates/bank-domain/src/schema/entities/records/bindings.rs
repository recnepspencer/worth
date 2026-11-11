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

    const WRITABLE: bool = false;
}
impl fragment::Scalar<InstitutionId> for BankRecordsBindings {
    type Binding = InstitutionIdBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<BusinessId> for BankRecordsBindings {
    type Binding = BusinessIdBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<PaymentId> for BankRecordsBindings {
    type Binding = PaymentIdBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<AccountName> for BankRecordsBindings {
    type Binding = AccountNameBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<AccountKind> for BankRecordsBindings {
    type Binding = AccountKindBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<AccountJournalRevision> for BankRecordsBindings {
    type Binding = AccountJournalRevisionBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<AccountStatus> for BankRecordsBindings {
    type Binding = AccountStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<CustomerRole> for BankRecordsBindings {
    type Binding = CustomerRoleBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<AccountAuthorizationId> for BankRecordsBindings {
    type Binding = AccountAuthorizationIdBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<EmployeeAssignmentId> for BankRecordsBindings {
    type Binding = EmployeeAssignmentIdBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<EmployeeRole> for BankRecordsBindings {
    type Binding = EmployeeRoleBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<SignedMoney<USD>> for BankRecordsBindings {
    type Binding = SignedUsdMoneyBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<PostingId> for BankRecordsBindings {
    type Binding = PostingIdBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<PostingPurpose> for BankRecordsBindings {
    type Binding = PostingPurposeBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<PaymentStatus> for BankRecordsBindings {
    type Binding = PaymentStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<JournalEntryId> for BankRecordsBindings {
    type Binding = JournalEntryIdBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<Money<USD>> for BankRecordsBindings {
    type Binding = UsdMoneyBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<BranchId> for BankRecordsBindings {
    type Binding = BranchIdBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<DeathNoticeId> for BankRecordsBindings {
    type Binding = DeathNoticeIdBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<DeathNoticeStatus> for BankRecordsBindings {
    type Binding = DeathNoticeStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<EstateCaseId> for BankRecordsBindings {
    type Binding = EstateCaseIdBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<EstateWorkflowStage> for BankRecordsBindings {
    type Binding = EstateWorkflowStageBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<EstateCaseStatus> for BankRecordsBindings {
    type Binding = EstateCaseStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<LegalAuthorityId> for BankRecordsBindings {
    type Binding = LegalAuthorityIdBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<LegalAuthorityKind> for BankRecordsBindings {
    type Binding = LegalAuthorityKindBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<bool> for BankRecordsBindings {
    type Binding = BoolApplicationValueBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<CapabilityGrantId> for BankRecordsBindings {
    type Binding = CapabilityGrantIdBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<EstateCapabilityOperation> for BankRecordsBindings {
    type Binding = EstateCapabilityOperationBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<EstateCapabilityPurpose> for BankRecordsBindings {
    type Binding = EstateCapabilityPurposeBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<RestrictedBankField> for BankRecordsBindings {
    type Binding = RestrictedBankFieldBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<EstateMoment> for BankRecordsBindings {
    type Binding = EstateMomentBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<DelegationLimit> for BankRecordsBindings {
    type Binding = DelegationLimitBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<CapabilityGrantStatus> for BankRecordsBindings {
    type Binding = CapabilityGrantStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<EmergencyAccessId> for BankRecordsBindings {
    type Binding = EmergencyAccessIdBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<EmergencyAccessReason> for BankRecordsBindings {
    type Binding = EmergencyAccessReasonBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<EmergencyAccessStatus> for BankRecordsBindings {
    type Binding = EmergencyAccessStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<MandatoryReviewId> for BankRecordsBindings {
    type Binding = MandatoryReviewIdBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<MandatoryReviewStatus> for BankRecordsBindings {
    type Binding = MandatoryReviewStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<MandatoryReviewKind> for BankRecordsBindings {
    type Binding = MandatoryReviewKindBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for BankRecordsBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for BankRecordsBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<BankPrincipalId> for BankRecordsBindings {
    type Binding = BankPrincipalIdBinding;

    const WRITABLE: bool = false;
}
