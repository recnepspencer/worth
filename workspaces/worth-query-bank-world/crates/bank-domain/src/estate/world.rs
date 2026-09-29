use std::collections::{BTreeMap, BTreeSet};

use crate::model::{
    AccountId, BankPrincipalId, EmployeeAssignmentId, EmployeeRole, InstitutionId, SignedMoney, USD,
};

use super::{
    BranchId, CapabilityGrantId, DeathNoticeId, EmergencyAccessId, EstateCapabilityGrant,
    EstateCaseId, EstateEmergencyAccess, LegalAuthorityId, MandatoryReviewId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum EstateWorkflowStage {
    DeathReported,
    AccountsFrozen,
    AuthorityReview,
    Administration,
    ReleaseReview,
    Released,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EstateCaseStatus {
    PendingOpening,
    Open,
    Released,
    Closed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeathNoticeStatus {
    Reported,
    NotificationRequested,
    Verified,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegalAuthorityKind {
    CourtAppointment,
    SmallEstateAffidavit,
    InstitutionalRecognition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MandatoryReviewStatus {
    Required,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MandatoryReviewKind {
    EstateRelease,
    EmergencyAccess,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateCase {
    pub id: EstateCaseId,
    pub institution: InstitutionId,
    pub branch: BranchId,
    pub deceased: BankPrincipalId,
    pub account: AccountId,
    pub death_notice: DeathNoticeId,
    pub stage: EstateWorkflowStage,
    pub status: EstateCaseStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateBranch {
    pub id: BranchId,
    pub institution: InstitutionId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateDeathNotice {
    pub id: DeathNoticeId,
    pub subject: BankPrincipalId,
    pub status: DeathNoticeStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateEmployeeAssignment {
    pub id: EmployeeAssignmentId,
    pub principal: BankPrincipalId,
    pub institution: InstitutionId,
    pub branch: BranchId,
    pub role: EmployeeRole,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EstateLegalAuthority {
    pub id: LegalAuthorityId,
    pub estate: EstateCaseId,
    pub holder: BankPrincipalId,
    pub kind: LegalAuthorityKind,
    pub recognized: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MandatoryEstateReview {
    pub id: MandatoryReviewId,
    pub estate: EstateCaseId,
    pub kind: MandatoryReviewKind,
    pub reviewer: Option<BankPrincipalId>,
    pub status: MandatoryReviewStatus,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BankEstateWorld {
    branches: BTreeMap<BranchId, EstateBranch>,
    death_notices: BTreeMap<DeathNoticeId, EstateDeathNotice>,
    cases: BTreeMap<EstateCaseId, EstateCase>,
    assignments: BTreeMap<EmployeeAssignmentId, EstateEmployeeAssignment>,
    legal_authorities: BTreeMap<LegalAuthorityId, EstateLegalAuthority>,
    grants: BTreeMap<CapabilityGrantId, EstateCapabilityGrant>,
    emergency_access: BTreeMap<EmergencyAccessId, EstateEmergencyAccess>,
    reviews: BTreeMap<MandatoryReviewId, MandatoryEstateReview>,
    executors: BTreeSet<(EstateCaseId, BankPrincipalId)>,
    beneficiaries: BTreeSet<(EstateCaseId, BankPrincipalId)>,
    estate_assignments: BTreeSet<(EstateCaseId, EmployeeAssignmentId)>,
    joint_owners: BTreeSet<(AccountId, BankPrincipalId)>,
    authorized_signers: BTreeSet<(AccountId, BankPrincipalId)>,
    balances: BTreeMap<AccountId, SignedMoney<USD>>,
}

impl BankEstateWorld {
    pub fn with_branch(mut self, branch: EstateBranch) -> Self {
        self.branches.insert(branch.id, branch);
        self
    }

    pub fn with_death_notice(mut self, notice: EstateDeathNotice) -> Self {
        self.death_notices.insert(notice.id, notice);
        self
    }

    pub fn with_case(mut self, case: EstateCase) -> Self {
        self.cases.insert(case.id, case);
        self
    }

    pub fn with_assignment(mut self, assignment: EstateEmployeeAssignment) -> Self {
        self.assignments.insert(assignment.id, assignment);
        self
    }

    pub fn with_legal_authority(mut self, authority: EstateLegalAuthority) -> Self {
        self.legal_authorities.insert(authority.id, authority);
        self
    }

    pub fn with_grant(mut self, grant: EstateCapabilityGrant) -> Self {
        self.grants.insert(grant.id, grant);
        self
    }

    pub fn with_emergency_access(mut self, access: EstateEmergencyAccess) -> Self {
        self.emergency_access.insert(access.id, access);
        self
    }

    pub fn with_review(mut self, review: MandatoryEstateReview) -> Self {
        self.reviews.insert(review.id, review);
        self
    }

    pub fn with_executor(mut self, estate: EstateCaseId, principal: BankPrincipalId) -> Self {
        self.executors.insert((estate, principal));
        self
    }

    pub fn with_beneficiary(mut self, estate: EstateCaseId, principal: BankPrincipalId) -> Self {
        self.beneficiaries.insert((estate, principal));
        self
    }

    pub fn with_estate_assignment(
        mut self,
        estate: EstateCaseId,
        assignment: EmployeeAssignmentId,
    ) -> Self {
        self.estate_assignments.insert((estate, assignment));
        self
    }

    pub fn with_joint_owner(mut self, account: AccountId, principal: BankPrincipalId) -> Self {
        self.joint_owners.insert((account, principal));
        self
    }

    pub fn with_authorized_signer(
        mut self,
        account: AccountId,
        principal: BankPrincipalId,
    ) -> Self {
        self.authorized_signers.insert((account, principal));
        self
    }

    pub fn with_balance(mut self, account: AccountId, balance: SignedMoney<USD>) -> Self {
        self.balances.insert(account, balance);
        self
    }

    pub fn case(&self, id: EstateCaseId) -> Option<&EstateCase> {
        self.cases.get(&id)
    }

    pub fn branch(&self, id: BranchId) -> Option<&EstateBranch> {
        self.branches.get(&id)
    }

    pub fn death_notice(&self, id: DeathNoticeId) -> Option<&EstateDeathNotice> {
        self.death_notices.get(&id)
    }

    pub fn assignment(&self, id: EmployeeAssignmentId) -> Option<&EstateEmployeeAssignment> {
        self.assignments.get(&id)
    }

    pub fn legal_authority(&self, id: LegalAuthorityId) -> Option<&EstateLegalAuthority> {
        self.legal_authorities.get(&id)
    }

    pub fn grant(&self, id: CapabilityGrantId) -> Option<&EstateCapabilityGrant> {
        self.grants.get(&id)
    }

    pub fn emergency_access(&self, id: EmergencyAccessId) -> Option<&EstateEmergencyAccess> {
        self.emergency_access.get(&id)
    }

    pub fn review(&self, id: MandatoryReviewId) -> Option<&MandatoryEstateReview> {
        self.reviews.get(&id)
    }

    pub fn branches(&self) -> impl ExactSizeIterator<Item = &EstateBranch> {
        self.branches.values()
    }

    pub fn death_notices(&self) -> impl ExactSizeIterator<Item = &EstateDeathNotice> {
        self.death_notices.values()
    }

    pub fn cases(&self) -> impl ExactSizeIterator<Item = &EstateCase> {
        self.cases.values()
    }

    pub fn assignments(&self) -> impl ExactSizeIterator<Item = &EstateEmployeeAssignment> {
        self.assignments.values()
    }

    pub fn legal_authorities(&self) -> impl ExactSizeIterator<Item = &EstateLegalAuthority> {
        self.legal_authorities.values()
    }

    pub fn grants(&self) -> impl ExactSizeIterator<Item = &EstateCapabilityGrant> {
        self.grants.values()
    }

    pub fn emergency_accesses(&self) -> impl ExactSizeIterator<Item = &EstateEmergencyAccess> {
        self.emergency_access.values()
    }

    pub fn reviews(&self) -> impl ExactSizeIterator<Item = &MandatoryEstateReview> {
        self.reviews.values()
    }

    pub fn executors(&self) -> impl ExactSizeIterator<Item = (EstateCaseId, BankPrincipalId)> + '_ {
        self.executors.iter().copied()
    }

    pub fn beneficiaries(
        &self,
    ) -> impl ExactSizeIterator<Item = (EstateCaseId, BankPrincipalId)> + '_ {
        self.beneficiaries.iter().copied()
    }

    pub fn estate_assignments(
        &self,
    ) -> impl ExactSizeIterator<Item = (EstateCaseId, EmployeeAssignmentId)> + '_ {
        self.estate_assignments.iter().copied()
    }

    pub fn joint_owners(&self) -> impl ExactSizeIterator<Item = (AccountId, BankPrincipalId)> + '_ {
        self.joint_owners.iter().copied()
    }

    pub fn authorized_signers(
        &self,
    ) -> impl ExactSizeIterator<Item = (AccountId, BankPrincipalId)> + '_ {
        self.authorized_signers.iter().copied()
    }

    pub fn is_executor(&self, estate: EstateCaseId, principal: BankPrincipalId) -> bool {
        self.executors.contains(&(estate, principal))
    }

    pub fn is_beneficiary(&self, estate: EstateCaseId, principal: BankPrincipalId) -> bool {
        self.beneficiaries.contains(&(estate, principal))
    }

    pub fn is_joint_owner(&self, account: AccountId, principal: BankPrincipalId) -> bool {
        self.joint_owners.contains(&(account, principal))
    }

    pub fn balance(&self, account: AccountId) -> Option<SignedMoney<USD>> {
        self.balances.get(&account).copied()
    }

    pub fn has_completed_review(&self, estate: EstateCaseId, kind: MandatoryReviewKind) -> bool {
        self.reviews.values().any(|review| {
            review.estate == estate
                && review.kind == kind
                && review.status == MandatoryReviewStatus::Completed
        })
    }

    pub fn has_recognized_executor(&self, estate: EstateCaseId) -> bool {
        self.legal_authorities.values().any(|authority| {
            authority.estate == estate
                && authority.recognized
                && self.is_executor(estate, authority.holder)
        })
    }
}
