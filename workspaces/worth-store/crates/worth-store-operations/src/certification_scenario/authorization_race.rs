use std::sync::{Arc, Barrier};

use sha2::{Digest, Sha256};

use crate::{
    AuthorizationConsumptionDenial, AuthorizationDenial, AuthorizationProviderDecision,
    AuthorizationProviderFailure, AuthorizationReplayPolicy, AuthorizationRevocationObservation,
    BackupRestoreIntent, BackupRestoreReadinessDenial, ExternalOperatorAssertion,
    OperationalAuthorizationPort, OperationalAuthorizationRequest, OperationalOperationId,
    OperationalTransitionId,
};

use super::{
    certification_operator_assertion, ExactScenarioAuthorizationPort, OwnerBackedBackupScenario,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenarioAuthorizationRaceReceipt {
    plan_fingerprint: [u8; 32],
    winner_count: u8,
    consumed_replay_denials: u8,
    evidence_identity: [u8; 32],
}

#[expect(
    clippy::disallowed_methods,
    reason = "certification-harness thread-spawn=2: Certification drives independent actors to expose concurrency behavior."
)]
pub fn certify_scenario_authorization_race(case: &str) -> ScenarioAuthorizationRaceReceipt {
    let scenario = OwnerBackedBackupScenario::materialize(case);
    let control = scenario.control_store();
    let source = scenario.execute_named(case, "authorization-race-source", &control);
    let target = scenario.workspace_root().join("authorization-race-target");
    std::fs::create_dir_all(&target).expect("authorization race target");
    let lowered = BackupRestoreIntent::from_verified_backup(
        OperationalOperationId::new(format!("{case}/authorization-race")).unwrap(),
        source.into_restore_source(),
        &target,
        scenario.operational_security_scope(),
        u64::MAX,
        64 * 1024,
    )
    .resolve()
    .lower()
    .expect("lower exact authorization race plan");
    let plan_fingerprint = lowered.explanation().plan_fingerprint();

    let wrong_scope = lowered
        .clone()
        .authorize(
            &WrongPossessionAuthorizationPort,
            &certification_operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
        )
        .expect_err("wrong possession must lose before execution readiness");
    assert_eq!(
        wrong_scope,
        AuthorizationDenial::Provider(AuthorizationProviderFailure::InvalidProofOfPossession)
    );

    let first = authorize(lowered.clone());
    let second = authorize(lowered);
    drop(control);
    let first_control = scenario.control_store();
    let second_control = scenario.control_store();
    let first_authority = scenario.authority().clone();
    let second_authority = scenario.authority().clone();
    let barrier = Arc::new(Barrier::new(2));
    let first_barrier = Arc::clone(&barrier);
    let first_transition = format!("{case}/race-first");
    let second_transition = format!("{case}/race-second");
    let first = std::thread::spawn(move || {
        first_barrier.wait();
        classify(first.ready(
            &first_control,
            OperationalTransitionId::new(first_transition).unwrap(),
            &first_authority,
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        ))
    });
    let second = std::thread::spawn(move || {
        barrier.wait();
        classify(second.ready(
            &second_control,
            OperationalTransitionId::new(second_transition).unwrap(),
            &second_authority,
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        ))
    });
    let outcomes = [first.join().unwrap(), second.join().unwrap()];
    let winner_count = outcomes
        .iter()
        .filter(|value| **value == RaceOutcome::Winner)
        .count() as u8;
    let consumed_replay_denials = outcomes
        .iter()
        .filter(|value| **value == RaceOutcome::ConsumedReplay)
        .count() as u8;
    assert_eq!((winner_count, consumed_replay_denials), (1, 1));

    let mut digest = Sha256::new();
    digest.update(b"worth-store-s10-authorization-race-receipt-v1");
    digest.update(plan_fingerprint);
    digest.update([winner_count, consumed_replay_denials]);
    ScenarioAuthorizationRaceReceipt {
        plan_fingerprint,
        winner_count,
        consumed_replay_denials,
        evidence_identity: digest.finalize().into(),
    }
}

fn authorize(lowered: crate::LoweredBackupRestorePlan) -> crate::AuthorizedBackupRestorePlan {
    lowered
        .authorize(
            &ExactScenarioAuthorizationPort,
            &certification_operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
        )
        .expect("valid exact-plan authorization")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RaceOutcome {
    Winner,
    ConsumedReplay,
}

fn classify(
    result: Result<crate::ExecutionReadyBackupRestore<'_>, BackupRestoreReadinessDenial>,
) -> RaceOutcome {
    match result {
        Ok(_) => RaceOutcome::Winner,
        Err(BackupRestoreReadinessDenial::Authorization(
            AuthorizationConsumptionDenial::AlreadyConsumed,
        )) => RaceOutcome::ConsumedReplay,
        Err(denial) => panic!("unexpected authorization race denial: {denial:?}"),
    }
}

struct WrongPossessionAuthorizationPort;

impl OperationalAuthorizationPort for WrongPossessionAuthorizationPort {
    fn authorize(
        &self,
        request: OperationalAuthorizationRequest<'_>,
        _assertion: &ExternalOperatorAssertion,
    ) -> Result<AuthorizationProviderDecision, AuthorizationProviderFailure> {
        Ok(AuthorizationProviderDecision::authorized(
            request.plan_fingerprint(),
            request.plan_fingerprint(),
            [0x9f; 32],
            request.requested_at(),
            request.expires_at(),
        ))
    }
}

impl ScenarioAuthorizationRaceReceipt {
    pub const fn plan_fingerprint(self) -> [u8; 32] {
        self.plan_fingerprint
    }
    pub const fn winner_count(self) -> u8 {
        self.winner_count
    }
    pub const fn consumed_replay_denials(self) -> u8 {
        self.consumed_replay_denials
    }
    pub const fn evidence_identity(self) -> [u8; 32] {
        self.evidence_identity
    }
}
