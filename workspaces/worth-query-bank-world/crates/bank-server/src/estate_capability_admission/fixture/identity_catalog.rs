use bank_domain::{
    estate::{
        BranchId, CapabilityGrantId, DeathNoticeId, EmergencyAccessId, EstateCaseId,
        LegalAuthorityId, MandatoryReviewId,
    },
    model::{AccountId, BankPrincipalId, EmployeeAssignmentId, InstitutionId},
};

pub(crate) const INSTITUTION: InstitutionId = InstitutionId::new(1).unwrap();
pub(crate) const BRANCH: BranchId = BranchId::new(2).unwrap();
pub(crate) const ESTATE: EstateCaseId = EstateCaseId::new(3).unwrap();
pub(crate) const ALTERNATE_INSTITUTION: InstitutionId = InstitutionId::new(101).unwrap();
pub(crate) const ALTERNATE_BRANCH: BranchId = BranchId::new(102).unwrap();
pub(crate) const ACCOUNT: AccountId = AccountId::new(4).unwrap();
pub(crate) const OTHER_ACCOUNT: AccountId = AccountId::new(5).unwrap();
pub(crate) const DECEASED: BankPrincipalId = BankPrincipalId::new(6).unwrap();
pub(crate) const SPECIALIST: BankPrincipalId = BankPrincipalId::new(7).unwrap();
pub(crate) const EXECUTOR: BankPrincipalId = BankPrincipalId::new(8).unwrap();
pub(crate) const ASSIGNMENT: EmployeeAssignmentId = EmployeeAssignmentId::new(9).unwrap();
pub(crate) const APPROVER: BankPrincipalId = BankPrincipalId::new(13).unwrap();
pub(crate) const APPROVER_ASSIGNMENT: EmployeeAssignmentId = EmployeeAssignmentId::new(14).unwrap();
pub(crate) const REVIEWER: BankPrincipalId = BankPrincipalId::new(15).unwrap();
pub(crate) const REVIEWER_ASSIGNMENT: EmployeeAssignmentId = EmployeeAssignmentId::new(16).unwrap();
pub(crate) const DELEGATION_EXECUTOR_ASSIGNMENT: EmployeeAssignmentId =
    EmployeeAssignmentId::new(17).unwrap();
pub(crate) const DELEGATION_REVIEWER_ASSIGNMENT: EmployeeAssignmentId =
    EmployeeAssignmentId::new(18).unwrap();
pub(crate) const AUTHORITY: LegalAuthorityId = LegalAuthorityId::new(10).unwrap();
pub(crate) const NOTICE: DeathNoticeId = DeathNoticeId::new(12).unwrap();
pub(crate) const OTHER_AUTHORITY: LegalAuthorityId = LegalAuthorityId::new(11).unwrap();
pub(crate) const GRANT: CapabilityGrantId = CapabilityGrantId::new(20).unwrap();
pub(crate) const COMMAND_GRANT: CapabilityGrantId = CapabilityGrantId::new(21).unwrap();
pub(crate) const APPROVAL_GRANT: CapabilityGrantId = CapabilityGrantId::new(22).unwrap();
pub(crate) const SELF_APPROVAL_GRANT: CapabilityGrantId = CapabilityGrantId::new(23).unwrap();
pub(crate) const APPROVER_REQUEST_GRANT: CapabilityGrantId = CapabilityGrantId::new(24).unwrap();
pub(crate) const APPROVER_UPPER_BOUND_GRANT: CapabilityGrantId =
    CapabilityGrantId::new(25).unwrap();
pub(crate) const CLOSE_GRANT: CapabilityGrantId = CapabilityGrantId::new(26).unwrap();
pub(crate) const REVIEW_GRANT: CapabilityGrantId = CapabilityGrantId::new(27).unwrap();
pub(crate) const LIFECYCLE_OBSERVER_GRANT: CapabilityGrantId = CapabilityGrantId::new(28).unwrap();
pub(crate) const ALTERNATE_EMERGENCY_BOUND_GRANT: CapabilityGrantId =
    CapabilityGrantId::new(29).unwrap();
pub(crate) const DELEGATED_GRANT: CapabilityGrantId = CapabilityGrantId::new(30).unwrap();
pub(crate) const DISBURSEMENT_GRANT: CapabilityGrantId = CapabilityGrantId::new(31).unwrap();
pub(crate) const EMERGENCY_BOUND_GRANT: CapabilityGrantId = CapabilityGrantId::new(32).unwrap();
pub(crate) const REVOKE_CAPABILITY_GRANT: CapabilityGrantId = CapabilityGrantId::new(33).unwrap();
pub(crate) const APPROVER_DELEGATION_GRANT: CapabilityGrantId = CapabilityGrantId::new(34).unwrap();
pub(crate) const UNRELATED_GOVERNANCE_GRANT: CapabilityGrantId =
    CapabilityGrantId::new(35).unwrap();
pub(crate) const APPROVER_REVOKE_CAPABILITY_GRANT: CapabilityGrantId =
    CapabilityGrantId::new(36).unwrap();
pub(crate) const REQUESTED_ACCESS: EmergencyAccessId = EmergencyAccessId::new(40).unwrap();
pub(crate) const CLOSED_ACCESS: EmergencyAccessId = EmergencyAccessId::new(41).unwrap();
pub(crate) const REQUESTED_REVIEW: MandatoryReviewId = MandatoryReviewId::new(50).unwrap();
pub(crate) const COMPLETED_REVIEW: MandatoryReviewId = MandatoryReviewId::new(51).unwrap();
