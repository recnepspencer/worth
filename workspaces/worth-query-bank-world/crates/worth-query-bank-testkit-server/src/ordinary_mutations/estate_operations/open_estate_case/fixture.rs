pub use bank_domain::{
    estate::{
        BankEstateWorld, BranchId, CapabilityGrantId, CapabilityGrantStatus, CapabilityValidity,
        DeathNoticeId, DeathNoticeStatus, DelegationLimit, EstateAction, EstateBranch,
        EstateCapabilityGrant, EstateCapabilityOperation, EstateCapabilityPurpose,
        EstateCapabilityScope, EstateCase, EstateCaseId, EstateCaseStatus, EstateDeathNotice,
        EstateEmployeeAssignment, EstateMoment, EstateWorkflowStage,
    },
    model::{
        AccountId, AccountName, BankPrincipalId, BankSnapshotVersion, EmployeeAssignmentId,
        EmployeeRole, InstitutionId,
    },
    proposals::BankSnapshotBuilder,
    schema::AccountStatus,
};
pub use bank_server::{
    BankAuthenticatedPrincipal, BankEmployeeAssignmentSeed, BankPrincipalSeed, BankWorldSeed,
};

pub use crate::facade::support::{
    block_on, runtime, CausalCredential, DynamicIdentity, TestIdentityWorld,
};

pub struct CaseOpeningFixture {
    pub world: TestIdentityWorld,
    pub identities: [DynamicIdentity; 3],
    pub estate: EstateCaseId,
    pub notice: DeathNoticeId,
    pub foreign_notice: DeathNoticeId,
}

impl CaseOpeningFixture {
    pub fn authenticate_specialist(&self) -> BankAuthenticatedPrincipal {
        let request = crate::facade::support::request_scope();
        block_on(self.world.runtime.authenticate_with(
            &self.world.authentication,
            CausalCredential::for_identity(&self.identities[1]),
            &request,
        ))
        .expect("the causally mapped estate specialist should authenticate")
    }

    pub const fn action(&self, notice: DeathNoticeId) -> EstateAction {
        EstateAction::OpenEstateCase {
            estate: self.estate,
            notice,
        }
    }
}

pub fn case_opening_world(
    scenario: &str,
    case_status: EstateCaseStatus,
    notice_status: DeathNoticeStatus,
) -> CaseOpeningFixture {
    let identities = [
        DynamicIdentity::new(&format!("{scenario}-deceased")),
        DynamicIdentity::new(&format!("{scenario}-specialist")),
        DynamicIdentity::new(&format!("{scenario}-foreign-deceased")),
    ];
    let principals = [DECEASED, SPECIALIST, FOREIGN_DECEASED];
    let seed = identities.iter().enumerate().fold(
        BankWorldSeed::new(snapshot())
            .employee(BankEmployeeAssignmentSeed::new(
                ASSIGNMENT,
                INSTITUTION,
                SPECIALIST,
                EmployeeRole::EstateSpecialist,
            ))
            .estate(estate_world(case_status, notice_status)),
        |seed, (ordinal, identity)| {
            seed.principal(BankPrincipalSeed::enabled(
                principals[ordinal],
                identity.external(),
            ))
        },
    );
    CaseOpeningFixture {
        world: runtime(seed),
        identities,
        estate: ESTATE,
        notice: NOTICE,
        foreign_notice: FOREIGN_NOTICE,
    }
}

pub const INSTITUTION: InstitutionId = InstitutionId::new(1).unwrap();
pub const BRANCH: BranchId = BranchId::new(2).unwrap();
pub const ESTATE: EstateCaseId = EstateCaseId::new(3).unwrap();
pub const FOREIGN_ESTATE: EstateCaseId = EstateCaseId::new(4).unwrap();
pub const ESTATE_ACCOUNT: AccountId = AccountId::new(5).unwrap();
pub const FOREIGN_ACCOUNT: AccountId = AccountId::new(6).unwrap();
pub const DECEASED: BankPrincipalId = BankPrincipalId::new(7).unwrap();
pub const SPECIALIST: BankPrincipalId = BankPrincipalId::new(8).unwrap();
pub const FOREIGN_DECEASED: BankPrincipalId = BankPrincipalId::new(9).unwrap();
pub const ASSIGNMENT: EmployeeAssignmentId = EmployeeAssignmentId::new(10).unwrap();
pub const NOTICE: DeathNoticeId = DeathNoticeId::new(11).unwrap();
pub const FOREIGN_NOTICE: DeathNoticeId = DeathNoticeId::new(12).unwrap();
pub const GRANT: CapabilityGrantId = CapabilityGrantId::new(13).unwrap();

