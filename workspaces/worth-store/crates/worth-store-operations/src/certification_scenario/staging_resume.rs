use sha2::{Digest, Sha256};
use worth_store_authority::ControlStoreFencingAuthority;
use worth_store_physical_backend::NonCurrentStagingBoundary;

use crate::{
    AuthorizationReplayPolicy, AuthorizationRevocationObservation, BackupRestoreExecutionDenial,
    BackupRestoreIntent, OperationalOperationId, OperationalTransitionId,
    StagingAuthorizationContinuationDenial,
};

use super::{
    certification_operator_assertion, CurrentScenarioStagingPort, ExactScenarioAuthorizationPort,
    ExactScenarioControlSelection, OwnerBackedBackupScenario,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenarioStagingResumeReceipt {
    recovered_boundaries: u8,
    evidence_identity: [u8; 32],
}

pub fn certify_scenario_staging_resume(case: &str) -> ScenarioStagingResumeReceipt {
    let boundaries = [
        NonCurrentStagingBoundary::Allocation,
        NonCurrentStagingBoundary::Artifact { index: 0 },
        NonCurrentStagingBoundary::OwnerEffect,
        NonCurrentStagingBoundary::OwnerEffectApplied,
        NonCurrentStagingBoundary::Finalization,
    ];
    let mut digest = Sha256::new();
    digest.update(b"worth-store-s10-staging-resume-receipt-v1");
    for (index, boundary) in boundaries.into_iter().enumerate() {
        let identity = format!("{case}/{index}");
        digest.update(exercise_boundary(&identity, boundary));
    }
    ScenarioStagingResumeReceipt {
        recovered_boundaries: boundaries.len() as u8,
        evidence_identity: digest.finalize().into(),
    }
}

fn exercise_boundary(case: &str, boundary: NonCurrentStagingBoundary) -> [u8; 32] {
    let scenario = OwnerBackedBackupScenario::materialize(case);
    let control = scenario.control_store();
    let source = scenario.execute_named(case, "staging-resume-source", &control);
    let target = scenario.workspace_root().join("staging-resume-target");
    std::fs::create_dir_all(&target).unwrap();
    let lowered = BackupRestoreIntent::from_verified_backup(
        OperationalOperationId::new(format!("{case}/staging-resume")).unwrap(),
        source.into_restore_source(),
        &target,
        scenario.operational_security_scope(),
        u64::MAX,
        17,
    )
    .resolve()
    .lower()
    .unwrap();
    let restart = lowered.clone();
    let denial = lowered
        .authorize(
            &ExactScenarioAuthorizationPort,
            &certification_operator_assertion(),
            20,
            80,
            AuthorizationReplayPolicy::SingleUse,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 20 },
        )
        .unwrap()
        .ready(
            &control,
            OperationalTransitionId::new(format!("{case}/consume")).unwrap(),
            scenario.authority(),
            21,
            AuthorizationRevocationObservation::NotRevoked { observed_at: 21 },
        )
        .unwrap()
        .execute(&RevokeAtBoundary(boundary))
        .unwrap_err();
    assert!(matches!(
        denial,
        BackupRestoreExecutionDenial::Authorization(
            StagingAuthorizationContinuationDenial::Revoked {
                boundary: denied_boundary
            }
        ) if denied_boundary == boundary
    ));

    drop(control);
    let reopened = scenario.control_store();
    let selection = ExactScenarioControlSelection::current(scenario.authority(), &reopened);
    let fencing = ControlStoreFencingAuthority::for_current_store(scenario.authority(), &selection);
    let crate::ControlStoreTrustPosture::Selected(selected) =
        reopened.inspect_generations(&fencing)
    else {
        panic!("interrupted staging history must remain selected");
    };
    let [handle] = selected.indeterminate_recovery_staging_handles() else {
        panic!("one exact durable staging recovery handle required");
    };
    let executed = restart
        .recover_ready(handle, &reopened, scenario.authority())
        .unwrap()
        .execute(&CurrentScenarioStagingPort)
        .unwrap();
    assert!(executed.receipt().authorization().recovered_for_resume());
    assert!(executed
        .staged_media()
        .root()
        .join(".closed-staging")
        .is_file());

    let mut digest = Sha256::new();
    digest.update(boundary_identity(boundary));
    digest.update(executed.staged_media().plan_fingerprint());
    digest.update(executed.staged_media().content_fingerprint());
    digest.finalize().into()
}

#[derive(Clone, Copy)]
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
        CurrentScenarioStagingPort.apply_staged_wal(request)
    }
}

const fn boundary_identity(boundary: NonCurrentStagingBoundary) -> [u8; 9] {
    match boundary {
        NonCurrentStagingBoundary::Allocation => [1, 0, 0, 0, 0, 0, 0, 0, 0],
        NonCurrentStagingBoundary::Artifact { index } => {
            let bytes = index.to_be_bytes();
            [
                2, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]
        }
        NonCurrentStagingBoundary::OwnerEffect => [3, 0, 0, 0, 0, 0, 0, 0, 0],
        NonCurrentStagingBoundary::OwnerEffectApplied => [4, 0, 0, 0, 0, 0, 0, 0, 0],
        NonCurrentStagingBoundary::Finalization => [5, 0, 0, 0, 0, 0, 0, 0, 0],
    }
}

impl ScenarioStagingResumeReceipt {
    pub const fn recovered_boundaries(self) -> u8 {
        self.recovered_boundaries
    }
    pub const fn evidence_identity(self) -> [u8; 32] {
        self.evidence_identity
    }
}
