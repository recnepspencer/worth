use bank_domain::estate::*;
use bank_domain::model::{
    AccountId, BankPrincipalId, EmployeeAssignmentId, EmployeeRole, InstitutionId, Money,
    SignedMoney, USD,
};

#[allow(
    dead_code,
    reason = "shared integration fixture fields vary by courtroom"
)]
pub(super) struct Courtroom {
    pub world: BankEstateWorld,
    pub estate: EstateCaseId,
    pub source: AccountId,
    pub destination: AccountId,
    pub specialist: BankPrincipalId,
    pub manager: BankPrincipalId,
    pub beneficiary: BankPrincipalId,
    pub executor: BankPrincipalId,
    pub authority: LegalAuthorityId,
    pub review: MandatoryReviewId,
    pub teller: BankPrincipalId,
    pub specialist_assignment: EmployeeAssignmentId,
    pub manager_assignment: EmployeeAssignmentId,
    pub teller_assignment: EmployeeAssignmentId,
    pub emergency_review: MandatoryReviewId,
}

pub(super) fn courtroom() -> Courtroom {
    let institution = InstitutionId::new(1).unwrap();
    let branch = BranchId::new(2).unwrap();
    let estate = EstateCaseId::new(3).unwrap();
    let source = AccountId::new(4).unwrap();
    let destination = AccountId::new(5).unwrap();
    let deceased = BankPrincipalId::new(6).unwrap();
    let specialist = BankPrincipalId::new(7).unwrap();
    let manager = BankPrincipalId::new(8).unwrap();
    let beneficiary = BankPrincipalId::new(9).unwrap();
    let teller = BankPrincipalId::new(13).unwrap();
    let specialist_assignment = EmployeeAssignmentId::new(10).unwrap();
    let manager_assignment = EmployeeAssignmentId::new(11).unwrap();
    let teller_assignment = EmployeeAssignmentId::new(14).unwrap();
    let notice = DeathNoticeId::new(12).unwrap();
    let authority = LegalAuthorityId::new(30).unwrap();
    let review = MandatoryReviewId::new(20).unwrap();
    let emergency_review = MandatoryReviewId::new(21).unwrap();
    let world = BankEstateWorld::default()
        .with_branch(EstateBranch {
            id: branch,
            institution,
        })
        .with_death_notice(EstateDeathNotice {
            id: notice,
            subject: deceased,
            status: DeathNoticeStatus::Verified,
        })
        .with_case(EstateCase {
            id: estate,
            institution,
            branch,
            deceased,
            account: source,
            death_notice: notice,
            stage: EstateWorkflowStage::Administration,
            status: EstateCaseStatus::Open,
        })
        .with_assignment(EstateEmployeeAssignment {
            id: specialist_assignment,
            principal: specialist,
            institution,
            branch,
            role: EmployeeRole::EstateSpecialist,
        })
        .with_assignment(EstateEmployeeAssignment {
            id: manager_assignment,
            principal: manager,
            institution,
            branch,
            role: EmployeeRole::BranchManager,
        })
        .with_assignment(EstateEmployeeAssignment {
            id: teller_assignment,
            principal: teller,
            institution,
            branch,
            role: EmployeeRole::Teller,
        })
        .with_estate_assignment(estate, specialist_assignment)
        .with_estate_assignment(estate, manager_assignment)
        .with_estate_assignment(estate, teller_assignment)
        .with_legal_authority(EstateLegalAuthority {
            id: authority,
            estate,
            holder: beneficiary,
            kind: LegalAuthorityKind::CourtAppointment,
            recognized: true,
        })
        .with_review(MandatoryEstateReview {
            id: review,
            estate,
            kind: MandatoryReviewKind::EstateRelease,
            reviewer: Some(specialist),
            status: MandatoryReviewStatus::Completed,
        })
        .with_review(MandatoryEstateReview {
            id: emergency_review,
            estate,
            kind: MandatoryReviewKind::EmergencyAccess,
            reviewer: None,
            status: MandatoryReviewStatus::Required,
        })
        .with_executor(estate, beneficiary)
        .with_beneficiary(estate, beneficiary)
        .with_beneficiary(estate, manager)
        .with_joint_owner(destination, beneficiary)
        .with_joint_owner(destination, manager)
        .with_balance(source, SignedMoney::from_minor(100_000));
    Courtroom {
        world,
        estate,
        source,
        destination,
        specialist,
        manager,
        beneficiary,
        executor: beneficiary,
        authority,
        review,
        teller,
        specialist_assignment,
        manager_assignment,
        teller_assignment,
        emergency_review,
    }
}

pub(super) fn grant(
    id: u64,
    grantee: BankPrincipalId,
    operation: EstateCapabilityOperation,
    account: Option<AccountId>,
    field: Option<RestrictedBankField>,
    amount_ceiling: Option<i64>,
    delegation: DelegationLimit,
) -> EstateCapabilityGrant {
    EstateCapabilityGrant {
        id: CapabilityGrantId::new(id).unwrap(),
        grantor: BankPrincipalId::new(99).unwrap(),
        grantee,
        scope: EstateCapabilityScope {
            account,
            estate: EstateCaseId::new(3).unwrap(),
            institution: InstitutionId::new(1).unwrap(),
            branch: BranchId::new(2).unwrap(),
            operation,
            purpose: operation_purpose(operation),
            field,
            amount_ceiling: amount_ceiling.map(|amount| Money::<USD>::from_minor(amount).unwrap()),
            validity: CapabilityValidity::new(
                EstateMoment::from_epoch_seconds(100),
                EstateMoment::from_epoch_seconds(200),
            )
            .unwrap(),
            delegation,
            workflow_stage: EstateWorkflowStage::Administration,
        },
        parent: None,
        status: CapabilityGrantStatus::Active,
    }
}

fn operation_purpose(operation: EstateCapabilityOperation) -> EstateCapabilityPurpose {
    match operation {
        EstateCapabilityOperation::DisburseEstate => EstateCapabilityPurpose::EstateDisbursement,
        EstateCapabilityOperation::ApproveEmergencyAccess => {
            EstateCapabilityPurpose::EmergencyProtection
        }
        EstateCapabilityOperation::RecognizeExecutor => EstateCapabilityPurpose::LegalCompliance,
        _ => EstateCapabilityPurpose::EstateAdministration,
    }
}
