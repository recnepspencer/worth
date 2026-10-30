#[path = "fixture/snapshot.rs"]
pub mod snapshot;
#[path = "fixture/world.rs"]
pub mod world;

pub use bank_domain::{
    estate::{
        BranchId, CapabilityGrantId, DeathNoticeId, EmergencyAccessId, EstateAction, EstateCaseId,
        EstateDisbursement, EstatePosting, LegalAuthorityId, MandatoryReviewId,
    },
    model::{
        AccountId, BankPrincipalId, EmployeeAssignmentId, EmployeeRole, InstitutionId, Money,
        SignedMoney,
    },
    schema::AccountStatus,
};
pub use bank_server::{
    BankAuthenticatedPrincipal, BankEmployeeAssignmentSeed, BankPrincipalSeed, BankWorldSeed,
};

pub use crate::facade::authorization_time::{
    runtime_with_authorization_time, AuthorizationTimeController,
};
pub use crate::facade::support::{
    block_on, runtime, CausalCredential, DynamicIdentity, TestIdentityWorld,
};
pub use snapshot::snapshot;
pub use world::estate_world;

#[derive(Clone, Copy, Debug)]
pub enum BeneficiaryPosture {
    Ready,
    Missing,
    WrongEstate,
    JointOwnerMissing,
    JointOwnerWrongAccount,
}

#[derive(Clone, Copy, Debug)]
pub enum ExecutorPosture {
    Ready,
    Missing,
    Unrecognized,
    WrongHolder,
    MultipleLawful,
}

#[derive(Clone, Copy, Debug)]
pub enum ActorConflict {
    None,
    Beneficiary,
    Executor,
}

#[derive(Clone, Copy, Debug)]
pub enum GrantPosture {
    Disbursement,
    ApprovedEmergencyOnly,
}

#[derive(Clone, Copy)]
pub struct DisbursementWorldSpec {
    pub source_status: AccountStatus,
    pub destination_status: AccountStatus,
    pub beneficiary: BeneficiaryPosture,
    pub executor: ExecutorPosture,
    pub actor_conflict: ActorConflict,
    pub grant: GrantPosture,
}

impl DisbursementWorldSpec {
    pub const fn ready() -> Self {
        Self {
            source_status: AccountStatus::Open,
            destination_status: AccountStatus::Open,
            beneficiary: BeneficiaryPosture::Ready,
            executor: ExecutorPosture::Ready,
            actor_conflict: ActorConflict::None,
            grant: GrantPosture::Disbursement,
        }
    }
}

pub struct DisbursementFixture {
    pub world: TestIdentityWorld,
    pub identities: [DynamicIdentity; 5],
    pub estate: EstateCaseId,
    pub source: AccountId,
    pub destination: AccountId,
    pub beneficiary: BankPrincipalId,
}

impl DisbursementFixture {
    pub fn authenticate_actor(&self) -> BankAuthenticatedPrincipal {
        self.authenticate(1)
    }

    pub fn authenticate_source_owner(&self) -> BankAuthenticatedPrincipal {
        self.authenticate(0)
    }

    pub fn authenticate_beneficiary(&self) -> BankAuthenticatedPrincipal {
        self.authenticate(2)
    }

    pub fn authenticate(&self, identity: usize) -> BankAuthenticatedPrincipal {
        let request = crate::facade::support::request_scope();
        block_on(self.world.runtime.authenticate_with(
            &self.world.authentication,
            CausalCredential::for_identity(&self.identities[identity]),
            &request,
        ))
        .expect("the disbursement fixture principal should authenticate")
    }

    pub fn action(&self, amount: i64) -> EstateAction {
        disbursement_action(
            self.estate,
            self.source,
            self.destination,
            self.beneficiary,
            amount,
        )
    }
}

pub fn disbursement_world(scenario: &str, source_balance: i64) -> DisbursementFixture {
    build_world(
        scenario,
        source_balance,
        false,
        DisbursementWorldSpec::ready(),
        None,
        None,
    )
}

pub fn disbursement_drift_world(scenario: &str) -> DisbursementFixture {
    build_world(
        scenario,
        1_000,
        true,
        DisbursementWorldSpec::ready(),
        None,
        None,
    )
}

pub fn hostile_world(scenario: &str, spec: DisbursementWorldSpec) -> DisbursementFixture {
    build_world(scenario, 1_000, false, spec, None, None)
}

