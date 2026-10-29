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

pub use crate::facade::server::authorization_time::{
    runtime_with_authorization_time, AuthorizationTimeController,
};
pub use crate::facade::server::support::{
    block_on, runtime, CausalCredential, DynamicIdentity, TestIdentityWorld,
};

pub struct NotificationFixture {
    pub world: TestIdentityWorld,
    pub identities: [DynamicIdentity; 4],
    pub estate: EstateCaseId,
    pub second_estate: EstateCaseId,
    pub notice: DeathNoticeId,
    pub second_notice: DeathNoticeId,
    pub foreign_notice: DeathNoticeId,
    pub deceased: BankPrincipalId,
    pub foreign_deceased: BankPrincipalId,
    pub other_subject: BankPrincipalId,
    pub estate_account: AccountId,
}

impl NotificationFixture {
    pub fn authenticate_specialist(&self) -> BankAuthenticatedPrincipal {
        let request = crate::facade::server::support::request_scope();
        block_on(self.world.runtime.authenticate_with(
            &self.world.authentication,
            CausalCredential::for_identity(&self.identities[1]),
            &request,
        ))
        .expect("the causally mapped estate specialist should authenticate")
    }

    pub fn authenticate_deceased(&self) -> BankAuthenticatedPrincipal {
        let request = crate::facade::server::support::request_scope();
        block_on(self.world.runtime.authenticate_with(
            &self.world.authentication,
            CausalCredential::for_identity(&self.identities[0]),
            &request,
        ))
        .expect("the causally mapped deceased principal should authenticate")
    }

    pub const fn action(&self, notice: DeathNoticeId, subject: BankPrincipalId) -> EstateAction {
        EstateAction::NotifyDeath {
            estate: self.estate,
            notice,
            subject,
        }
    }
}

pub fn notification_world(scenario: &str, status: DeathNoticeStatus) -> NotificationFixture {
    notification_world_with_authorization_time(scenario, status, None)
}

pub fn notification_world_with_authorization_time(
    scenario: &str,
    status: DeathNoticeStatus,
    authorization_time: Option<AuthorizationTimeController>,
) -> NotificationFixture {
    notification_world_with_clock_and_grant_validity(scenario, status, authorization_time, None)
}

