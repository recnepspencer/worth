use bank_domain::{
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
use bank_server::{
    BankAuthenticatedPrincipal, BankEmployeeAssignmentSeed, BankPrincipalSeed, BankWorldSeed,
};

use crate::support::{block_on, runtime, CausalCredential, DynamicIdentity, TestIdentityWorld};

pub(crate) struct FreezeFixture {
    pub(crate) world: TestIdentityWorld,
    identities: [DynamicIdentity; 3],
    pub(crate) estate: EstateCaseId,
    pub(crate) estate_account: AccountId,
    pub(crate) foreign_account: AccountId,
}

impl FreezeFixture {
    pub(crate) fn authenticate_specialist(&self) -> BankAuthenticatedPrincipal {
        self.authenticate(1)
    }

    pub(crate) fn authenticate_foreign_owner(&self) -> BankAuthenticatedPrincipal {
        self.authenticate(2)
    }

    pub(crate) const fn action(&self, account: AccountId) -> EstateAction {
        EstateAction::FreezeAccount {
            estate: self.estate,
            account,
        }
    }

    fn authenticate(&self, identity: usize) -> BankAuthenticatedPrincipal {
        let request = crate::support::request_scope();
        block_on(self.world.runtime.authenticate_with(
            &self.world.authentication,
            CausalCredential::for_identity(&self.identities[identity]),
            &request,
        ))
        .expect("the causally mapped freeze-world principal should authenticate")
    }
}

pub(crate) fn exact_freeze_world(scenario: &str, status: AccountStatus) -> FreezeFixture {
    freeze_world(scenario, status, ESTATE_ACCOUNT, AccountStatus::Open)
}

pub(crate) fn freeze_world_with_protected_foreign_status(
    scenario: &str,
    estate_account_status: AccountStatus,
    protected_foreign_status: AccountStatus,
) -> FreezeFixture {
    freeze_world(
        scenario,
        estate_account_status,
        ESTATE_ACCOUNT,
        protected_foreign_status,
    )
}

pub(crate) fn foreign_account_freeze_world(scenario: &str) -> FreezeFixture {
    freeze_world(
        scenario,
        AccountStatus::Open,
        FOREIGN_ACCOUNT,
        AccountStatus::Open,
    )
}

const INSTITUTION: InstitutionId = InstitutionId::new(1).unwrap();
const BRANCH: BranchId = BranchId::new(2).unwrap();
const ESTATE: EstateCaseId = EstateCaseId::new(3).unwrap();
const ESTATE_ACCOUNT: AccountId = AccountId::new(4).unwrap();
const FOREIGN_ACCOUNT: AccountId = AccountId::new(5).unwrap();
const DECEASED: BankPrincipalId = BankPrincipalId::new(6).unwrap();
const SPECIALIST: BankPrincipalId = BankPrincipalId::new(7).unwrap();
const FOREIGN_OWNER: BankPrincipalId = BankPrincipalId::new(8).unwrap();
const ASSIGNMENT: EmployeeAssignmentId = EmployeeAssignmentId::new(9).unwrap();
const NOTICE: DeathNoticeId = DeathNoticeId::new(10).unwrap();
const GRANT: CapabilityGrantId = CapabilityGrantId::new(11).unwrap();
const ALTERNATE_GRANT: CapabilityGrantId = CapabilityGrantId::new(12).unwrap();

fn freeze_world(
    scenario: &str,
    estate_account_status: AccountStatus,
    granted_account: AccountId,
    protected_foreign_status: AccountStatus,
) -> FreezeFixture {
    let identities = [
        DynamicIdentity::new(&format!("{scenario}-deceased")),
        DynamicIdentity::new(&format!("{scenario}-specialist")),
        DynamicIdentity::new(&format!("{scenario}-foreign-owner")),
    ];
    let snapshot = BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
        .institution(INSTITUTION)
        .principal(DECEASED)
        .principal(SPECIALIST)
        .principal(FOREIGN_OWNER)
        .personal_account(
            ESTATE_ACCOUNT,
            INSTITUTION,
            DECEASED,
            AccountName::new("Estate Operating").unwrap(),
            estate_account_status,
        )
        .personal_account(
            FOREIGN_ACCOUNT,
            INSTITUTION,
            FOREIGN_OWNER,
            AccountName::new("Foreign Operating").unwrap(),
            protected_foreign_status,
        )
        .build()
        .expect("the freeze fixture snapshot should be structurally valid");
    let estate = estate_world(granted_account);
    let seed = identities.iter().enumerate().fold(
        BankWorldSeed::new(snapshot)
            .employee(BankEmployeeAssignmentSeed::new(
                ASSIGNMENT,
                INSTITUTION,
                SPECIALIST,
                EmployeeRole::EstateSpecialist,
            ))
            .estate(estate),
        |seed, (ordinal, identity)| {
            let principal = [DECEASED, SPECIALIST, FOREIGN_OWNER][ordinal];
            seed.principal(BankPrincipalSeed::enabled(principal, identity.external()))
        },
    );
    FreezeFixture {
        world: runtime(seed),
        identities,
        estate: ESTATE,
        estate_account: ESTATE_ACCOUNT,
        foreign_account: FOREIGN_ACCOUNT,
    }
}

fn estate_world(granted_account: AccountId) -> BankEstateWorld {
    let alternate_account = if granted_account == ESTATE_ACCOUNT {
        FOREIGN_ACCOUNT
    } else {
        ESTATE_ACCOUNT
    };
    BankEstateWorld::default()
        .with_branch(EstateBranch {
            id: BRANCH,
            institution: INSTITUTION,
        })
        .with_death_notice(EstateDeathNotice {
            id: NOTICE,
            subject: DECEASED,
            status: DeathNoticeStatus::Verified,
        })
        .with_case(EstateCase {
            id: ESTATE,
            institution: INSTITUTION,
            branch: BRANCH,
            deceased: DECEASED,
            account: ESTATE_ACCOUNT,
            death_notice: NOTICE,
            stage: EstateWorkflowStage::Administration,
            status: EstateCaseStatus::Open,
        })
        .with_assignment(EstateEmployeeAssignment {
            id: ASSIGNMENT,
            principal: SPECIALIST,
            institution: INSTITUTION,
            branch: BRANCH,
            role: EmployeeRole::EstateSpecialist,
        })
        .with_estate_assignment(ESTATE, ASSIGNMENT)
        .with_grant(freeze_grant(GRANT, granted_account))
        .with_grant(freeze_grant(ALTERNATE_GRANT, alternate_account))
}

fn freeze_grant(id: CapabilityGrantId, account: AccountId) -> EstateCapabilityGrant {
    EstateCapabilityGrant {
        id,
        grantor: DECEASED,
        grantee: SPECIALIST,
        scope: EstateCapabilityScope {
            account: Some(account),
            estate: ESTATE,
            institution: INSTITUTION,
            branch: BRANCH,
            operation: EstateCapabilityOperation::FreezeAccount,
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