pub fn snapshot() -> bank_domain::proposals::BankSnapshot {
    BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
        .institution(INSTITUTION)
        .principal(DECEASED)
        .principal(SPECIALIST)
        .principal(FOREIGN_DECEASED)
        .personal_account(
            ESTATE_ACCOUNT,
            INSTITUTION,
            DECEASED,
            AccountName::new("Estate Opening").unwrap(),
            AccountStatus::Open,
        )
        .personal_account(
            FOREIGN_ACCOUNT,
            INSTITUTION,
            FOREIGN_DECEASED,
            AccountName::new("Foreign Estate").unwrap(),
            AccountStatus::Open,
        )
        .build()
        .expect("the case-opening snapshot should be structurally valid")
}

pub fn estate_world(
    case_status: EstateCaseStatus,
    notice_status: DeathNoticeStatus,
) -> BankEstateWorld {
    BankEstateWorld::default()
        .with_branch(EstateBranch {
            id: BRANCH,
            institution: INSTITUTION,
        })
        .with_death_notice(EstateDeathNotice {
            id: NOTICE,
            subject: DECEASED,
            status: notice_status,
        })
        .with_death_notice(EstateDeathNotice {
            id: FOREIGN_NOTICE,
            subject: FOREIGN_DECEASED,
            status: DeathNoticeStatus::Verified,
        })
        .with_case(estate_case(
            ESTATE,
            ESTATE_ACCOUNT,
            DECEASED,
            NOTICE,
            case_status,
        ))
        .with_case(estate_case(
            FOREIGN_ESTATE,
            FOREIGN_ACCOUNT,
            FOREIGN_DECEASED,
            FOREIGN_NOTICE,
            EstateCaseStatus::Open,
        ))
        .with_assignment(EstateEmployeeAssignment {
            id: ASSIGNMENT,
            principal: SPECIALIST,
            institution: INSTITUTION,
            branch: BRANCH,
            role: EmployeeRole::EstateSpecialist,
        })
        .with_estate_assignment(ESTATE, ASSIGNMENT)
        .with_grant(case_opening_grant())
}

pub fn estate_case(
    id: EstateCaseId,
    account: AccountId,
    deceased: BankPrincipalId,
    death_notice: DeathNoticeId,
    status: EstateCaseStatus,
) -> EstateCase {
    EstateCase {
        id,
        institution: INSTITUTION,
        branch: BRANCH,
        deceased,
        account,
        death_notice,
        stage: EstateWorkflowStage::Administration,
        status,
    }
}

pub fn case_opening_grant() -> EstateCapabilityGrant {
    EstateCapabilityGrant {
        id: GRANT,
        grantor: DECEASED,
        grantee: SPECIALIST,
        scope: EstateCapabilityScope {
            account: None,
            estate: ESTATE,
            institution: INSTITUTION,
            branch: BRANCH,
            operation: EstateCapabilityOperation::OpenEstateCase,
            purpose: EstateCapabilityPurpose::EstateAdministration,
            field: None,
            amount_ceiling: None,
            validity: CapabilityValidity::new(
                EstateMoment::from_epoch_seconds(0),
                EstateMoment::from_epoch_seconds(u64::MAX),
            )
            .unwrap(),
            delegation: DelegationLimit::none(),
            workflow_stage: EstateWorkflowStage::Administration,
        },
        parent: None,
        status: CapabilityGrantStatus::Active,
    }
}
