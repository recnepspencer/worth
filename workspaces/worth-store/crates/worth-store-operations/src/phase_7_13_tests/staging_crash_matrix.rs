use super::*;
use crate::{
    AuthorizationReplayPolicy, AuthorizationRevocationObservation, BackupRestoreIntent,
    OperationalOperationId, OperationalTransitionId,
};
use worth_store_authority::ControlStoreFencingAuthority;
use worth_store_physical_backend::NonCurrentStagingBoundary;

#[test]
fn restore_resumes_exactly_across_every_durable_staging_boundary() {
    let inventory = restore_world("phase-8-boundary-inventory");
    let artifact_count = inventory
        .admissible
        .custody()
        .structural()
        .materialized()
        .manifest()
        .artifacts()
        .len()
        .checked_add(1)
        .expect("manifest artifact count");
    drop(inventory);

    let mut boundaries = vec![NonCurrentStagingBoundary::Allocation];
    boundaries.extend(
        (0..artifact_count).map(|index| NonCurrentStagingBoundary::Artifact {
            index: u64::try_from(index).expect("fixture artifact index"),
        }),
    );
    boundaries.extend([
        NonCurrentStagingBoundary::OwnerEffect,
        NonCurrentStagingBoundary::OwnerEffectApplied,
        NonCurrentStagingBoundary::Finalization,
    ]);

    for (case, boundary) in boundaries.into_iter().enumerate() {
        exercise_staging_crash_boundary(case, boundary);
    }
}

fn exercise_staging_crash_boundary(case: usize, boundary: NonCurrentStagingBoundary) {
    let world = restore_world(&format!("phase-8-staging-boundary-{case}"));
    let current_before = media_snapshot(world.scenario.source_root());
    let target = world.restore_directory.path().join("non-current");
    std::fs::create_dir_all(&target).expect("non-current parent");
    let security_scope = recovery_security_scope(&world.admissible);
    let lowered = BackupRestoreIntent::from_verified_backup(
        OperationalOperationId::new(format!("restore-boundary-{case}")).unwrap(),
        world.admissible,
        &target,
        security_scope,
        u64::MAX,
        17,
    )
    .resolve()
    .lower()
    .expect("lower exact restore plan");
    let restart = lowered.clone();
    let ready = lowered
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
        .expect("authorize staging")
        .ready(
            &world.control,
            OperationalTransitionId::new(format!("consume-boundary-{case}")).unwrap(),
            &world.authority,
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        )
        .expect("consume staging authorization");
    let denial = ready
        .execute(&RevokeAtBoundary(boundary))
        .expect_err("selected boundary must interrupt execution");
    assert!(matches!(
        denial,
        crate::BackupRestoreExecutionDenial::Authorization(
            crate::StagingAuthorizationContinuationDenial::Revoked {
                boundary: denied_boundary
            }
        ) if denied_boundary == boundary
    ));
    assert_eq!(media_snapshot(world.scenario.source_root()), current_before);
    assert!(std::fs::read_dir(&target)
        .expect("staging parent")
        .all(|entry| !entry
            .expect("staging entry")
            .path()
            .join(".closed-staging")
            .exists()));

    drop(world.control);
    let control = world.scenario.control_store();
    let selection = ExactControlSelection::current(&world.authority, &control);
    let fencing = ControlStoreFencingAuthority::for_current_store(&world.authority, &selection);
    let crate::ControlStoreTrustPosture::Selected(selected) = control.inspect_generations(&fencing)
    else {
        panic!("fresh process must select interrupted staging history");
    };
    let [handle] = selected.indeterminate_recovery_staging_handles() else {
        panic!("one exact staging handle must survive boundary interruption");
    };
    assert_eq!(handle.completed_media_identity(), None);
    let recovered = restart
        .recover_ready(handle, &control, &world.authority)
        .expect("exact plan must rebind interrupted staging")
        .execute(&CurrentStagingAuthorizationPort)
        .expect("idempotent owner effects must converge after restart");
    assert!(recovered.receipt().authorization().recovered_for_resume());
    assert!(recovered
        .staged_media()
        .root()
        .join(".closed-staging")
        .is_file());
    assert_eq!(media_snapshot(world.scenario.source_root()), current_before);
}

struct RevokeAtBoundary(NonCurrentStagingBoundary);

impl crate::StagingAuthorizationContinuationPort for RevokeAtBoundary {
    fn observe_revocation(
        &self,
        request: crate::StagingAuthorizationContinuationRequest,
    ) -> Result<AuthorizationRevocationObservation, crate::AuthorizationProviderFailure> {
        if request.boundary() == self.0 {
            Ok(AuthorizationRevocationObservation::Revoked {
                observed_at: 40,
                reason_fingerprint: [0xe8; 32],
            })
        } else {
            Ok(AuthorizationRevocationObservation::NotRevoked { observed_at: 40 })
        }
    }
}

impl crate::workflow::StagedWalApplicationPort for RevokeAtBoundary {
    fn apply_staged_wal(
        &self,
        request: crate::workflow::StagedWalApplicationRequest<'_>,
    ) -> Result<
        crate::workflow::StagedWalApplicationProviderReceipt,
        crate::workflow::StagedWalApplicationDenial,
    > {
        apply_fixture_wal(request)
    }
}
