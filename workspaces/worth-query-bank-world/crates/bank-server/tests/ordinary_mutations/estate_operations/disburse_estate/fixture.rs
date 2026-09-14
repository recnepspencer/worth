#[path = "fixture/snapshot.rs"]
mod snapshot;
#[path = "fixture/world.rs"]
mod world;

use bank_domain::{
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
use bank_server::{
    BankAuthenticatedPrincipal, BankEmployeeAssignmentSeed, BankPrincipalSeed, BankWorldSeed,
};

use crate::authorization_time::{runtime_with_authorization_time, AuthorizationTimeController};
use crate::support::{block_on, runtime, CausalCredential, DynamicIdentity, TestIdentityWorld};
use snapshot::snapshot;
use world::estate_world;

#[derive(Clone, Copy, Debug)]
pub(super) enum BeneficiaryPosture {
    Ready,
    Missing,
    WrongEstate,
    JointOwnerMissing,
    JointOwnerWrongAccount,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ExecutorPosture {
    Ready,
    Missing,
    Unrecognized,
    WrongHolder,
    MultipleLawful,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ActorConflict {
    None,
    Beneficiary,
    Executor,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum GrantPosture {
    Disbursement,
    ApprovedEmergencyOnly,
}

#[derive(Clone, Copy)]
pub(super) struct DisbursementWorldSpec {
    pub(super) source_status: AccountStatus,
    pub(super) destination_status: AccountStatus,
    pub(super) beneficiary: BeneficiaryPosture,
    pub(super) executor: ExecutorPosture,
    pub(super) actor_conflict: ActorConflict,
    pub(super) grant: GrantPosture,
}

impl DisbursementWorldSpec {
    pub(super) const fn ready() -> Self {
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

pub(crate) struct DisbursementFixture {
    pub(crate) world: TestIdentityWorld,
    identities: [DynamicIdentity; 5],
    pub(crate) estate: EstateCaseId,
    pub(crate) source: AccountId,
    pub(crate) destination: AccountId,
    pub(crate) beneficiary: BankPrincipalId,
}

impl DisbursementFixture {
    pub(crate) fn authenticate_actor(&self) -> BankAuthenticatedPrincipal {
        self.authenticate(1)
    }

    pub(crate) fn authenticate_source_owner(&self) -> BankAuthenticatedPrincipal {
        self.authenticate(0)
    }

    pub(crate) fn authenticate_beneficiary(&self) -> BankAuthenticatedPrincipal {
        self.authenticate(2)
    }

    fn authenticate(&self, identity: usize) -> BankAuthenticatedPrincipal {
        let request = crate::support::request_scope();
        block_on(self.world.runtime.authenticate_with(
            &self.world.authentication,
            CausalCredential::for_identity(&self.identities[identity]),
            &request,
        ))
        .expect("the disbursement fixture principal should authenticate")
    }

    pub(crate) fn action(&self, amount: i64) -> EstateAction {
        disbursement_action(
            self.estate,
            self.source,
            self.destination,
            self.beneficiary,
            amount,
        )
    }
}

pub(crate) fn disbursement_world(scenario: &str, source_balance: i64) -> DisbursementFixture {
    build_world(
        scenario,
        source_balance,
        false,
        DisbursementWorldSpec::ready(),
        None,
        None,
    )
}

pub(super) fn disbursement_drift_world(scenario: &str) -> DisbursementFixture {
    build_world(
        scenario,
        1_000,
        true,
        DisbursementWorldSpec::ready(),
        None,
        None,
    )
}

pub(super) fn hostile_world(scenario: &str, spec: DisbursementWorldSpec) -> DisbursementFixture {
    build_world(scenario, 1_000, false, spec, None, None)
}

fn build_world(
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

pub(super) fn disbursement_action(
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

const INSTITUTION: InstitutionId = InstitutionId::new(1).unwrap();
const BRANCH: BranchId = BranchId::new(2).unwrap();
const ESTATE: EstateCaseId = EstateCaseId::new(3).unwrap();
const SOURCE: AccountId = AccountId::new(4).unwrap();
const DESTINATION: AccountId = AccountId::new(5).unwrap();
const CASH: AccountId = AccountId::new(6).unwrap();
const DECEASED: BankPrincipalId = BankPrincipalId::new(7).unwrap();
const ACTOR: BankPrincipalId = BankPrincipalId::new(8).unwrap();
const BENEFICIARY: BankPrincipalId = BankPrincipalId::new(9).unwrap();
const EXECUTOR: BankPrincipalId = BankPrincipalId::new(10).unwrap();
const SECOND_EXECUTOR: BankPrincipalId = BankPrincipalId::new(21).unwrap();
const ASSIGNMENT: EmployeeAssignmentId = EmployeeAssignmentId::new(11).unwrap();
const NOTICE: DeathNoticeId = DeathNoticeId::new(12).unwrap();
const GRANT: CapabilityGrantId = CapabilityGrantId::new(13).unwrap();
const AUTHORITY: LegalAuthorityId = LegalAuthorityId::new(14).unwrap();
const ALTERNATE_ESTATE: EstateCaseId = EstateCaseId::new(18).unwrap();
const ALTERNATE_ESTATE_GRANT: CapabilityGrantId = CapabilityGrantId::new(19).unwrap();
const ALTERNATE_SOURCE_GRANT: CapabilityGrantId = CapabilityGrantId::new(20).unwrap();
const SECOND_AUTHORITY: LegalAuthorityId = LegalAuthorityId::new(22).unwrap();
const EMERGENCY_GRANT: CapabilityGrantId = CapabilityGrantId::new(23).unwrap();
const EMERGENCY_ACCESS: EmergencyAccessId = EmergencyAccessId::new(24).unwrap();
const EMERGENCY_REVIEW: MandatoryReviewId = MandatoryReviewId::new(25).unwrap();