pub fn build_world(
    scenario: &str,
    source_balance: i64,
    include_drift_authority: bool,
    spec: DisbursementWorldSpec,
    authorization_time: Option<AuthorizationTimeController>,
    grant_valid_until_epoch: Option<u64>,
) -> DisbursementFixture {
    let identities = [
        DynamicIdentity::new(&format!("{scenario}-deceased")),
        DynamicIdentity::new(&format!("{scenario}-specialist")),
        DynamicIdentity::new(&format!("{scenario}-beneficiary")),
        DynamicIdentity::new(&format!("{scenario}-executor")),
        DynamicIdentity::new(&format!("{scenario}-second-executor")),
    ];
    let principals = [DECEASED, ACTOR, BENEFICIARY, EXECUTOR, SECOND_EXECUTOR];
    let seed = identities.iter().enumerate().fold(
        BankWorldSeed::new(snapshot(
            source_balance,
            spec.source_status,
            spec.destination_status,
        ))
        .employee(BankEmployeeAssignmentSeed::new(
            ASSIGNMENT,
            INSTITUTION,
            ACTOR,
            EmployeeRole::EstateSpecialist,
        ))
        .employee(BankEmployeeAssignmentSeed::new(
            EmployeeAssignmentId::new(2).unwrap(),
            INSTITUTION,
            ACTOR,
            EmployeeRole::Teller,
        ))
        .estate(estate_world(
            include_drift_authority,
            spec,
            grant_valid_until_epoch,
        )),
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
    DisbursementFixture {
        world,
        identities,
        estate: ESTATE,
        source: SOURCE,
        destination: DESTINATION,
        beneficiary: BENEFICIARY,
    }
}

pub fn disbursement_action(
    estate: EstateCaseId,
    source: AccountId,
    destination: AccountId,
    beneficiary: BankPrincipalId,
    amount: i64,
) -> EstateAction {
    EstateAction::DisburseEstate(EstateDisbursement {
        estate,
        source_account: source,
        destination_account: destination,
        beneficiary,
        amount: Money::from_minor(amount).unwrap(),
        postings: [
            EstatePosting {
                account: source,
                amount: SignedMoney::from_minor(-amount),
            },
            EstatePosting {
                account: destination,
                amount: SignedMoney::from_minor(amount),
            },
        ],
    })
}

pub const INSTITUTION: InstitutionId = InstitutionId::new(1).unwrap();
pub const BRANCH: BranchId = BranchId::new(2).unwrap();
pub const ESTATE: EstateCaseId = EstateCaseId::new(3).unwrap();
pub const SOURCE: AccountId = AccountId::new(4).unwrap();
pub const DESTINATION: AccountId = AccountId::new(5).unwrap();
pub const CASH: AccountId = AccountId::new(6).unwrap();
pub const DECEASED: BankPrincipalId = BankPrincipalId::new(7).unwrap();
pub const ACTOR: BankPrincipalId = BankPrincipalId::new(8).unwrap();
pub const BENEFICIARY: BankPrincipalId = BankPrincipalId::new(9).unwrap();
pub const EXECUTOR: BankPrincipalId = BankPrincipalId::new(10).unwrap();
pub const SECOND_EXECUTOR: BankPrincipalId = BankPrincipalId::new(21).unwrap();
pub const ASSIGNMENT: EmployeeAssignmentId = EmployeeAssignmentId::new(11).unwrap();
pub const NOTICE: DeathNoticeId = DeathNoticeId::new(12).unwrap();
pub const GRANT: CapabilityGrantId = CapabilityGrantId::new(13).unwrap();
pub const AUTHORITY: LegalAuthorityId = LegalAuthorityId::new(14).unwrap();
pub const ALTERNATE_ESTATE: EstateCaseId = EstateCaseId::new(18).unwrap();
pub const ALTERNATE_ESTATE_GRANT: CapabilityGrantId = CapabilityGrantId::new(19).unwrap();
pub const ALTERNATE_SOURCE_GRANT: CapabilityGrantId = CapabilityGrantId::new(20).unwrap();
pub const SECOND_AUTHORITY: LegalAuthorityId = LegalAuthorityId::new(22).unwrap();
pub const EMERGENCY_GRANT: CapabilityGrantId = CapabilityGrantId::new(23).unwrap();
pub const EMERGENCY_ACCESS: EmergencyAccessId = EmergencyAccessId::new(24).unwrap();
pub const EMERGENCY_REVIEW: MandatoryReviewId = MandatoryReviewId::new(25).unwrap();
