//! Real native pressure between a persisted C4 read and temporary parser records.
//! These bytes test allocation ownership, not checkpoint-format admission.

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy, FilesystemAccessPosture,
    FilesystemMediaAdmission, PhysicalRecoveryCoordinationCapacity, PhysicalRecoveryFreshnessPort,
    PhysicalRecoveryRejoinResidentDenial, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia, ReadGrant, UnchargedRead,
};
use worth_store_physical_format::PhysicalRecordFormatDeclaration;

use super::{Failure, RecordEvidenceAllocation};

// This fixture must admit the real qualified address/open peak before it
// presses parser retention. Production recovery profiles are unchanged.
const ORIGINAL: u64 = 512;
const PARSER_BYTES: u64 = ORIGINAL - 128;

#[test]
fn persisted_read_and_parser_records_share_original_native_ceiling_until_disposal() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("parser-read-pressure");
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(&root).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("production media must issue the persisted Store identity");
    };
    let store = media.store_identity();
    media.close();
    let input = [0x5a; 128];
    let families = root.join("families");
    std::fs::create_dir_all(&families).unwrap();
    std::fs::write(families.join("checkpoint.current"), input).unwrap();
    let qualified = QualifiedRecoveryFilesystemMedia::qualify_existing(&root).unwrap();
    let freshness = PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let media = qualified.admit_persisted_store().unwrap();
    assert_eq!(media.store_identity(), store);
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let mut coordination = freshness
        .register_session()
        .unwrap()
        .admit_coordination(
            &media,
            PhysicalRecoveryCoordinationCapacity::admit(1, 4096, 1, 4096)
                .unwrap()
                .with_recovery_allocation_bytes(ORIGINAL)
                .unwrap(),
            AdmittedPhysicalRecordResidencyPolicy::canonical(format),
            None,
        )
        .unwrap();
    let mut discovery = media.bounded_discovery(1, 4096).unwrap();
    let mut window = coordination.begin_source_read_allocation().unwrap();
    let observed = window
        .read_checkpoint(&mut discovery, ReadGrant::ceiling_only())
        .observed()
        .unwrap();
    assert_eq!(observed.observed().store_identity(), store);
    assert_eq!(observed.observed().bytes(), Some(input.as_slice()));
    assert_eq!(observed.charged_bytes(), 128);
    assert_eq!(discovery.counters().bytes_read, 128);
    assert_eq!(
        window.charged_bytes(),
        0,
        "read backing is independently owned"
    );

    {
        let mut allocation = RecordEvidenceAllocation::new(&mut window);
        let mut records = allocation.reserve_records::<u64>(48).unwrap();
        records.extend(0..48);
        assert_eq!(allocation.window.charged_bytes(), PARSER_BYTES);
        let denial = allocation.reserve_records::<u64>(1).unwrap_err();
        assert_eq!(
            denial,
            Failure::ParserBacking {
                requested: 8,
                cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
                    required: ORIGINAL + 8,
                    admitted: ORIGINAL,
                },
            }
        );
        assert!(records.iter().copied().eq(0..48));
        assert_eq!(observed.observed().bytes(), Some(input.as_slice()));
        assert_eq!(discovery.counters().bytes_read, 128);
        assert_eq!(allocation.window.charged_bytes(), PARSER_BYTES);
        drop(records);
    }
    // Temporary backing retains its high-water until the concrete window ends.
    assert_eq!(window.charged_bytes(), PARSER_BYTES);
    drop(window);

    let mut retry = coordination.begin_source_read_allocation().unwrap();
    retry
        .reserve_total(PARSER_BYTES)
        .expect("disposed parser backing makes its exact capacity available");
    assert_eq!(
        retry.reserve_total(PARSER_BYTES + 1).unwrap_err(),
        PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
            required: ORIGINAL + 1,
            admitted: ORIGINAL
        }
    );
    assert_eq!(retry.charged_bytes(), PARSER_BYTES);
    drop(observed);
    retry
        .reserve_total(ORIGINAL)
        .expect("disposing the real read releases its exact 128 bytes");
    assert_eq!(retry.charged_bytes(), ORIGINAL);
    drop(retry);
    drop(coordination);
    discovery.finish();
    assert_eq!(
        std::fs::read(families.join("checkpoint.current")).unwrap(),
        input
    );
}
