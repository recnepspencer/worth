//! Gate 8.6 turn 3 / Gate 8.2 O2 — mutation-free external effect through the
//! real rail.
//!
//! `RetransmitDeathNotice` emits one declared external effect and writes no
//! domain fields. The co-committed dispatch outbox is the sole local anchor
//! (R8.25 / R8.55). Lost-response recovery resolves by idempotency; a retry
//! emits the rail effect exactly once (counted at the rail, not the request).

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

use bank_domain::estate::{
    BankEstateWorld, BranchId, CapabilityGrantId, CapabilityGrantStatus, CapabilityValidity,
    DeathNoticeId, DeathNoticeStatus, DelegationLimit, EstateAction, EstateBranch,
    EstateCapabilityGrant, EstateCapabilityOperation, EstateCapabilityPurpose,
    EstateCapabilityScope, EstateCase, EstateCaseId, EstateCaseStatus, EstateDeathNotice,
    EstateEmployeeAssignment, EstateMoment, EstateWorkflowStage,
};
use bank_domain::model::{
    AccountId, AccountName, BankPrincipalId, BankSnapshotVersion, EmployeeAssignmentId,
    EmployeeRole, InstitutionId,
};
use bank_domain::proposals::{BankIdempotencyKey, BankSnapshotBuilder};
use bank_domain::schema::AccountStatus;
use bank_external_rail::test_control::FaultScript;
use bank_external_rail::RailProcessHandle;
use bank_server::{
    queries, BankCommitReceipt, BankEmployeeAssignmentSeed, BankMutationCommitOutcome,
    BankPrincipalSeed, BankReadControls, BankWorldSeed,
};
use worth_query_host::facade::publication::application_aftermath::WorthQueryPublishedExternalEffectFailure;

use super::external_effect_dispatch::rail_transport::{spawn_rail, BankEstateRailTransport};
use crate::support::{
    block_on, request_scope, runtime, CausalCredential, DynamicIdentity, TestIdentityWorld,
};

const PATIENT: Duration = Duration::from_secs(5);

struct MutationFreeWorld {
    world: TestIdentityWorld,
    identities: [DynamicIdentity; 2],
    estate: EstateCaseId,
    notice: DeathNoticeId,
    deceased: BankPrincipalId,
    transport: Arc<BankEstateRailTransport>,
    _rail: RailProcessHandle,
}

#[test]
fn mutation_free_external_effect_co_commits_outbox_recovers_lost_response_once() {
    let world = mutation_free_world("o2-mutation-free");
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, PATIENT);
    let binding = idempotency(61);
    let status_before = world.notice_status();
    assert_eq!(status_before, DeathNoticeStatus::NotificationRequested);

    let receipt = world.commit_with(binding.clone());
    assert!(
        receipt.co_committed_dispatch_outbox(),
        "declared external effect must co-commit its outbox even with zero domain writes"
    );
    assert_eq!(
        receipt.changed_record_count(),
        2,
        "scaffolding only: Query idempotency + dispatch outbox; zero domain field writes"
    );
    assert_eq!(
        world.notice_status(),
        status_before,
        "mutation-free retransmit must not change death-notice domain state"
    );
    assert_eq!(world.transport.attempts().len(), 1);
    assert_eq!(
        receipt
            .external_dispatch_posture()
            .and_then(|posture| posture.failure()),
        Some(WorthQueryPublishedExternalEffectFailure::LostResponse)
    );

    assert_equivalent_retry_skips_rail_and_unseen_request_consumes_sentinel(
        &world, binding, &receipt,
    );
}

fn assert_equivalent_retry_skips_rail_and_unseen_request_consumes_sentinel(
    world: &MutationFreeWorld,
    binding: BankIdempotencyKey,
    receipt: &BankCommitReceipt,
) {
    let correlation = world.transport.attempts()[0].clone();
    let completed = world
        .transport
        .completed_notice(&correlation)
        .expect("the rail completed the response-lost retransmission");
    assert_eq!(completed.estate(), world.estate.get());
    assert_eq!(completed.notice(), world.notice.get());
    assert_eq!(completed.subject(), world.deceased.get());
    let dispatched = Arc::new(AtomicBool::new(false));
    let armed = dispatched.clone();
    world
        .transport
        .after_next_dispatch(move || armed.store(true, Ordering::Release));
    let retry = world.retransmit(binding);
    let BankMutationCommitOutcome::AlreadyCommitted(recovered) = retry else {
        panic!("lost-response retry must resolve by idempotency: {retry:?}");
    };
    super::external_effect_dispatch::publication_assertions::assert_recovered_commit_axes(
        receipt, &recovered,
    );
    assert!(recovered.co_committed_dispatch_outbox());
    assert_eq!(
        recovered.emitted_effect_count(),
        receipt.emitted_effect_count()
    );
    assert_eq!(
        world.transport.attempts().len(),
        1,
        "retry must emit the external effect exactly once at the rail"
    );
    assert!(!dispatched.load(Ordering::Acquire));
    assert_eq!(
        world.notice_status(),
        DeathNoticeStatus::NotificationRequested
    );

    world.transport.under(FaultScript::Succeed, PATIENT);
    assert!(matches!(
        world.retransmit(idempotency(62)),
        BankMutationCommitOutcome::Committed(_)
    ));
    assert!(dispatched.load(Ordering::Acquire));
    assert_eq!(world.transport.attempts().len(), 2);
    assert_eq!(
        world.transport.completed_notice(&correlation),
        Some(completed)
    );
}

