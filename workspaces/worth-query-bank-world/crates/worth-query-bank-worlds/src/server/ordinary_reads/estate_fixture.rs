pub use bank_domain::{
    estate::{
        BankEstateWorld, BranchId, DeathNoticeId, DeathNoticeStatus, EstateBranch, EstateCase,
        EstateCaseId, EstateCaseStatus, EstateDeathNotice, EstateEmployeeAssignment,
        EstateLegalAuthority, EstateWorkflowStage, LegalAuthorityId, LegalAuthorityKind,
        MandatoryEstateReview, MandatoryReviewId, MandatoryReviewKind, MandatoryReviewStatus,
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

pub struct EstateReadFixture {
    pub world: TestIdentityWorld,
    pub identities: [DynamicIdentity; 3],
    pub estate: EstateCaseId,
    pub account: AccountId,
    pub branch: BranchId,
    pub deceased: BankPrincipalId,
    pub specialist: BankPrincipalId,
    pub executor: BankPrincipalId,
    pub assignment: EmployeeAssignmentId,
    pub authority: LegalAuthorityId,
    pub review: MandatoryReviewId,
}

impl EstateReadFixture {
    pub fn authenticate(&self, ordinal: usize) -> BankAuthenticatedPrincipal {
        let request = crate::facade::server::support::request_scope();
        block_on(self.world.runtime.authenticate_with(
            &self.world.authentication,
            CausalCredential::for_identity(&self.identities[ordinal]),
            &request,
        ))
        .expect("estate fixture principal should authenticate")
    }
}

pub fn estate_read_world(scenario: &str) -> EstateReadFixture {
    let institution = InstitutionId::new(1).unwrap();
    let branch = BranchId::new(2).unwrap();
    let estate = EstateCaseId::new(3).unwrap();
    let account = AccountId::new(4).unwrap();
    let deceased = BankPrincipalId::new(5).unwrap();
    let specialist = BankPrincipalId::new(6).unwrap();
    let executor = BankPrincipalId::new(7).unwrap();
    let assignment = EmployeeAssignmentId::new(8).unwrap();
    let notice = DeathNoticeId::new(9).unwrap();
    let authority = LegalAuthorityId::new(10).unwrap();
    let review = MandatoryReviewId::new(11).unwrap();
    let identities = [
        DynamicIdentity::new(&format!("{scenario}-deceased")),
        DynamicIdentity::new(&format!("{scenario}-specialist")),
        DynamicIdentity::new(&format!("{scenario}-executor")),
    ];
    let snapshot = BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
        .institution(institution)
        .principal(deceased)
        .principal(specialist)
        .principal(executor)
        .personal_account(
            account,
            institution,
            deceased,
            AccountName::new("Estate Operating").unwrap(),
            AccountStatus::Frozen,
        )
        .build()
        .expect("estate snapshot should be structurally valid");
    let estate_world = BankEstateWorld::default()
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
            account,
            death_notice: notice,
            stage: EstateWorkflowStage::Administration,
            status: EstateCaseStatus::Open,
        })
        .with_assignment(EstateEmployeeAssignment {
            id: assignment,
            principal: specialist,
            institution,
            branch,
            role: EmployeeRole::EstateSpecialist,
        })
        .with_estate_assignment(estate, assignment)
        .with_legal_authority(EstateLegalAuthority {
            id: authority,
            estate,
            holder: executor,
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
        .with_executor(estate, executor)
        .with_beneficiary(estate, executor);
    let seed = identities.iter().enumerate().fold(
        BankWorldSeed::new(snapshot)
            .employee(BankEmployeeAssignmentSeed::new(
                assignment,
                institution,
                specialist,
                EmployeeRole::EstateSpecialist,
            ))
            .estate(estate_world),
        |seed, (ordinal, identity)| {
            let principal = [deceased, specialist, executor][ordinal];
            seed.principal(BankPrincipalSeed::enabled(principal, identity.external()))
        },
    );
    EstateReadFixture {
        world: runtime(seed),
        identities,
        estate,
        account,
        branch,
        deceased,
        specialist,
        executor,
        assignment,
        authority,
        review,
    }
}
