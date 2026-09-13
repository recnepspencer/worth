use std::sync::{Arc, Barrier};

use super::*;
use crate::{
    AuthorizationDenial, AuthorizationProviderDecision, AuthorizationProviderFailure,
    AuthorizationReplayPolicy, AuthorizationRevocationObservation, BackupRestoreIntent,
    BackupRestoreReadinessDenial, ExternalOperatorAssertion, OperationalAuthorizationPort,
    OperationalAuthorizationRequest, OperationalOperationId, OperationalTransitionId,
};

#[test]
fn provider_failures_and_wrong_possession_remain_machine_distinguishable() {
    let world = restore_world("phase-7-provider-denials");
    let target = world.restore_directory.path().join("provider-denials");
    std::fs::create_dir_all(&target).expect("target parent");
    let scope = recovery_security_scope(&world.admissible);
    let lowered = BackupRestoreIntent::from_verified_backup(
        OperationalOperationId::new("provider-denials").expect("operation"),
        world.admissible,
        &target,
        scope,
        u64::MAX,
        64,
    )
    .resolve()
    .lower()
    .expect("lowered restore");
    let failures = [
        AuthorizationProviderFailure::Unavailable,
        AuthorizationProviderFailure::Unsupported,
        AuthorizationProviderFailure::UnsupportedAssertion,
        AuthorizationProviderFailure::Timeout,
    ];
    for failure in failures {
        let denial = lowered
            .clone()
            .authorize(
                &FailingAuthorizationPort(failure),
                &operator_assertion(),
                20,
                80,
                AuthorizationReplayPolicy::SingleUse,
                AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
            )
            .expect_err("provider failure must deny");
        assert_eq!(denial, AuthorizationDenial::Provider(failure));
    }
    let denial = lowered
        .authorize(
            &WrongPossessionAuthorizationPort,
            &operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
        )
        .expect_err("wrong possession binding must deny");
    assert_eq!(
        denial,
        AuthorizationDenial::Provider(AuthorizationProviderFailure::InvalidProofOfPossession)
    );
    assert!(std::fs::read_dir(target)
        .expect("target contents")
        .next()
        .is_none());
}

#[test]
fn concurrent_single_use_consumers_have_one_durable_winner() {
    let world = restore_world("phase-7-single-use-race");
    let target = world.restore_directory.path().join("race-target");
    std::fs::create_dir_all(&target).expect("target parent");
    let scope = recovery_security_scope(&world.admissible);
    let lowered = BackupRestoreIntent::from_verified_backup(
        OperationalOperationId::new("single-use-race").expect("operation"),
        world.admissible,
        &target,
        scope,
        u64::MAX,
        64,
    )
    .resolve()
    .lower()
    .expect("lowered restore");
    let first = authorize_single_use(lowered.clone());
    let second = authorize_single_use(lowered);
    drop(world.control);
    let first_control = world.scenario.control_store();
    let second_control = world.scenario.control_store();
    let first_authority = world.authority.clone();
    let second_authority = world.authority;
    let barrier = Arc::new(Barrier::new(2));
    let first_barrier = Arc::clone(&barrier);
    let first_thread = std::thread::spawn(move || {
        first_barrier.wait();
        classify_readiness(first.ready(
            &first_control,
            OperationalTransitionId::new("single-use-first").expect("transition"),
            &first_authority,
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        ))
    });
    let second_thread = std::thread::spawn(move || {
        barrier.wait();
        classify_readiness(second.ready(
            &second_control,
            OperationalTransitionId::new("single-use-second").expect("transition"),
            &second_authority,
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        ))
    });
    let outcomes = [
        first_thread.join().expect("first consumer"),
        second_thread.join().expect("second consumer"),
    ];
    assert_eq!(outcomes.iter().filter(|outcome| **outcome == 1).count(), 1);
    assert_eq!(outcomes.iter().filter(|outcome| **outcome == 0).count(), 1);
}

#[test]
fn revoked_or_expired_authorization_cannot_cross_the_execution_boundary() {
    let world = restore_world("phase-7-revocation-expiry");
    let target = world.restore_directory.path().join("revocation-target");
    std::fs::create_dir_all(&target).expect("target parent");
    let scope = recovery_security_scope(&world.admissible);
    let lowered = BackupRestoreIntent::from_verified_backup(
        OperationalOperationId::new("revocation-expiry").expect("operation"),
        world.admissible,
        &target,
        scope,
        u64::MAX,
        64,
    )
    .resolve()
    .lower()
    .expect("lowered restore");
    let revoked = lowered
        .clone()
        .authorize(
            &ExactAuthorizationPort {
                substitute_plan: None,
            },
            &operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::Revoked {
                observed_at: 20,
                reason_fingerprint: [0x91; 32],
            },
        )
        .expect_err("revoked authorization must not be admitted");
    assert_eq!(revoked, AuthorizationDenial::AuthorizationRevoked);

    let expired = lowered
        .authorize(
            &ExactAuthorizationPort {
                substitute_plan: None,
            },
            &operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
        )
        .expect("authorization before expiry")
        .ready(
            &world.control,
            OperationalTransitionId::new("expired-before-execution").expect("transition"),
            &world.authority,
            81,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 81 },
        )
        .expect_err("expired authorization must not execute");
    assert!(matches!(
        expired,
        BackupRestoreReadinessDenial::Authorization(crate::AuthorizationConsumptionDenial::Expired)
    ));
    assert!(std::fs::read_dir(target)
        .expect("target contents")
        .next()
        .is_none());
}