impl MutationFreeWorld {
    fn commit_with(&self, binding: BankIdempotencyKey) -> BankCommitReceipt {
        let outcome = self.retransmit(binding);
        let BankMutationCommitOutcome::Committed(receipt) = outcome else {
            panic!("mutation-free retransmit must commit: {outcome:?}");
        };
        receipt
    }

    fn retransmit(&self, binding: BankIdempotencyKey) -> BankMutationCommitOutcome {
        self.world
            .runtime
            .retransmit_estate_death_notice_with_key(
                &self.authenticate_specialist(),
                EstateAction::RetransmitDeathNotice {
                    estate: self.estate,
                    notice: self.notice,
                    subject: self.deceased,
                },
                &binding,
                &request_scope(),
            )
            .expect("lawful retransmit should reach commit")
    }

    fn authenticate_specialist(&self) -> bank_server::BankAuthenticatedPrincipal {
        let request = request_scope();
        block_on(self.world.runtime.authenticate_with(
            &self.world.authentication,
            CausalCredential::for_identity(&self.identities[1]),
            &request,
        ))
        .expect("causally mapped estate specialist should authenticate")
    }

    fn notice_status(&self) -> DeathNoticeStatus {
        self.world
            .runtime
            .query(queries::estate_case(self.estate))
            .as_principal(&self.authenticate_specialist())
            .controls(BankReadControls::current(request_scope(), 16, 20_000).unwrap())
            .execute()
            .expect("assigned specialist should observe the death notice")
            .rows()[0]
            .death_notice()
            .status()
    }
}

fn mutation_free_world(scenario: &str) -> MutationFreeWorld {
    let rail = spawn_rail();
    let transport = Arc::new(BankEstateRailTransport::connected_to(
        rail.local_addr(),
        rail.test_control_addr(),
    ));
    let identities = [
        DynamicIdentity::new(&format!("{scenario}-deceased")),
        DynamicIdentity::new(&format!("{scenario}-specialist")),
    ];
    let seed = identities.iter().enumerate().fold(
        BankWorldSeed::new(snapshot())
            .employee(BankEmployeeAssignmentSeed::new(
                ASSIGNMENT,
                INSTITUTION,
                SPECIALIST,
                EmployeeRole::EstateSpecialist,
            ))
            .estate(estate_world()),
        |seed, (ordinal, identity)| {
            seed.principal(BankPrincipalSeed::enabled(
                [DECEASED, SPECIALIST][ordinal],
                identity.external(),
            ))
        },
    );
    let world = runtime(seed);
    world
        .runtime
        .install_external_effect_transport(transport.clone())
        .expect("bank installs its rail once per runtime");
    MutationFreeWorld {
        world,
        identities,
        estate: ESTATE,
        notice: NOTICE,
        deceased: DECEASED,
        transport,
        _rail: rail,
    }
}

fn snapshot() -> bank_domain::proposals::BankSnapshot {
    BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
        .institution(INSTITUTION)
        .principal(DECEASED)
        .principal(SPECIALIST)
        .personal_account(
            ESTATE_ACCOUNT,
            INSTITUTION,
            DECEASED,
            AccountName::new("Estate Operating").unwrap(),
            AccountStatus::Open,
        )
        .build()
        .expect("mutation-free snapshot should be valid")
}

fn estate_world() -> BankEstateWorld {
    BankEstateWorld::default()
        .with_branch(EstateBranch {
            id: BRANCH,
            institution: INSTITUTION,
        })
        .with_death_notice(EstateDeathNotice {
            id: NOTICE,
            subject: DECEASED,
            status: DeathNoticeStatus::NotificationRequested,
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
        .with_grant(EstateCapabilityGrant {
            id: GRANT,
            grantor: DECEASED,
            grantee: SPECIALIST,
            scope: EstateCapabilityScope {
                account: None,
                estate: ESTATE,
                institution: INSTITUTION,
                branch: BRANCH,
                operation: EstateCapabilityOperation::RetransmitDeathNotice,
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
        })
}

fn idempotency(identity: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("retransmit-death-notice-{identity}")).unwrap()
}

const INSTITUTION: InstitutionId = InstitutionId::new(1).unwrap();
const BRANCH: BranchId = BranchId::new(2).unwrap();
const ESTATE: EstateCaseId = EstateCaseId::new(3).unwrap();
const ESTATE_ACCOUNT: AccountId = AccountId::new(5).unwrap();
const DECEASED: BankPrincipalId = BankPrincipalId::new(7).unwrap();
const SPECIALIST: BankPrincipalId = BankPrincipalId::new(8).unwrap();
const ASSIGNMENT: EmployeeAssignmentId = EmployeeAssignmentId::new(11).unwrap();
const NOTICE: DeathNoticeId = DeathNoticeId::new(12).unwrap();
const GRANT: CapabilityGrantId = CapabilityGrantId::new(14).unwrap();
