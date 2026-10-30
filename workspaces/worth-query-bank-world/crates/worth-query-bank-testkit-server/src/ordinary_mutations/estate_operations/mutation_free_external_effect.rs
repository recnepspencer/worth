//! Gate 8.6 turn 3 / Gate 8.2 O2 — mutation-free external effect through the
//! real rail.
//!
//! `RetransmitDeathNotice` emits one declared external effect and writes no
//! domain fields. The co-committed dispatch outbox is the sole local anchor
//! (R8.25 / R8.55). Lost-response recovery resolves by idempotency; a retry
//! emits the rail effect exactly once (counted at the rail, not the request).

pub use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
pub use std::time::Duration;

pub use bank_domain::estate::{
    BankEstateWorld, BranchId, CapabilityGrantId, CapabilityGrantStatus, CapabilityValidity,
    DeathNoticeId, DeathNoticeStatus, DelegationLimit, EstateAction, EstateBranch,
    EstateCapabilityGrant, EstateCapabilityOperation, EstateCapabilityPurpose,
    EstateCapabilityScope, EstateCase, EstateCaseId, EstateCaseStatus, EstateDeathNotice,
    EstateEmployeeAssignment, EstateMoment, EstateWorkflowStage,
};
pub use bank_domain::model::{
    AccountId, AccountName, BankPrincipalId, BankSnapshotVersion, EmployeeAssignmentId,
    EmployeeRole, InstitutionId,
};
pub use bank_domain::proposals::{BankIdempotencyKey, BankSnapshotBuilder};
pub use bank_domain::schema::AccountStatus;
pub use bank_external_rail::test_control::FaultScript;
pub use bank_external_rail::RailProcessHandle;
pub use bank_server::{
    queries, BankCommitReceipt, BankEmployeeAssignmentSeed, BankPrincipalSeed, BankReadControls,
    BankWorldSeed,
};
pub use worth_query_host::facade::publication::application_aftermath::WorthQueryPublishedExternalEffectFailure;

pub use super::external_effect_dispatch::rail_transport::{spawn_rail, BankEstateRailTransport};
pub use crate::facade::support::{
    block_on, request_scope, runtime, CausalCredential, DynamicIdentity, TestIdentityWorld,
};

pub const PATIENT: Duration = Duration::from_secs(5);

pub struct MutationFreeWorld {
    pub world: TestIdentityWorld,
    pub identities: [DynamicIdentity; 2],
    pub estate: EstateCaseId,
    pub notice: DeathNoticeId,
    pub deceased: BankPrincipalId,
    pub transport: Arc<BankEstateRailTransport>,
    pub _rail: RailProcessHandle,
}

pub fn assert_equivalent_retry_skips_rail_and_unseen_request_consumes_sentinel(
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
    let (recovered, fresh) = world.retransmit(binding);
    assert!(
        !fresh,
        "native recovered evidence, no original handler result"
    );
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
    let (_receipt, fresh) = world.retransmit(idempotency(62));
    assert!(
        fresh,
        "new retransmission completes through the native source owner"
    );
    assert!(dispatched.load(Ordering::Acquire));
    assert_eq!(world.transport.attempts().len(), 2);
    assert_eq!(
        world.transport.completed_notice(&correlation),
        Some(completed)
    );
}

impl MutationFreeWorld {
    pub fn commit_with(&self, binding: BankIdempotencyKey) -> BankCommitReceipt {
        let (receipt, fresh) = self.retransmit(binding);
        assert!(fresh, "native fresh completion");
        receipt
    }

    pub fn retransmit(&self, binding: BankIdempotencyKey) -> (BankCommitReceipt, bool) {
        {
            let source_actor = &self.authenticate_specialist();
            let source_scope = &request_scope();
            let source_key = &binding;
            let source_intent = bank_domain::schema::RetransmitEstateDeathNotice::try_from(
                EstateAction::RetransmitDeathNotice {
                    estate: self.estate,
                    notice: self.notice,
                    subject: self.deceased,
                },
            )
            .expect("typed estate command preserves the exact variant");
            let source_request = self.world.runtime.request(source_actor, source_scope);
            let source_operation = self
                .world
                .runtime
                .mutate(bank_server::mutations::retransmit_estate_death_notice(
                    source_intent,
                ))
                .as_principal(source_actor)
                .controls(bank_server::BankMutationControls::new(
                    (*source_scope).clone(),
                    (*source_key).clone(),
                ))
                .retain_operation()
                .unwrap_or_else(|cause| panic!("exact declared estate construction: {cause:?}"));
            let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
                source_completed,
            ) = source_operation.advance(&source_request)
            else {
                panic!("native source completion required")
            };
            let fresh = matches!(
                source_completed.evidence(),
                worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
            );
            let closed_source = source_completed
                .close_with_evidence(&source_request)
                .unwrap_or_else(|(_, cause)| panic!("lawful native source close: {cause:?}"));
            (
                bank_server::BankCommitReceipt::from_closed_source(closed_source),
                fresh,
            )
        }
    }

    pub fn authenticate_specialist(&self) -> bank_server::BankAuthenticatedPrincipal {
        let request = request_scope();
        block_on(self.world.runtime.authenticate_with(
            &self.world.authentication,
            CausalCredential::for_identity(&self.identities[1]),
            &request,
        ))
        .expect("causally mapped estate specialist should authenticate")
    }

    pub fn notice_status(&self) -> DeathNoticeStatus {
        self.world
            .runtime
            .query(queries::estate_case(self.estate))
            .as_principal(&self.authenticate_specialist())
            .controls(BankReadControls::current(request_scope(), 16).unwrap())
            .execute()
            .expect("assigned specialist should observe the death notice")
            .rows()[0]
            .death_notice()
            .status()
    }
}

pub fn mutation_free_world(scenario: &str) -> MutationFreeWorld {
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

pub fn snapshot() -> bank_domain::proposals::BankSnapshot {
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

pub fn estate_world() -> BankEstateWorld {
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

pub fn idempotency(identity: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("retransmit-death-notice-{identity}")).unwrap()
}

pub const INSTITUTION: InstitutionId = InstitutionId::new(1).unwrap();
pub const BRANCH: BranchId = BranchId::new(2).unwrap();
pub const ESTATE: EstateCaseId = EstateCaseId::new(3).unwrap();
pub const ESTATE_ACCOUNT: AccountId = AccountId::new(5).unwrap();
pub const DECEASED: BankPrincipalId = BankPrincipalId::new(7).unwrap();
pub const SPECIALIST: BankPrincipalId = BankPrincipalId::new(8).unwrap();
pub const ASSIGNMENT: EmployeeAssignmentId = EmployeeAssignmentId::new(11).unwrap();
pub const NOTICE: DeathNoticeId = DeathNoticeId::new(12).unwrap();
pub const GRANT: CapabilityGrantId = CapabilityGrantId::new(14).unwrap();
