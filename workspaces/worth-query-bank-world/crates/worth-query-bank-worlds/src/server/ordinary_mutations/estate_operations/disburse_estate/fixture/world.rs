pub use bank_domain::{
    estate::{
        BankEstateWorld, CapabilityGrantStatus, CapabilityValidity, DelegationLimit,
        EmergencyAccessReason, EmergencyAccessStatus, EstateBranch, EstateCapabilityGrant,
        EstateCapabilityOperation, EstateCapabilityPurpose, EstateCapabilityScope, EstateCase,
        EstateCaseStatus, EstateDeathNotice, EstateEmergencyAccess, EstateEmployeeAssignment,
        EstateLegalAuthority, EstateMoment, EstateWorkflowStage, LegalAuthorityId,
        LegalAuthorityKind, MandatoryEstateReview, MandatoryReviewKind, MandatoryReviewStatus,
        RestrictedBankField,
    },
    model::{BankPrincipalId, EmployeeRole, Money},
};

pub use super::{
    ActorConflict, BeneficiaryPosture, DisbursementWorldSpec, ExecutorPosture, GrantPosture, ACTOR,
    ALTERNATE_ESTATE, ALTERNATE_ESTATE_GRANT, ALTERNATE_SOURCE_GRANT, ASSIGNMENT, AUTHORITY,
    BENEFICIARY, BRANCH, DECEASED, DESTINATION, EMERGENCY_ACCESS, EMERGENCY_GRANT,
    EMERGENCY_REVIEW, ESTATE, EXECUTOR, GRANT, INSTITUTION, NOTICE, SECOND_AUTHORITY,
    SECOND_EXECUTOR, SOURCE,
};

pub fn estate_world(
    include_drift_authority: bool,
    spec: DisbursementWorldSpec,
    grant_valid_until_epoch: Option<u64>,
) -> BankEstateWorld {
    let world = BankEstateWorld::default()
        .with_branch(EstateBranch {
            id: BRANCH,
            institution: INSTITUTION,
        })
        .with_death_notice(EstateDeathNotice {
            id: NOTICE,
            subject: DECEASED,
            status: bank_domain::estate::DeathNoticeStatus::Verified,
        })
        .with_case(estate_case(ESTATE))
        .with_assignment(EstateEmployeeAssignment {
            id: ASSIGNMENT,
            principal: ACTOR,
            institution: INSTITUTION,
            branch: BRANCH,
            role: EmployeeRole::EstateSpecialist,
        })
        .with_estate_assignment(ESTATE, ASSIGNMENT);
    let world = install_beneficiary(world, spec.beneficiary);
    let world = install_executor(world, spec.executor);
    let world = install_actor_conflict(world, spec.actor_conflict);
    let world = install_grant(world, spec.grant, grant_valid_until_epoch);
    if !include_drift_authority {
        return world;
    }
    world
        .with_case(estate_case(ALTERNATE_ESTATE))
        .with_estate_assignment(ALTERNATE_ESTATE, ASSIGNMENT)
        .with_grant(disbursement_grant(
            ALTERNATE_ESTATE_GRANT,
            ALTERNATE_ESTATE,
            SOURCE,
            None,
        ))
        .with_grant(disbursement_grant(
            ALTERNATE_SOURCE_GRANT,
            ESTATE,
            DESTINATION,
            None,
        ))
}

pub fn install_beneficiary(world: BankEstateWorld, posture: BeneficiaryPosture) -> BankEstateWorld {
    match posture {
        BeneficiaryPosture::Ready => world
            .with_beneficiary(ESTATE, BENEFICIARY)
            .with_joint_owner(DESTINATION, BENEFICIARY),
        BeneficiaryPosture::Missing => world.with_joint_owner(DESTINATION, BENEFICIARY),
        BeneficiaryPosture::WrongEstate => world
            .with_case(estate_case(ALTERNATE_ESTATE))
            .with_beneficiary(ALTERNATE_ESTATE, BENEFICIARY)
            .with_joint_owner(DESTINATION, BENEFICIARY),
        BeneficiaryPosture::JointOwnerMissing => world.with_beneficiary(ESTATE, BENEFICIARY),
        BeneficiaryPosture::JointOwnerWrongAccount => world
            .with_beneficiary(ESTATE, BENEFICIARY)
            .with_joint_owner(SOURCE, BENEFICIARY),
    }
}

