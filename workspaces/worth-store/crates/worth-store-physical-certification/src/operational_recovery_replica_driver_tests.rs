use worth_foundational::CanonicalDigestId;
use worth_store_operations::{
    AuthorizationReplayPolicy, AuthorizationRevocationObservation, OperationalOperationId,
    OperationalSecurityScope, OperationalTransitionId, ReplicaBootstrapIntent,
};
use worth_store_physical_isolation::RecoverySourceLeaseRegistry;
use worth_store_replication::{
    LoweredReplicaBootstrapPlan, ReplicaBootstrapDenial, ReplicaBootstrapExecutionCounters,
    ReplicaBootstrapExecutionPort, ReplicaBootstrapExecutionReport, ReplicaRecoveryFrontier,
    ReplicationPeerId,
};

use super::operational_recovery_authorization_fixture::{operator_assertion, ExactAuthorization};
use super::operational_recovery_replica_driver_fixture::DisasterRecoveryFixture;
use crate::{
    DrivenOperationalControlStore, DrivenOperationalTransition,
    OperationalRecoveryControlTransitionKind as Control, OperationalRecoveryCrashCutDenial,
    OperationalRecoveryCrashCutEvidence, OperationalRecoveryProductionDriver,
    OperationalRecoveryYieldpoint as Point,
};

#[test]
fn replica_bootstrap_owner_writes_flow_through_driven_durable_control_port() {
    let run = run_bootstrap(None);
    assert!(run.persist_succeeded);
    assert_eq!(run.durable_generation, 2);
    assert_eq!(run.reopened_generation(), 2);
    for kind in [
        Control::AuthorizationConsumption,
        Control::ReplicaBootstrapTransfer,
    ] {
        assert!(run
            .trace
            .reached()
            .contains(&Point::BeforeDurableControlTransition(kind)));
        assert!(run
            .trace
            .reached()
            .contains(&Point::AfterDurableControlTransition(kind)));
    }
    assert_eq!(run.trace.control_artifact_identities().len(), 2);
}

#[test]
fn replica_bootstrap_transfer_cutpoints_reopen_the_exact_durable_prefix() {
    let before = run_bootstrap(Some(Point::BeforeDurableControlTransition(
        Control::ReplicaBootstrapTransfer,
    )));
    assert!(!before.persist_succeeded);
    assert_eq!(before.durable_generation, 1);
    assert_eq!(before.reopened_generation(), 1);
    assert!(before
        .trace
        .reached()
        .contains(&Point::BeforeDurableControlTransition(
            Control::ReplicaBootstrapTransfer,
        )));
    assert!(!before
        .trace
        .reached()
        .contains(&Point::AfterDurableControlTransition(
            Control::ReplicaBootstrapTransfer,
        )));

    let after = run_bootstrap(Some(Point::AfterDurableControlTransition(
        Control::ReplicaBootstrapTransfer,
    )));
    assert!(!after.persist_succeeded);
    assert_eq!(after.durable_generation, 2);
    assert_eq!(after.reopened_generation(), 2);
    assert!(after
        .trace
        .reached()
        .contains(&Point::AfterDurableControlTransition(
            Control::ReplicaBootstrapTransfer,
        )));
    assert_eq!(after.trace.control_artifact_identities().len(), 2);

    assert_eq!(
        before.crash_evidence_denial(),
        OperationalRecoveryCrashCutDenial::SameProcess
    );
    assert_eq!(
        after.crash_evidence_denial(),
        OperationalRecoveryCrashCutDenial::SameProcess
    );
}

#[test]
fn control_crash_evidence_rejects_same_session_and_same_process_reopen() {
    let before = run_bootstrap(Some(Point::BeforeDurableControlTransition(
        Control::ReplicaBootstrapTransfer,
    )));
    assert_eq!(
        OperationalRecoveryCrashCutEvidence::from_fresh_process_reopen(
            Point::BeforeDurableControlTransition(Control::ReplicaBootstrapTransfer),
            &before.trace,
            before.before_transition,
            before.before_transition,
        )
        .unwrap_err(),
        OperationalRecoveryCrashCutDenial::SameControlSession
    );
    assert_eq!(
        before.crash_evidence_denial(),
        OperationalRecoveryCrashCutDenial::SameProcess
    );
}

struct BootstrapRun {
    fixture: DisasterRecoveryFixture,
    trace: crate::OperationalRecoveryDriverTrace,
    pause: Option<Point>,
    before_transition: worth_store_operations::OperationalControlSessionObservation,
    durable_generation: u64,
    persist_succeeded: bool,
}

