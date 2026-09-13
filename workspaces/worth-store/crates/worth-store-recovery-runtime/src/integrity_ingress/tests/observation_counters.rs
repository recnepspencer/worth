use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia,
};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::PhysicalRecordFormatDeclaration;
use worth_store_physical_integrity::{
    BootstrapCatalogIntegrityValidation, IndeterminatePhysicalIntegrityCause,
    IndeterminatePhysicalIntegrityPosture, PhysicalArtifactScope, PhysicalBlastRadius,
    PhysicalByteRange, PhysicalDamageCause, PhysicalDamageLocalization, PhysicalIntegrityRejection,
    PhysicalIntegrityVersionAxis, UnknownPhysicalIntegrityCause, UnknownPhysicalIntegrityPosture,
    UnsupportedPhysicalIntegrityVersion,
};

use super::super::{
    admitted_artifact::IntegrityAdmittedRecoveryArtifact, observe_absent_recovery_artifact,
    RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressObservation,
    RecoveryIntegrityIngressObservationOutcome, RecoveryIntegrityIngressRejection,
};

#[test]
fn counters_preserve_every_validator_and_binding_rejection_class() {
    let scope = scope();
    let damage = PhysicalIntegrityRejection::Damaged(PhysicalDamageLocalization::new(
        scope,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CompleteArtifact,
    ));
    let unsupported =
        PhysicalIntegrityRejection::Unsupported(UnsupportedPhysicalIntegrityVersion::new(
            scope,
            PhysicalIntegrityVersionAxis::EnvelopeSchema,
            u32::MAX,
        ));
    let unknown = PhysicalIntegrityRejection::Unknown(UnknownPhysicalIntegrityPosture::new(
        scope,
        UnknownPhysicalIntegrityCause::UnrecognizedArtifact,
    ));
    let indeterminate =
        PhysicalIntegrityRejection::Indeterminate(IndeterminatePhysicalIntegrityPosture::new(
            scope,
            IndeterminatePhysicalIntegrityCause::StableRangeNotProven,
            Some(scope.byte_range()),
        ));
    let rejections = [damage, unsupported, unknown, indeterminate];
    let observed = observed_artifact();
    let mut counters = RecoveryIntegrityIngressCounters::default();
    for rejection in rejections {
        let attempt = IntegrityAdmittedRecoveryArtifact::bind_bootstrap_catalog(
            &observed,
            scope,
            BootstrapCatalogIntegrityValidation::Rejected(rejection),
            &mut counters,
        );
        assert_eq!(
            attempt.observation().outcome(),
            RecoveryIntegrityIngressObservationOutcome::Rejected(
                RecoveryIntegrityIngressRejection::Integrity(rejection)
            )
        );
        assert_eq!(attempt.observation().scope(), scope);
    }
    let absent = observe_absent_recovery_artifact(&observed, scope, &mut counters);
    assert_eq!(
        absent.observation().outcome(),
        RecoveryIntegrityIngressObservationOutcome::Rejected(
            RecoveryIntegrityIngressRejection::Absent
        )
    );
    let local_rejections = [
        RecoveryIntegrityIngressRejection::ConflictingDuplication {
            observed_sources: 2,
        },
        RecoveryIntegrityIngressRejection::ScopeMismatch,
    ];
    for rejection in local_rejections {
        let observation = RecoveryIntegrityIngressObservation::rejected(scope, rejection);
        assert_eq!(
            observation.outcome(),
            RecoveryIntegrityIngressObservationOutcome::Rejected(rejection)
        );
        assert_eq!(observation.scope(), scope);
        counters.record(observation);
    }
    assert_eq!(counters.attempted, 7);
    assert_eq!(counters.admitted, 0);
    assert_eq!(counters.rejected_damaged, 1);
    assert_eq!(counters.rejected_unsupported, 1);
    assert_eq!(counters.rejected_unknown, 1);
    assert_eq!(counters.rejected_indeterminate, 1);
    assert_eq!(counters.rejected_absent, 1);
    assert_eq!(counters.rejected_conflicting, 1);
    assert_eq!(counters.rejected_source_binding, 1);
    assert_eq!(counters.owner_projection_entries, 0);
}

fn observed_artifact() -> worth_store::physical_runtime::ObservedRecoveryArtifact {
    let parent = tempfile::tempdir().expect("test parent");
    let root = parent.path().join("ingress-rejection-observation");
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(root.clone()).expect("declared root"))
            .expect("ordinary runtime admission");
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let media = match runtime.try_admit_filesystem_media(admission).into_raw() {
        TransitionOutcome::Success(media) => media,
        _ => panic!("ordinary media initialization failed"),
    };
    let _ = media.close();
    let media = QualifiedRecoveryFilesystemMedia::qualify_existing(&root)
        .unwrap()
        .admit_persisted_store()
        .unwrap();
    let mut discovery = media.bounded_discovery(1, 256).unwrap();
    let observed = discovery.read_bootstrap_catalog(256).unwrap();
    drop(discovery.finish());
    observed
}

fn scope() -> PhysicalArtifactScope {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([0x51; 16]).unwrap(),
    )
    .published_identity();
    PhysicalArtifactScope::bootstrap_catalog(
        store,
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        PhysicalByteRange::new(0, 82).unwrap(),
    )
}
