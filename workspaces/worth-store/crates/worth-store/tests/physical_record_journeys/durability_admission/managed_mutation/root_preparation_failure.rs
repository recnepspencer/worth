use super::*;
use worth_store::physical_runtime::{
    PhysicalMutationIndeterminateStage, PhysicalRootCandidateWriteFailureCause,
    PhysicalRootCandidateWriteFailurePosture, PhysicalRootPreparationEffectPosture,
    PhysicalRootPublicationPreparationFailureCause, RecordAppendDenial,
};

#[test]
fn managed_root_preparation_preserves_cause_without_erasing_durable_wal() {
    for after_write in [false, true] {
        let parent = tempfile::tempdir().unwrap();
        let serving = serving_from_initialization(&parent.path().join("store"));
        let (_, placement, _) = configuration();
        let prepared = prepare_records(&serving, placement, [0xd8; 32], &[b"root-failure"]);
        let identity = prepared.mutation_identity();
        let gate = serving.certification_pause_physical_mutation_at(
            CertificationPhysicalMutationCheckpoint::AfterDataSettlement,
        );
        let handle = prepared.start();
        assert!(
            gate.await_arrival(),
            "the real data-settled boundary must be reached"
        );
        if after_write {
            serving.certification_reject_next_candidate_publication_after_physical_write();
        } else {
            serving.certification_reject_next_candidate_retention_before_effect();
        }
        gate.release();
        let PhysicalMutationOutcome::Indeterminate(fate) = handle.wait() else {
            panic!("root failure after durable WAL must remain globally indeterminate");
        };
        assert_eq!(fate.mutation_identity(), identity);
        assert_eq!(
            fate.stage(),
            PhysicalMutationIndeterminateStage::RootPreparation
        );
        assert_eq!(fate.completed_effect_count(), 1);
        let evidence = fate
            .root_preparation_failure()
            .expect("root cause must survive translation");
        assert_eq!(
            evidence.effect_posture(),
            if after_write {
                PhysicalRootPreparationEffectPosture::InspectionRequired
            } else {
                PhysicalRootPreparationEffectPosture::NotStarted
            },
        );
        let PhysicalRootPublicationPreparationFailureCause::CandidateWrite {
            completed_artifact_count,
            cause: PhysicalRootCandidateWriteFailureCause::Residency { denial, posture },
            ..
        } = evidence.cause()
        else {
            panic!("candidate-residency failure must retain its typed boundary: {evidence:?}");
        };
        assert_eq!(*completed_artifact_count, 0);
        assert!(matches!(
            denial,
            RecordAppendDenial::ResidencyUnavailable(_)
        ));
        assert_eq!(
            *posture,
            if after_write {
                PhysicalRootCandidateWriteFailurePosture::EffectPossible
            } else {
                PhysicalRootCandidateWriteFailurePosture::ProvenNoEffect
            },
        );
        assert_eq!(
            fate.diagnostic_evidence().root_preparation_failure(),
            Some(evidence)
        );
        assert_eq!(serving.certification_pending_publication_count(), 1);
        let shutdown = serving.close();
        assert_eq!(shutdown.mutations().indeterminate(), 1);
        assert_eq!(shutdown.mutations().proven_no_effect(), 0);
    }
}
