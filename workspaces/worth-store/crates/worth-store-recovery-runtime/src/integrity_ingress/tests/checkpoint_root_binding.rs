use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia,
};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, FreeSpaceBlockReference, FreeSpaceKey,
    PhysicalRecordFormatDeclaration, RecordAllocationClass, RecordArtifactFile,
};
use worth_store_physical_integrity::{PhysicalDamageCause, PhysicalIntegrityRejection};

use super::super::{
    admit_observed_root_manifest, IntegrityAdmittedRecoveryArtifact,
    RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressObservationOutcome,
    RecoveryIntegrityIngressRejection,
};

#[test]
fn exact_checkpoint_root_absence_and_corruption_stop_before_owner_projection() {
    for scenario in ["intact", "absent", "checksum"] {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join(scenario);
        let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(&root).unwrap()).unwrap();
        let TransitionOutcome::Success(media) = runtime
            .try_admit_filesystem_media(FilesystemMediaAdmission::production(
                FilesystemAccessPosture::CoordinatedServiceAccount,
            ))
            .into_raw()
        else {
            panic!("production media admission")
        };
        let store = media.store_identity();
        media.close();
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let key = FreeSpaceKey::new(RecordAllocationClass::Extent, 1).unwrap();
        let manifest = DurablePhysicalRootManifest::builder(4, 7, 4, 19)
            .free_space_root(Some(
                FreeSpaceBlockReference::new(4, 1, 0, 17, key, key).unwrap(),
            ))
            .admit()
            .unwrap();
        let mut bytes = manifest.encode(format);
        let exact_length = bytes.len() as u64;
        if scenario == "checksum" {
            bytes[56] ^= 1;
        }
        let roots = root.join("families/records/roots");
        std::fs::create_dir_all(&roots).unwrap();
        if scenario != "absent" {
            std::fs::write(
                roots.join(RecordArtifactFile::RootManifest { generation: 4 }.file_name()),
                bytes,
            )
            .unwrap();
        }
        let media = QualifiedRecoveryFilesystemMedia::qualify_existing(&root)
            .unwrap()
            .admit_persisted_store()
            .unwrap();
        let mut discovery = media.bounded_discovery(1, 4096).unwrap();
        let observed = discovery.read_root_manifest(4, 4096).unwrap();
        let mut counters = RecoveryIntegrityIngressCounters::default();
        assert!(matches!(
            admit_observed_root_manifest(&observed, store, format, 0, &mut counters),
            Err(RecoveryIntegrityIngressRejection::ScopeMismatch)
        ));
        assert_eq!(
            counters.attempted, 0,
            "invalid source address never enters validation"
        );
        let attempt =
            admit_observed_root_manifest(&observed, store, format, 4, &mut counters).unwrap();
        assert_eq!(attempt.observation().scope().store_identity(), store);
        assert_eq!(attempt.observation().scope().root_generation(), Some(4));
        assert_eq!(counters.attempted, 1);
        assert_eq!(counters.owner_projection_entries, 0);
        assert_eq!(counters.owner_decoder_entries, 0);
        match scenario {
            "intact" => {
                assert_eq!(counters.admitted, 1);
                let IntegrityAdmittedRecoveryArtifact::RootManifest(admitted) =
                    attempt.into_outcome().unwrap()
                else {
                    panic!("exact source root family")
                };
                assert_eq!(admitted.project_for_recovery(&mut counters).0, manifest);
                assert_eq!(counters.owner_projection_entries, 1);
            }
            "absent" => {
                assert_eq!(counters.rejected_absent, 1);
                assert_eq!(
                    attempt.observation().outcome(),
                    RecoveryIntegrityIngressObservationOutcome::Rejected(
                        RecoveryIntegrityIngressRejection::Absent
                    )
                );
            }
            "checksum" => {
                assert_eq!(counters.rejected_damaged, 1);
                let RecoveryIntegrityIngressObservationOutcome::Rejected(
                    RecoveryIntegrityIngressRejection::Integrity(
                        PhysicalIntegrityRejection::Damaged(damage),
                    ),
                ) = attempt.observation().outcome()
                else {
                    panic!("exact checksum damage")
                };
                assert_eq!(damage.cause(), PhysicalDamageCause::ChecksumMismatch);
                assert_eq!(damage.damaged_range().offset(), 0);
                assert_eq!(damage.damaged_range().length(), exact_length);
            }
            _ => unreachable!(),
        }
        discovery.finish();
    }
}