pub fn notification_world_with_clock_and_grant_validity(
    scenario: &str,
    status: DeathNoticeStatus,
    authorization_time: Option<AuthorizationTimeController>,
    grant_valid_until_epoch: Option<u64>,
) -> NotificationFixture {
    let identities = [
        DynamicIdentity::new(&format!("{scenario}-deceased")),
        DynamicIdentity::new(&format!("{scenario}-specialist")),
        DynamicIdentity::new(&format!("{scenario}-other-subject")),
        DynamicIdentity::new(&format!("{scenario}-foreign-deceased")),
    ];
    let snapshot = snapshot();
    let principals = [DECEASED, SPECIALIST, OTHER_SUBJECT, FOREIGN_DECEASED];
    let seed = identities.iter().enumerate().fold(
        BankWorldSeed::new(snapshot)
            .program_deployment_grant(
                bank_server::BankProgramDeploymentGrantSeed::for_program::<
                    bank_server::BankApplication,
                >(
                    "specialist-p0-deployment",
                    SPECIALIST,
                    SPECIALIST,
                    0,
                    u64::MAX,
                )
                .expect("the fixture explicitly issues one source-program deployment grant"),
            )
            .employee(BankEmployeeAssignmentSeed::new(
                ASSIGNMENT,
                INSTITUTION,
                SPECIALIST,
                EmployeeRole::EstateSpecialist,
            ))
            .estate(estate_world(status, grant_valid_until_epoch)),
        |seed, (ordinal, identity)| {
            seed.principal(BankPrincipalSeed::enabled(
                principals[ordinal],
                identity.external(),
            ))
        },
    );
    let world = match authorization_time {
        Some(source) => runtime_with_authorization_time(seed, source),
        None => runtime(seed),
    };
    NotificationFixture {
        world,
        identities,
        estate: ESTATE,
        second_estate: SECOND_ESTATE,
        notice: NOTICE,
        second_notice: SECOND_NOTICE,
        foreign_notice: FOREIGN_NOTICE,
        deceased: DECEASED,
        foreign_deceased: FOREIGN_DECEASED,
        other_subject: OTHER_SUBJECT,
        estate_account: ESTATE_ACCOUNT,
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
pub const OTHER_SUBJECT: BankPrincipalId = BankPrincipalId::new(9).unwrap();
pub const FOREIGN_DECEASED: BankPrincipalId = BankPrincipalId::new(10).unwrap();
pub const ASSIGNMENT: EmployeeAssignmentId = EmployeeAssignmentId::new(11).unwrap();
pub const NOTICE: DeathNoticeId = DeathNoticeId::new(12).unwrap();
pub const FOREIGN_NOTICE: DeathNoticeId = DeathNoticeId::new(13).unwrap();
pub const GRANT: CapabilityGrantId = CapabilityGrantId::new(14).unwrap();
pub const SECOND_ESTATE: EstateCaseId = EstateCaseId::new(15).unwrap();
pub const SECOND_ACCOUNT: AccountId = AccountId::new(16).unwrap();
pub const SECOND_NOTICE: DeathNoticeId = DeathNoticeId::new(17).unwrap();
pub const SECOND_GRANT: CapabilityGrantId = CapabilityGrantId::new(18).unwrap();

pub fn snapshot() -> bank_domain::proposals::BankSnapshot {
    BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
        .institution(INSTITUTION)
        .principal(DECEASED)
        .principal(SPECIALIST)
        .principal(OTHER_SUBJECT)
        .principal(FOREIGN_DECEASED)
        .personal_account(
            ESTATE_ACCOUNT,
            INSTITUTION,
            DECEASED,
            AccountName::new("Estate Operating").unwrap(),
            AccountStatus::Open,
        )
        .personal_account(
            FOREIGN_ACCOUNT,
            INSTITUTION,
            FOREIGN_DECEASED,
            AccountName::new("Foreign Estate Operating").unwrap(),
            AccountStatus::Open,
        )
        .personal_account(
            SECOND_ACCOUNT,
            INSTITUTION,
            OTHER_SUBJECT,
            AccountName::new("Second Estate Operating").unwrap(),
            AccountStatus::Open,
        )
        .build()
        .expect("the death-notification snapshot should be valid")
}

pub fn estate_world(
    status: DeathNoticeStatus,
    grant_valid_until_epoch: Option<u64>,
) -> BankEstateWorld {
    BankEstateWorld::default()
        .with_branch(EstateBranch {
            id: BRANCH,
            institution: INSTITUTION,
        })
        .with_death_notice(EstateDeathNotice {
            id: NOTICE,
            subject: DECEASED,
            status,
        })
        .with_death_notice(EstateDeathNotice {
            id: FOREIGN_NOTICE,
            subject: FOREIGN_DECEASED,
            status: DeathNoticeStatus::Reported,
        })
        .with_death_notice(EstateDeathNotice {
            id: SECOND_NOTICE,
            subject: OTHER_SUBJECT,
            status: DeathNoticeStatus::Reported,
        })
        .with_case(estate_case(ESTATE, ESTATE_ACCOUNT, DECEASED, NOTICE))
        .with_case(estate_case(
            FOREIGN_ESTATE,
            FOREIGN_ACCOUNT,
            FOREIGN_DECEASED,
            FOREIGN_NOTICE,
        ))
        .with_case(estate_case(
            SECOND_ESTATE,
            SECOND_ACCOUNT,
            OTHER_SUBJECT,
            SECOND_NOTICE,
        ))
        .with_assignment(EstateEmployeeAssignment {
            id: ASSIGNMENT,
            principal: SPECIALIST,
            institution: INSTITUTION,
            branch: BRANCH,
            role: EmployeeRole::EstateSpecialist,
        })
        .with_estate_assignment(ESTATE, ASSIGNMENT)
        .with_estate_assignment(SECOND_ESTATE, ASSIGNMENT)
        .with_grant(notification_grant(
            GRANT,
            ESTATE,
            DECEASED,
            grant_valid_until_epoch,
        ))
        .with_grant(notification_grant(
            SECOND_GRANT,
            SECOND_ESTATE,
            OTHER_SUBJECT,
            grant_valid_until_epoch,
        ))
}

pub fn estate_case(
    id: EstateCaseId,
    account: AccountId,
    deceased: BankPrincipalId,
    notice: DeathNoticeId,
) -> EstateCase {
    EstateCase {
        id,
        institution: INSTITUTION,
        branch: BRANCH,
        deceased,
        account,
        death_notice: notice,
        stage: EstateWorkflowStage::Administration,
        status: EstateCaseStatus::Open,
    }
}

pub fn notification_grant(
    id: CapabilityGrantId,
    estate: EstateCaseId,
    grantor: BankPrincipalId,
    grant_valid_until_epoch: Option<u64>,
) -> EstateCapabilityGrant {
    let valid_until = grant_valid_until_epoch.unwrap_or(u64::MAX);
    EstateCapabilityGrant {
        id,
        grantor,
        grantee: SPECIALIST,
        scope: EstateCapabilityScope {
            account: None,
            estate,
            institution: INSTITUTION,
            branch: BRANCH,
            operation: EstateCapabilityOperation::NotifyDeath,
            purpose: EstateCapabilityPurpose::EstateAdministration,
            field: None,
            amount_ceiling: None,
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
