pub use bank_domain::{
    estate::{
        BankEstateWorld, BranchId, CapabilityGrantId, CapabilityGrantStatus, CapabilityValidity,
        DeathNoticeId, DeathNoticeStatus, DelegationLimit, EstateAction, EstateBranch,
        EstateCapabilityGrant, EstateCapabilityOperation, EstateCapabilityPurpose,
        EstateCapabilityScope, EstateCase, EstateCaseId, EstateCaseStatus, EstateDeathNotice,
        EstateEmployeeAssignment, EstateLegalAuthority, EstateMoment, EstateWorkflowStage,
        LegalAuthorityId, LegalAuthorityKind, MandatoryEstateReview, MandatoryReviewId,
        MandatoryReviewKind, MandatoryReviewStatus,
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

pub use crate::facade::server::support::{
    block_on, runtime, CausalCredential, DynamicIdentity, TestIdentityWorld,
};

#[derive(Clone, Copy, Debug)]
pub enum ExecutorPosture {
    Ready,
    Missing,
    UnrecognizedAuthority,
    WrongHolderAuthority,
    WrongEstateAuthority,
}

#[derive(Clone, Copy, Debug)]
pub enum ReviewPosture {
    Completed,
    Missing,
    Required,
    WrongKind,
    Retargeted,
    NoReviewer,
}

#[derive(Clone, Copy, Debug)]
pub enum ActorConflict {
    None,
    Beneficiary,
    Executor,
}

#[derive(Clone, Copy)]
pub struct ReleaseWorldSpec {
    pub executor: ExecutorPosture,
    pub review: ReviewPosture,
    pub additional_executors: usize,
    pub unrelated_reviews: usize,
    pub actor_conflict: ActorConflict,
}

impl ReleaseWorldSpec {
    pub const fn ready() -> Self {
        Self {
            executor: ExecutorPosture::Ready,
            review: ReviewPosture::Completed,
            additional_executors: 0,
            unrelated_reviews: 0,
            actor_conflict: ActorConflict::None,
        }
    }
}

pub struct ReleaseFixture {
    pub world: TestIdentityWorld,
    pub identities: [DynamicIdentity; 7],
    pub estate: EstateCaseId,
}

impl ReleaseFixture {
    pub fn authenticate_actor(&self) -> BankAuthenticatedPrincipal {
        let request = crate::facade::server::support::request_scope();
        block_on(self.world.runtime.authenticate_with(
            &self.world.authentication,
            CausalCredential::for_identity(&self.identities[1]),
            &request,
        ))
        .expect("the assigned release actor should authenticate")
    }

    pub const fn action(&self) -> EstateAction {
        EstateAction::ReleaseEstate {
            estate: self.estate,
            executor: EXECUTOR,
            authority: AUTHORITY,
            review: REVIEW,
        }
    }
}

pub fn release_world(scenario: &str, spec: ReleaseWorldSpec) -> ReleaseFixture {
    let identities = [
        DynamicIdentity::new(&format!("{scenario}-deceased")),
        DynamicIdentity::new(&format!("{scenario}-actor")),
        DynamicIdentity::new(&format!("{scenario}-executor")),
        DynamicIdentity::new(&format!("{scenario}-reviewer")),
        DynamicIdentity::new(&format!("{scenario}-additional-executor-1")),
        DynamicIdentity::new(&format!("{scenario}-additional-executor-2")),
        DynamicIdentity::new(&format!("{scenario}-additional-executor-3")),
    ];
    let principals = [
        DECEASED,
        ACTOR,
        EXECUTOR,
        REVIEWER,
        ADDITIONAL_EXECUTORS[0],
        ADDITIONAL_EXECUTORS[1],
        ADDITIONAL_EXECUTORS[2],
    ];
    let seed = identities.iter().enumerate().fold(
        BankWorldSeed::new(snapshot())
            .employee(BankEmployeeAssignmentSeed::new(
                ASSIGNMENT,
                INSTITUTION,
                ACTOR,
                EmployeeRole::EstateSpecialist,
            ))
            .estate(estate_world(spec)),
        |seed, (ordinal, identity)| {
            seed.principal(BankPrincipalSeed::enabled(
                principals[ordinal],
                identity.external(),
            ))
        },
    );
    ReleaseFixture {
        world: runtime(seed),
        identities,
        estate: ESTATE,
    }
}

pub const INSTITUTION: InstitutionId = InstitutionId::new(1).unwrap();
pub const BRANCH: BranchId = BranchId::new(2).unwrap();
pub const ESTATE: EstateCaseId = EstateCaseId::new(3).unwrap();
pub const FOREIGN_ESTATE: EstateCaseId = EstateCaseId::new(4).unwrap();
pub const ESTATE_ACCOUNT: AccountId = AccountId::new(5).unwrap();
pub const FOREIGN_ACCOUNT: AccountId = AccountId::new(6).unwrap();
pub const DECEASED: BankPrincipalId = BankPrincipalId::new(7).unwrap();
pub const ACTOR: BankPrincipalId = BankPrincipalId::new(8).unwrap();
pub const EXECUTOR: BankPrincipalId = BankPrincipalId::new(9).unwrap();
pub const REVIEWER: BankPrincipalId = BankPrincipalId::new(10).unwrap();
pub const ADDITIONAL_EXECUTORS: [BankPrincipalId; 3] = [
    BankPrincipalId::new(11).unwrap(),
    BankPrincipalId::new(20).unwrap(),
    BankPrincipalId::new(21).unwrap(),
];
pub const ASSIGNMENT: EmployeeAssignmentId = EmployeeAssignmentId::new(12).unwrap();
pub const NOTICE: DeathNoticeId = DeathNoticeId::new(13).unwrap();
pub const FOREIGN_NOTICE: DeathNoticeId = DeathNoticeId::new(14).unwrap();
pub const GRANT: CapabilityGrantId = CapabilityGrantId::new(15).unwrap();
pub const AUTHORITY: LegalAuthorityId = LegalAuthorityId::new(16).unwrap();
pub const ADDITIONAL_AUTHORITIES: [LegalAuthorityId; 3] = [
    LegalAuthorityId::new(17).unwrap(),
    LegalAuthorityId::new(22).unwrap(),
    LegalAuthorityId::new(23).unwrap(),
];
pub const REVIEW: MandatoryReviewId = MandatoryReviewId::new(18).unwrap();

pub fn snapshot() -> bank_domain::proposals::BankSnapshot {
    BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
        .institution(INSTITUTION)
        .principal(DECEASED)
        .principal(ACTOR)
        .principal(EXECUTOR)
        .principal(REVIEWER)
        .principal(ADDITIONAL_EXECUTORS[0])
        .principal(ADDITIONAL_EXECUTORS[1])
        .principal(ADDITIONAL_EXECUTORS[2])
        .personal_account(
            ESTATE_ACCOUNT,
            INSTITUTION,
            DECEASED,
            AccountName::new("Release Estate").unwrap(),
            AccountStatus::Frozen,
        )
        .personal_account(
            FOREIGN_ACCOUNT,
            INSTITUTION,
            REVIEWER,
            AccountName::new("Foreign Estate").unwrap(),
            AccountStatus::Frozen,
        )
        .build()
        .expect("the estate-release snapshot should be structurally valid")
}

pub fn estate_world(spec: ReleaseWorldSpec) -> BankEstateWorld {
    let world = BankEstateWorld::default()
        .with_branch(EstateBranch {
            id: BRANCH,
            institution: INSTITUTION,
        })
        .with_death_notice(death_notice(NOTICE, DECEASED))
        .with_death_notice(death_notice(FOREIGN_NOTICE, REVIEWER))
        .with_case(estate_case(ESTATE, ESTATE_ACCOUNT, DECEASED, NOTICE))
        .with_case(estate_case(
            FOREIGN_ESTATE,
            FOREIGN_ACCOUNT,
            REVIEWER,
            FOREIGN_NOTICE,
        ))
        .with_assignment(EstateEmployeeAssignment {
            id: ASSIGNMENT,
            principal: ACTOR,
            institution: INSTITUTION,
            branch: BRANCH,
            role: EmployeeRole::EstateSpecialist,
        })
        .with_estate_assignment(ESTATE, ASSIGNMENT)
        .with_grant(release_grant());
    let world = install_executor(world, spec.executor);
    let world = install_review(world, spec.review);
    let world = match spec.actor_conflict {
        ActorConflict::None => world,
        ActorConflict::Beneficiary => world.with_beneficiary(ESTATE, ACTOR),
        ActorConflict::Executor => world.with_executor(ESTATE, ACTOR),
    };
    assert!(spec.additional_executors <= ADDITIONAL_EXECUTORS.len());
    let world = ADDITIONAL_EXECUTORS
        .into_iter()
        .zip(ADDITIONAL_AUTHORITIES)
        .take(spec.additional_executors)
        .fold(world, |world, (executor, authority)| {
            world
                .with_legal_authority(legal_authority(authority, ESTATE, executor, true))
                .with_executor(ESTATE, executor)
        });
    (0..spec.unrelated_reviews).fold(world, |world, ordinal| {
        world.with_review(MandatoryEstateReview {
            id: MandatoryReviewId::new(1_000 + ordinal as u64).unwrap(),
            estate: ESTATE,
            kind: MandatoryReviewKind::EmergencyAccess,
            reviewer: Some(REVIEWER),
            status: MandatoryReviewStatus::Completed,
        })
    })
}

pub fn install_executor(world: BankEstateWorld, posture: ExecutorPosture) -> BankEstateWorld {
    match posture {
        ExecutorPosture::Missing => {
            world.with_legal_authority(legal_authority(AUTHORITY, ESTATE, EXECUTOR, true))
        }
        ExecutorPosture::Ready => world
            .with_legal_authority(legal_authority(AUTHORITY, ESTATE, EXECUTOR, true))
            .with_executor(ESTATE, EXECUTOR),
        ExecutorPosture::UnrecognizedAuthority => world
            .with_legal_authority(legal_authority(AUTHORITY, ESTATE, EXECUTOR, false))
            .with_executor(ESTATE, EXECUTOR),
        ExecutorPosture::WrongHolderAuthority => world
            .with_legal_authority(legal_authority(AUTHORITY, ESTATE, REVIEWER, true))
            .with_executor(ESTATE, EXECUTOR),
        ExecutorPosture::WrongEstateAuthority => world
            .with_legal_authority(legal_authority(AUTHORITY, FOREIGN_ESTATE, EXECUTOR, true))
            .with_executor(ESTATE, EXECUTOR),
    }
}

pub fn install_review(world: BankEstateWorld, posture: ReviewPosture) -> BankEstateWorld {
    if matches!(posture, ReviewPosture::Missing) {
        return world;
    }
    let (estate, kind, reviewer, status) = match posture {
        ReviewPosture::Completed => (
            ESTATE,
            MandatoryReviewKind::EstateRelease,
            Some(REVIEWER),
            MandatoryReviewStatus::Completed,
        ),
        ReviewPosture::Required => (
            ESTATE,
            MandatoryReviewKind::EstateRelease,
            Some(REVIEWER),
            MandatoryReviewStatus::Required,
        ),
        ReviewPosture::WrongKind => (
            ESTATE,
            MandatoryReviewKind::EmergencyAccess,
            Some(REVIEWER),
            MandatoryReviewStatus::Completed,
        ),
        ReviewPosture::Retargeted => (
            FOREIGN_ESTATE,
            MandatoryReviewKind::EstateRelease,
            Some(REVIEWER),
            MandatoryReviewStatus::Completed,
        ),
        ReviewPosture::NoReviewer => (
            ESTATE,
            MandatoryReviewKind::EstateRelease,
            None,
            MandatoryReviewStatus::Completed,
        ),
        ReviewPosture::Missing => unreachable!(),
    };
    world.with_review(MandatoryEstateReview {
        id: REVIEW,
        estate,
        kind,
        reviewer,
        status,
    })
}

pub fn death_notice(id: DeathNoticeId, subject: BankPrincipalId) -> EstateDeathNotice {
    EstateDeathNotice {
        id,
        subject,
        status: DeathNoticeStatus::Verified,
    }
}

pub fn estate_case(
    id: EstateCaseId,
    account: AccountId,
    deceased: BankPrincipalId,
    death_notice: DeathNoticeId,
) -> EstateCase {
    EstateCase {
        id,
        institution: INSTITUTION,
        branch: BRANCH,
        deceased,
        account,
        death_notice,
        stage: EstateWorkflowStage::Administration,
        status: EstateCaseStatus::Open,
    }
}

pub fn legal_authority(
    id: LegalAuthorityId,
    estate: EstateCaseId,
    holder: BankPrincipalId,
    recognized: bool,
) -> EstateLegalAuthority {
    EstateLegalAuthority {
        id,
        estate,
        holder,
        kind: LegalAuthorityKind::CourtAppointment,
        recognized,
    }
}

pub fn release_grant() -> EstateCapabilityGrant {
    EstateCapabilityGrant {
        id: GRANT,
        grantor: DECEASED,
        grantee: ACTOR,
        scope: EstateCapabilityScope {
            account: None,
            estate: ESTATE,
            institution: INSTITUTION,
            branch: BRANCH,
            operation: EstateCapabilityOperation::ReleaseEstate,
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
