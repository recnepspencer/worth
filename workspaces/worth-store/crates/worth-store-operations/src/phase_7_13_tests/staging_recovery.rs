use super::*;
use crate::{
    AuthorizationReplayPolicy, AuthorizationRevocationObservation, BackupRestoreIntent,
    OperationalOperationId, OperationalTransitionId,
};
use worth_store_authority::ControlStoreFencingAuthority;

struct LyingWalRuntime;

impl crate::StagingAuthorizationContinuationPort for LyingWalRuntime {
    fn observe_revocation(
        &self,
        _request: crate::StagingAuthorizationContinuationRequest,
    ) -> Result<AuthorizationRevocationObservation, crate::AuthorizationProviderFailure> {
        Ok(AuthorizationRevocationObservation::NotRevoked { observed_at: 40 })
    }
}

impl crate::workflow::StagedWalApplicationPort for LyingWalRuntime {
    fn apply_staged_wal(
        &self,
        request: crate::workflow::StagedWalApplicationRequest<'_>,
    ) -> Result<
        crate::workflow::StagedWalApplicationProviderReceipt,
        crate::workflow::StagedWalApplicationDenial,
    > {
        let source = request.replay_source();
        Ok(
            crate::workflow::StagedWalApplicationProviderReceipt::observed(
                [0xd1; 32],
                request.staging().staging_plan_fingerprint(),
                [0xd2; 32],
                source.interval(),
                source.frame_count(),
                request.target_frontier_identity(),
                true,
            ),
        )
    }
}

#[test]
fn a_runtime_cannot_claim_replay_for_a_different_wal_source() {
    let world = restore_world("phase-8-lying-runtime");
    let target_parent = world.restore_directory.path().join("non-current");
    std::fs::create_dir_all(&target_parent).unwrap();
    let security_scope = recovery_security_scope(&world.admissible);
    let ready = BackupRestoreIntent::from_verified_backup(
        OperationalOperationId::new("restore-lying-runtime").unwrap(),
        world.admissible,
        &target_parent,
        security_scope,
        u64::MAX,
        31,
    )
    .resolve()
    .lower()
    .expect("lower exact staging plan")
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
    .expect("authorize exact staging plan")
    .ready(
        &world.control,
        OperationalTransitionId::new("consume-lying-runtime").unwrap(),
        &world.authority,
        21,
        AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
    )
    .expect("staging readiness");

    assert!(matches!(
        ready.execute(&LyingWalRuntime),
        Err(crate::BackupRestoreExecutionDenial::Recovery(
            crate::workflow::BackupRestoreReplayDenial::Application(
                crate::workflow::StagedWalApplicationDenial::ReceiptMismatch
            )
        ))
    ));
    let staging_root = std::fs::read_dir(&target_parent)
        .unwrap()
        .next()
        .expect("copy residue remains resumable")
        .unwrap()
        .path();
    assert!(!staging_root.join(".closed-staging").exists());
}

#[test]
fn consumed_restore_staging_reopens_as_an_exact_resumable_handle() {
    let world = restore_world("phase-8-staging-restart");
    let target_parent = world.restore_directory.path().join("non-current");
    std::fs::create_dir_all(&target_parent).unwrap();
    let security_scope = recovery_security_scope(&world.admissible);
    let lowered = BackupRestoreIntent::from_verified_backup(
        OperationalOperationId::new("restore-staging-restart").unwrap(),
        world.admissible,
        &target_parent,
        security_scope,
        u64::MAX,
        31,
    )
    .resolve()
    .lower()
    .expect("lower exact staging plan");
    let restart_plan = lowered.clone();
    let completed_reopen_plan = lowered.clone();
    let authorized = lowered
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
        .expect("authorize exact staging plan");
    let ready = authorized
        .ready(
            &world.control,
            OperationalTransitionId::new("consume-before-crash").unwrap(),
            &world.authority,
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        )
        .expect("durably consume staging authorization");
    drop(ready); // process loss before the first owner effect
    assert!(std::fs::read_dir(&target_parent).unwrap().next().is_none());

    let selection = ExactControlSelection::current(&world.authority, &world.control);
    let fencing = ControlStoreFencingAuthority::for_current_store(&world.authority, &selection);
    let crate::ControlStoreTrustPosture::Selected(selected) =
        world.control.inspect_generations(&fencing)
    else {
        panic!("fresh process must select consumed staging authorization");
    };
    let [handle] = selected.indeterminate_recovery_staging_handles() else {
        panic!("exactly one staging recovery handle must survive");
    };
    let recovered = restart_plan
        .recover_ready(handle, &world.control, &world.authority)
        .expect("exact lowered plan rebinds durable staging state")
        .execute(&CurrentStagingAuthorizationPort)
        .expect("idempotent staging resumes to completion");
    assert!(recovered.receipt().authorization().recovered_for_resume());

    let selection = ExactControlSelection::current(&world.authority, &world.control);
    let fencing = ControlStoreFencingAuthority::for_current_store(&world.authority, &selection);
    let crate::ControlStoreTrustPosture::Selected(selected) =
        world.control.inspect_generations(&fencing)
    else {
        panic!("completed staging history must remain selectable");
    };
    let [completed] = selected.indeterminate_recovery_staging_handles() else {
        panic!("completed staging remains recoverable until publication begins");
    };
    let completed = completed.clone();
    let completed_media_identity = recovered.staged_media().content_fingerprint();
    assert_eq!(
        completed.completed_media_identity(),
        Some(completed_media_identity)
    );
    drop(recovered); // process loss after staging close, before post-verification
    let reopened = completed_reopen_plan
        .recover_ready(&completed, &world.control, &world.authority)
        .expect("completed staging rebinds from durable control state")
        .execute(&CurrentStagingAuthorizationPort)
        .expect("completed owner effects validate idempotently");
    assert_eq!(
        reopened.staged_media().content_fingerprint(),
        completed_media_identity
    );
}