pub fn install_executor(world: BankEstateWorld, posture: ExecutorPosture) -> BankEstateWorld {
    let (recognized, holder) = match posture {
        ExecutorPosture::Unrecognized => (false, EXECUTOR),
        ExecutorPosture::WrongHolder => (true, BENEFICIARY),
        _ => (true, EXECUTOR),
    };
    let world = world.with_legal_authority(legal_authority(AUTHORITY, holder, recognized));
    let world = if matches!(posture, ExecutorPosture::Missing) {
        world
    } else {
        world.with_executor(ESTATE, EXECUTOR)
    };
    if matches!(posture, ExecutorPosture::MultipleLawful) {
        world
            .with_legal_authority(legal_authority(SECOND_AUTHORITY, SECOND_EXECUTOR, true))
            .with_executor(ESTATE, SECOND_EXECUTOR)
    } else {
        world
    }
}

pub fn install_actor_conflict(world: BankEstateWorld, conflict: ActorConflict) -> BankEstateWorld {
    match conflict {
        ActorConflict::None => world,
        ActorConflict::Beneficiary => world.with_beneficiary(ESTATE, ACTOR),
        ActorConflict::Executor => world.with_executor(ESTATE, ACTOR),
    }
}

pub fn install_grant(
    world: BankEstateWorld,
    posture: GrantPosture,
    grant_valid_until_epoch: Option<u64>,
) -> BankEstateWorld {
    match posture {
        GrantPosture::Disbursement => world.with_grant(disbursement_grant(
            GRANT,
            ESTATE,
            SOURCE,
            grant_valid_until_epoch,
        )),
        GrantPosture::ApprovedEmergencyOnly => world
            .with_grant(emergency_grant())
            .with_review(MandatoryEstateReview {
                id: EMERGENCY_REVIEW,
                estate: ESTATE,
                kind: MandatoryReviewKind::EmergencyAccess,
                reviewer: None,
                status: MandatoryReviewStatus::Required,
            })
            .with_emergency_access(EstateEmergencyAccess {
                id: EMERGENCY_ACCESS,
                requester: ACTOR,
                approver: Some(EXECUTOR),
                reviewer: None,
                grant: EMERGENCY_GRANT,
                review: EMERGENCY_REVIEW,
                reason: EmergencyAccessReason::PreventImmediateLoss,
                status: EmergencyAccessStatus::Approved,
                issued_at: EstateMoment::from_epoch_seconds(0),
                expires_at: EstateMoment::from_epoch_seconds(u64::MAX),
            }),
    }
}

pub fn estate_case(id: bank_domain::estate::EstateCaseId) -> EstateCase {
    EstateCase {
        id,
        institution: INSTITUTION,
        branch: BRANCH,
        deceased: DECEASED,
        account: SOURCE,
        death_notice: NOTICE,
        stage: EstateWorkflowStage::Administration,
        status: EstateCaseStatus::Open,
    }
}

pub fn legal_authority(
    id: LegalAuthorityId,
    holder: BankPrincipalId,
    recognized: bool,
) -> EstateLegalAuthority {
    EstateLegalAuthority {
        id,
        estate: ESTATE,
        holder,
        kind: LegalAuthorityKind::CourtAppointment,
        recognized,
    }
}

pub fn disbursement_grant(
    id: bank_domain::estate::CapabilityGrantId,
    estate: bank_domain::estate::EstateCaseId,
    account: bank_domain::model::AccountId,
    grant_valid_until_epoch: Option<u64>,
) -> EstateCapabilityGrant {
    let valid_until = grant_valid_until_epoch.unwrap_or(u64::MAX);
    EstateCapabilityGrant {
        id,
        grantor: DECEASED,
        grantee: ACTOR,
        scope: EstateCapabilityScope {
            account: Some(account),
            estate,
            institution: INSTITUTION,
            branch: BRANCH,
            operation: EstateCapabilityOperation::DisburseEstate,
            purpose: EstateCapabilityPurpose::EstateDisbursement,
            field: None,
            amount_ceiling: Some(Money::from_minor(10_000).unwrap()),
            validity: CapabilityValidity::new(
                EstateMoment::from_epoch_seconds(0),
                EstateMoment::from_epoch_seconds(valid_until),
            )
            .unwrap(),
            delegation: DelegationLimit::none(),
            workflow_stage: EstateWorkflowStage::Administration,
        },
        parent: None,
        status: CapabilityGrantStatus::Active,
    }
}

pub fn emergency_grant() -> EstateCapabilityGrant {
    let mut grant = disbursement_grant(EMERGENCY_GRANT, ESTATE, SOURCE, None);
    grant.scope.account = None;
    grant.scope.operation = EstateCapabilityOperation::ViewRestrictedEstate;
    grant.scope.purpose = EstateCapabilityPurpose::EmergencyProtection;
    grant.scope.field = Some(RestrictedBankField::AccountDetails);
    grant.scope.amount_ceiling = None;
    grant
}
