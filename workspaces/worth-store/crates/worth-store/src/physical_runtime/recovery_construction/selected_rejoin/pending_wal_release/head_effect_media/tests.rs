use super::*;
use sha2::{Digest, Sha256};
#[cfg(feature = "certification-test-authority")]
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension as Dimension,
};
use worth_store_physical_format::{
    BlobGenerationPublicationV1, BlobReclaimSourceBasisV1, PersistedRecordIdentity,
    ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1, ReleaseCustodyHeadMutationV1,
    ReleaseCustodyHeadTransitionLimitsV1, ReleaseCustodyHeadTransitionV1,
    ReleasedGenerationReclaimBasisV1,
};

#[path = "tests/fixture.rs"]
pub(super) mod fixture;
#[cfg(feature = "certification-test-authority")]
#[path = "tests/serving_freshness.rs"]
mod serving_freshness;

// A format-owner planned first upsert, not a fabricated recovery authority.
fn effect_fixture() -> (
    PersistedReleaseCustodyHeadEffectV1,
    PhysicalRecordFormatDeclaration,
) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let record = |ordinal| PersistedRecordIdentity::new([1; 16], ordinal).unwrap();
    let publication = BlobGenerationPublicationV1::new(
        [7; 16],
        [2; 16],
        [3; 16],
        3,
        record(3),
        [4; 32],
        8,
        [5; 32],
        64 * 1024,
        [6; 32],
    )
    .unwrap();
    let basis = ReleasedGenerationReclaimBasisV1::new(
        publication,
        record(4),
        Sha256::digest(publication.encode()).into(),
        [6; 32],
    )
    .unwrap();
    let next = ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new(basis.object(), basis.generation()).unwrap(),
        record(9),
        [5; 32],
        record(7),
        [8; 32],
        record(8),
        [9; 32],
        BlobReclaimSourceBasisV1::ReleasedGeneration(basis).digest(publication.store()),
        None,
        11,
        1,
        false,
    )
    .unwrap();
    let limits = ReleaseCustodyHeadTransitionLimitsV1::new(1, 3, 64 * 1024).unwrap();
    let planned = ReleaseCustodyHeadTransitionV1::plan(
        None,
        1,
        &[],
        ReleaseCustodyHeadMutationV1::Upsert {
            expected_prior: None,
            next,
        },
        12,
        6,
        format,
        limits,
    )
    .unwrap();
    let effect = PersistedReleaseCustodyHeadEffectV1::new_upsert(
        6,
        11,
        basis,
        vec![],
        planned,
        format,
        limits,
    )
    .unwrap();
    (effect, format)
}

#[test]
fn effect_verification_scratch_requires_native_capacity_before_media_read() {
    let (_directory, media, coordination) = fixture::coordination();
    let (effect, format) = effect_fixture();
    let mut window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let mut discovery = media.bounded_discovery(64, 2 << 20).unwrap();
    let scratch = effect.verification_additional_peak_bytes(format).unwrap();
    let held = window
        .reserve_owned(window.recovery_byte_limit() - 1)
        .unwrap();
    let denied = observe_effect_nodes(&mut discovery, &mut window, &effect, 11, format, scratch);
    assert!(matches!(denied, Err(Denial::Resident(_))));
    assert_eq!(discovery.counters().addressed_artifacts_read, 0);
    drop(held);
    assert_eq!(window.charged_bytes(), 0);
}

#[test]
fn effect_joint_window_denies_before_media_read_even_with_native_owner() {
    let (_directory, media, coordination) = fixture::coordination();
    let (effect, format) = effect_fixture();
    let mut window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let mut discovery = media.bounded_discovery(64, 2 << 20).unwrap();
    let count = effect.source_path().len() + effect.node_writes().len();
    let required = effect
        .verification_additional_peak_bytes(format)
        .unwrap()
        .max(
            count as u64 * std::mem::size_of::<SelectedArtifactSlice>() as u64
                + u64::from(format.page_size().bytes()),
        );
    let denied = observe_effect_nodes(
        &mut discovery,
        &mut window,
        &effect,
        11,
        format,
        required - 1,
    );
    assert!(matches!(denied, Err(Denial::BoundExceeded)));
    assert_eq!(discovery.counters().addressed_artifacts_read, 0);
    assert_eq!(window.charged_bytes(), 0);
}

#[test]
#[cfg(feature = "certification-test-authority")]
fn actual_c4_effect_reads_retain_only_funded_witness_backing() {
    let (directory, media, coordination) = fixture::coordination();
    let (effect, format) = effect_fixture();
    let paths = fixture::install_planned_nodes(directory.path(), &effect);
    let observer = coordination.certification_residency_allocations();
    let before = observer.snapshot();
    let mut window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let mut discovery = media.bounded_discovery(64, 2 << 20).unwrap();

    let witnessed = observe_effect_nodes(&mut discovery, &mut window, &effect, 11, format, 2 << 20)
        .expect("format-planned frames must pass actual native C4 reads");
    assert_eq!(witnessed.slices().len(), paths.len());
    assert_eq!(
        discovery.counters().addressed_artifacts_read,
        paths.len() as u64
    );
    assert!(witnessed.owned_heap_bytes().unwrap() > 0);
    assert_eq!(
        witnessed.charged_bytes(),
        witnessed.owned_heap_bytes().unwrap()
    );
    let retained = observer.snapshot();
    let scope = Dimension::OperationScope(Scope::Recovery);
    assert_eq!(
        retained.for_dimension(scope).active_units() - before.for_dimension(scope).active_units(),
        witnessed.charged_bytes(),
        "read buffers and verification scratch must already be disposed"
    );
    drop(witnessed);
    let after = observer.snapshot();
    assert_eq!(
        after.for_dimension(scope).active_units(),
        before.for_dimension(scope).active_units()
    );
    assert_eq!(
        after.for_dimension(scope).admitted_units() - before.for_dimension(scope).admitted_units(),
        after.for_dimension(scope).released_units() - before.for_dimension(scope).released_units()
    );
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
}

#[test]
#[cfg(feature = "certification-test-authority")]
fn actual_c4_changed_effect_frame_denies_and_disposes_all_native_grants() {
    let (directory, media, coordination) = fixture::coordination();
    let (effect, format) = effect_fixture();
    let paths = fixture::install_planned_nodes(directory.path(), &effect);
    let first = paths.first().expect("genuine first upsert has node writes");
    let mut changed = std::fs::read(first).unwrap();
    let offset = changed.len() / 2;
    changed[offset] ^= 0x40;
    std::fs::write(first, &changed).unwrap();
    let observer = coordination.certification_residency_allocations();
    let before = observer.snapshot();
    let mut window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let mut discovery = media.bounded_discovery(64, 2 << 20).unwrap();

    let denied = observe_effect_nodes(&mut discovery, &mut window, &effect, 11, format, 2 << 20);
    assert!(matches!(denied, Err(Denial::ControlFrame)));
    assert_eq!(discovery.counters().addressed_artifacts_read, 1);
    assert_eq!(std::fs::read(first).unwrap(), changed);
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
    let after = observer.snapshot();
    let scope = Dimension::OperationScope(Scope::Recovery);
    assert_eq!(
        after.for_dimension(scope).active_units(),
        before.for_dimension(scope).active_units()
    );
    assert_eq!(
        after.for_dimension(scope).admitted_units() - before.for_dimension(scope).admitted_units(),
        after.for_dimension(scope).released_units() - before.for_dimension(scope).released_units()
    );
}