impl BootstrapRun {
    fn reopened_generation(&self) -> u64 {
        self.fixture
            .control_store()
            .observe_selection_coordinates()
            .unwrap()
            .unwrap()
            .generation()
            .get()
    }

    fn reopened_observation(&self) -> worth_store_operations::OperationalControlSessionObservation {
        self.fixture.control_store().session_observation().unwrap()
    }

    fn crash_evidence_denial(&self) -> OperationalRecoveryCrashCutDenial {
        OperationalRecoveryCrashCutEvidence::from_fresh_process_reopen(
            self.pause.expect("crash run has a pause"),
            &self.trace,
            self.before_transition,
            self.reopened_observation(),
        )
        .unwrap_err()
    }
}

fn run_bootstrap(pause: Option<Point>) -> BootstrapRun {
    let fixture = DisasterRecoveryFixture::materialize();
    let authority = worth_store_test_support::layout_integrity_authority("s10-driver-bootstrap");
    let operation = OperationalOperationId::new("s10-driven-bootstrap").unwrap();
    let verified = fixture.verify();
    let resolved = verified
        .resolve_bootstrap_source_cut(operation.stable_fingerprint(), 2, 32 * 1024)
        .unwrap();
    let lease = RecoverySourceLeaseRegistry::open(fixture.lease_root())
        .unwrap()
        .admit_bootstrap_source_cut(resolved)
        .unwrap()
        .lease();
    let security = OperationalSecurityScope::from_admission(authority.security_scope().receipt());
    let lowered = ReplicaBootstrapIntent::new(
        operation.clone(),
        ReplicationPeerId::from_declared_peer("replica-b").unwrap(),
        [0xD4; 32],
        authority.current_authority().authority_identity(),
        security,
    )
    .unwrap()
    .resolve(verified, lease)
    .unwrap()
    .lower()
    .unwrap();
    let authorized = lowered
        .authorize(
            &ExactAuthorization,
            &operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
        )
        .unwrap();
    let (trace, before_transition, durable_generation, persist_succeeded) = {
        let control = fixture.control_store();
        let driver = pause.map_or_else(
            OperationalRecoveryProductionDriver::uninterrupted,
            OperationalRecoveryProductionDriver::pause_once_at,
        );
        let driven = DrivenOperationalControlStore::new(&control, &driver);
        let ready = authorized
            .ready_with_certification_control_store(
                &control,
                &driven,
                OperationalTransitionId::new("bootstrap-authorization").unwrap(),
                authority.current_authority(),
                30,
                AuthorizationRevocationObservation::NotRevoked { observed_at: 30 },
            )
            .unwrap();
        let mut owner = ExactBootstrapOwner {
            frontier: fixture.frontier(),
            target_identity: CanonicalDigestId::new([0xD4; 32]),
        };
        let transferred = match driver.bootstrap_transfer(ready, &mut owner).unwrap() {
            DrivenOperationalTransition::Completed(transferred) => transferred,
            other => panic!("uninterrupted driver returned {other:?}"),
        };
        let before_transition = control.session_observation().unwrap();
        let persisted = driver.persist_bootstrap_transfer(
            &transferred,
            OperationalTransitionId::new("bootstrap-transfer").unwrap(),
        );
        let persist_succeeded = match persisted {
            Ok(DrivenOperationalTransition::Completed(executed)) => {
                assert_eq!(executed.receipt().durable_target_identity(), [0xD4; 32]);
                true
            }
            Err(_) => false,
            Ok(other) => panic!("durable append decorator returned {other:?}"),
        };
        let trace = driver.trace();
        let durable_generation = control
            .observe_selection_coordinates()
            .unwrap()
            .unwrap()
            .generation()
            .get();
        (
            trace,
            before_transition,
            durable_generation,
            persist_succeeded,
        )
    };
    BootstrapRun {
        fixture,
        trace,
        pause,
        before_transition,
        durable_generation,
        persist_succeeded,
    }
}

struct ExactBootstrapOwner {
    frontier: ReplicaRecoveryFrontier,
    target_identity: CanonicalDigestId,
}

impl ReplicaBootstrapExecutionPort for ExactBootstrapOwner {
    fn execute_replica_bootstrap(
        &mut self,
        plan: &LoweredReplicaBootstrapPlan,
    ) -> Result<ReplicaBootstrapExecutionReport, ReplicaBootstrapDenial> {
        Ok(ReplicaBootstrapExecutionReport::from_replication_owner(
            plan.source_lease_identity(),
            self.frontier,
            *self.target_identity.bytes(),
            ReplicaBootstrapExecutionCounters::measured(64, 64, 2, 16).unwrap(),
        ))
    }
}