#[test]
fn revocation_during_reversible_staging_stops_before_close_and_resumes_from_durable_state() {
    let world = restore_world("phase-7-mid-staging-revocation");
    let target = world.restore_directory.path().join("revoked-staging");
    std::fs::create_dir_all(&target).unwrap();
    let scope = recovery_security_scope(&world.admissible);
    let lowered = BackupRestoreIntent::from_verified_backup(
        OperationalOperationId::new("mid-staging-revocation").unwrap(),
        world.admissible,
        &target,
        scope,
        u64::MAX,
        17,
    )
    .resolve()
    .lower()
    .expect("lower reversible staging");
    let restart = lowered.clone();
    let ready = authorize_single_use(lowered)
        .ready(
            &world.control,
            OperationalTransitionId::new("consume-before-mid-stage-revocation").unwrap(),
            &world.authority,
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        )
        .unwrap();
    let denial = ready
        .execute(&RevokeBeforeFinalization)
        .expect_err("revocation before the irreversible close must stop staging");
    assert!(matches!(
        denial,
        crate::BackupRestoreExecutionDenial::Authorization(
            crate::StagingAuthorizationContinuationDenial::Revoked {
                boundary: worth_store_physical_backend::NonCurrentStagingBoundary::Finalization
            }
        )
    ));
    let staging_root = std::fs::read_dir(&target)
        .unwrap()
        .next()
        .expect("reversible residue remains for governed recovery")
        .unwrap()
        .path();
    assert!(!staging_root.join(".closed-staging").exists());

    let selection = ExactControlSelection::current(&world.authority, &world.control);
    let fencing = worth_store_authority::ControlStoreFencingAuthority::for_current_store(
        &world.authority,
        &selection,
    );
    let crate::ControlStoreTrustPosture::Selected(selected) =
        world.control.inspect_generations(&fencing)
    else {
        panic!("revoked staging must retain an exact recovery handle");
    };
    let [handle] = selected.indeterminate_recovery_staging_handles() else {
        panic!("one interrupted staging operation must be recoverable");
    };
    assert_eq!(handle.completed_media_identity(), None);
    let executed = restart
        .recover_ready(handle, &world.control, &world.authority)
        .expect("the exact lowered plan rebinds the revoked residue")
        .execute(&CurrentStagingAuthorizationPort)
        .expect("a current continuation observation resumes idempotently");
    assert!(executed
        .staged_media()
        .root()
        .join(".closed-staging")
        .is_file());
}

fn authorize_single_use(
    lowered: crate::LoweredBackupRestorePlan,
) -> crate::AuthorizedBackupRestorePlan {
    lowered
        .authorize(
            &ExactAuthorizationPort {
                substitute_plan: None,
            },
            &operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
        )
        .expect("authorization")
}

fn classify_readiness(
    result: Result<crate::ExecutionReadyBackupRestore<'_>, BackupRestoreReadinessDenial>,
) -> u8 {
    match result {
        Ok(_) => 1,
        Err(BackupRestoreReadinessDenial::Authorization(
            crate::AuthorizationConsumptionDenial::AlreadyConsumed,
        )) => 0,
        Err(denial) => {
            eprintln!("unexpected readiness denial: {denial:?}");
            2
        }
    }
}

#[derive(Clone, Copy)]
struct FailingAuthorizationPort(AuthorizationProviderFailure);

impl OperationalAuthorizationPort for FailingAuthorizationPort {
    fn authorize(
        &self,
        _request: OperationalAuthorizationRequest<'_>,
        _assertion: &ExternalOperatorAssertion,
    ) -> Result<AuthorizationProviderDecision, AuthorizationProviderFailure> {
        Err(self.0)
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
            [0x81; 32],
            request.plan_fingerprint(),
            [0x82; 32],
            request.requested_at(),
            request.expires_at(),
        ))
    }
}

struct RevokeBeforeFinalization;

impl crate::StagingAuthorizationContinuationPort for RevokeBeforeFinalization {
    fn observe_revocation(
        &self,
        request: crate::StagingAuthorizationContinuationRequest,
    ) -> Result<AuthorizationRevocationObservation, AuthorizationProviderFailure> {
        if request.boundary()
            == worth_store_physical_backend::NonCurrentStagingBoundary::Finalization
        {
            Ok(AuthorizationRevocationObservation::Revoked {
                observed_at: 40,
                reason_fingerprint: [0x93; 32],
            })
        } else {
            Ok(AuthorizationRevocationObservation::NotRevoked { observed_at: 39 })
        }
    }
}

impl crate::workflow::StagedWalApplicationPort for RevokeBeforeFinalization {
    fn apply_staged_wal(
        &self,
        request: crate::workflow::StagedWalApplicationRequest<'_>,
    ) -> Result<
        crate::workflow::StagedWalApplicationProviderReceipt,
        crate::workflow::StagedWalApplicationDenial,
    > {
        super::apply_fixture_wal(request)
    }
}
