use bank_domain::{
    estate::{
        BankEstateWorld, BranchId, CapabilityGrantId, CapabilityGrantStatus, CapabilityValidity,
        DeathNoticeId, DeathNoticeStatus, DelegationLimit, EstateAction, EstateBranch,
        EstateCapabilityGrant, EstateCapabilityOperation, EstateCapabilityPurpose,
        EstateCapabilityScope, EstateCase, EstateCaseId, EstateCaseStatus, EstateDeathNotice,
        EstateEmployeeAssignment, EstateLegalAuthority, EstateMoment, EstateWorkflowStage,
        LegalAuthorityId, LegalAuthorityKind,
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

pub(super) struct RecognitionFixture {
    pub(super) world: TestIdentityWorld,
    identities: [DynamicIdentity; 5],
    pub(super) estate: EstateCaseId,
    pub(super) executor: BankPrincipalId,
    pub(super) authority: LegalAuthorityId,
}

impl RecognitionFixture {
    pub(super) fn authenticate_specialist(&self) -> BankAuthenticatedPrincipal {
        let request = crate::support::request_scope();
        block_on(self.world.runtime.authenticate_with(
            &self.world.authentication,
            CausalCredential::for_identity(&self.identities[1]),
            &request,
        ))
        .expect("the causally mapped estate specialist should authenticate")
    }

    pub(super) const fn action(&self, executor: BankPrincipalId) -> EstateAction {
        EstateAction::RecognizeExecutor {
            estate: self.estate,
            executor,
            authority: self.authority,
        }
    }
}

pub(super) fn exact_recognition_world(scenario: &str) -> RecognitionFixture {
    recognition_world(scenario, RecognitionPosture::exact(), None)
}

pub(super) fn adoption_recognition_world(scenario: &str) -> RecognitionFixture {
    let grant =
        bank_server::BankProgramDeploymentGrantSeed::for_program::<bank_server::BankApplication>(
            "recognition-context-deployment",
            SPECIALIST,
            SPECIALIST,
            0,
            u64::MAX,
        )
        .expect("explicit canonical P0 deployment authority");
    recognition_world(scenario, RecognitionPosture::exact(), Some(grant))
}
pub(super) fn foreign_authority_world(scenario: &str) -> RecognitionFixture {
    recognition_world(scenario, RecognitionPosture::foreign_authority(), None)
}

pub(super) fn holder_mismatch_world(scenario: &str) -> RecognitionFixture {
    recognition_world(scenario, RecognitionPosture::holder_mismatch(), None)
}

pub(super) fn unrecognized_authority_world(scenario: &str) -> RecognitionFixture {
    recognition_world(scenario, RecognitionPosture::unrecognized(), None)
}

pub(super) fn duplicate_executor_world(scenario: &str) -> RecognitionFixture {
    recognition_world(scenario, RecognitionPosture::duplicate(), None)
}

const INSTITUTION: InstitutionId = InstitutionId::new(1).unwrap();
const BRANCH: BranchId = BranchId::new(2).unwrap();
const ESTATE: EstateCaseId = EstateCaseId::new(3).unwrap();
const FOREIGN_ESTATE: EstateCaseId = EstateCaseId::new(4).unwrap();
const ESTATE_ACCOUNT: AccountId = AccountId::new(5).unwrap();
const FOREIGN_ACCOUNT: AccountId = AccountId::new(6).unwrap();
const DECEASED: BankPrincipalId = BankPrincipalId::new(7).unwrap();
const SPECIALIST: BankPrincipalId = BankPrincipalId::new(8).unwrap();
const EXECUTOR: BankPrincipalId = BankPrincipalId::new(9).unwrap();
const OTHER_HOLDER: BankPrincipalId = BankPrincipalId::new(10).unwrap();
const FOREIGN_DECEASED: BankPrincipalId = BankPrincipalId::new(11).unwrap();
const ASSIGNMENT: EmployeeAssignmentId = EmployeeAssignmentId::new(12).unwrap();
const NOTICE: DeathNoticeId = DeathNoticeId::new(13).unwrap();
const FOREIGN_NOTICE: DeathNoticeId = DeathNoticeId::new(14).unwrap();
const AUTHORITY: LegalAuthorityId = LegalAuthorityId::new(15).unwrap();
const GRANT: CapabilityGrantId = CapabilityGrantId::new(16).unwrap();

#[derive(Clone, Copy)]
struct RecognitionPosture {
    authority_estate: EstateCaseId,
    authority_holder: BankPrincipalId,
    authority_recognized: bool,
    duplicate_executor: bool,
}

impl RecognitionPosture {
    const fn exact() -> Self {
        Self::new(ESTATE, EXECUTOR, true, false)
    }

    const fn foreign_authority() -> Self {
        Self::new(FOREIGN_ESTATE, EXECUTOR, true, false)
    }

    const fn holder_mismatch() -> Self {
        Self::new(ESTATE, OTHER_HOLDER, true, false)
    }

    const fn unrecognized() -> Self {
        Self::new(ESTATE, EXECUTOR, false, false)
    }

    const fn duplicate() -> Self {
        Self::new(ESTATE, EXECUTOR, true, true)
    }

    const fn new(
        authority_estate: EstateCaseId,
        authority_holder: BankPrincipalId,
        authority_recognized: bool,
        duplicate_executor: bool,
    ) -> Self {
        Self {
            authority_estate,
            authority_holder,
            authority_recognized,
            duplicate_executor,
        }
    }
}

fn recognition_world(
    scenario: &str,
    posture: RecognitionPosture,
    deployment: Option<bank_server::BankProgramDeploymentGrantSeed>,
) -> RecognitionFixture {
    let identities = [
        DynamicIdentity::new(&format!("{scenario}-deceased")),
        DynamicIdentity::new(&format!("{scenario}-specialist")),
        DynamicIdentity::new(&format!("{scenario}-executor")),
        DynamicIdentity::new(&format!("{scenario}-other-holder")),
        DynamicIdentity::new(&format!("{scenario}-foreign-deceased")),
    ];
    let seed = recognition_seed(&identities, posture);
    let seed = match deployment {
        Some(grant) => seed.program_deployment_grant(grant),
        None => seed,
    };
    RecognitionFixture {
        world: runtime(seed),
        identities,
        estate: ESTATE,
        executor: EXECUTOR,
        authority: AUTHORITY,
    }
}

fn recognition_seed(
    identities: &[DynamicIdentity; 5],
    posture: RecognitionPosture,
) -> BankWorldSeed {
    let snapshot = BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
        .institution(INSTITUTION)
        .principal(DECEASED)
        .principal(SPECIALIST)
        .principal(EXECUTOR)
        .principal(OTHER_HOLDER)
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
        .build()
        .expect("the recognition fixture snapshot should be valid");
    let estate = estate_world(
        posture.authority_estate,
        posture.authority_holder,
        posture.authority_recognized,
        posture.duplicate_executor,
    );
    let principals = [
        DECEASED,
        SPECIALIST,
        EXECUTOR,
        OTHER_HOLDER,
        FOREIGN_DECEASED,
    ];
    identities.iter().enumerate().fold(
        BankWorldSeed::new(snapshot)
            .employee(BankEmployeeAssignmentSeed::new(
                ASSIGNMENT,
                INSTITUTION,
                SPECIALIST,
                EmployeeRole::EstateSpecialist,
            ))
            .estate(estate),
        |seed, (ordinal, identity)| {
            seed.principal(BankPrincipalSeed::enabled(
                principals[ordinal],
                identity.external(),
            ))
        },
    )
}

fn estate_world(
    authority_estate: EstateCaseId,
    authority_holder: BankPrincipalId,
    authority_recognized: bool,
    duplicate_executor: bool,
) -> BankEstateWorld {
    let world = BankEstateWorld::default()
        .with_branch(EstateBranch {
            id: BRANCH,
            institution: INSTITUTION,
        })
        .with_death_notice(EstateDeathNotice {
            id: NOTICE,
            subject: DECEASED,
            status: DeathNoticeStatus::Verified,
        })
        .with_death_notice(EstateDeathNotice {
            id: FOREIGN_NOTICE,
            subject: FOREIGN_DECEASED,
            status: DeathNoticeStatus::Verified,
        })
        .with_case(estate_case(ESTATE, ESTATE_ACCOUNT, DECEASED, NOTICE))
        .with_case(estate_case(
            FOREIGN_ESTATE,
            FOREIGN_ACCOUNT,
            FOREIGN_DECEASED,
            FOREIGN_NOTICE,
        ))
        .with_assignment(EstateEmployeeAssignment {
            id: ASSIGNMENT,
            principal: SPECIALIST,
            institution: INSTITUTION,
            branch: BRANCH,
            role: EmployeeRole::EstateSpecialist,
        })
        .with_estate_assignment(ESTATE, ASSIGNMENT)
        .with_legal_authority(EstateLegalAuthority {
            id: AUTHORITY,
            estate: authority_estate,
            holder: authority_holder,
            kind: LegalAuthorityKind::CourtAppointment,
            recognized: authority_recognized,
        })
        .with_grant(recognition_grant());
    if duplicate_executor {
        world.with_executor(ESTATE, EXECUTOR)
    } else {
        world
    }
}

fn estate_case(
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

fn recognition_grant() -> EstateCapabilityGrant {
    EstateCapabilityGrant {
        id: GRANT,
        grantor: DECEASED,
        grantee: SPECIALIST,
        scope: EstateCapabilityScope {
            account: None,
            estate: ESTATE,
            institution: INSTITUTION,
            branch: BRANCH,
            operation: EstateCapabilityOperation::RecognizeExecutor,
            purpose: EstateCapabilityPurpose::LegalCompliance,
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
