use crate::entry::authority_binding::AdmittedRecoveryWorldBindingDrift;
use worth_proof::Binding;
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalRuntimeAdmission, PhysicalStore,
};

use super::*;
use crate::entry::PhysicalRecoveryLimitDeclaration;

#[test]
fn semantic_comparator_rejects_each_axis_without_mutating_an_authority() {
    let parent = tempfile::tempdir().expect("test parent");
    let primary_root = parent.path().join("primary");
    let alternate_root = parent.path().join("alternate");
    initialize_store(&primary_root);
    initialize_store(&alternate_root);
    let primary = PhysicalRecoveryPlatformAuthority::acquire(
        primary_root,
        PhysicalRecoveryStaticConfiguration::current(),
        limits(8),
    )
    .expect("primary authority");
    let alternate = PhysicalRecoveryPlatformAuthority::acquire(
        alternate_root,
        PhysicalRecoveryStaticConfiguration::current(),
        limits(9),
    )
    .expect("alternate authority");
    let base = primary.binding.axes().clone();
    let other = alternate.binding.axes();
    assert_ne!(base.backend_profile, other.backend_profile);

    let mut candidates = Vec::new();
    let mut axes = base.clone();
    axes.root_ownership = other.root_ownership.clone();
    candidates.push((axes, PhysicalRecoveryEntryBindingDrift::RootOwnership));
    let mut axes = base.clone();
    axes.recovery_session = other.recovery_session;
    candidates.push((axes, PhysicalRecoveryEntryBindingDrift::RecoverySession));
    let mut axes = base.clone();
    axes.backend_profile = other.backend_profile.clone();
    candidates.push((axes, PhysicalRecoveryEntryBindingDrift::BackendProfile));
    let mut axes = base.clone();
    axes.qualified_media_generation = other.qualified_media_generation;
    candidates.push((
        axes,
        PhysicalRecoveryEntryBindingDrift::QualifiedMediaGeneration,
    ));
    let mut axes = base.clone();
    axes.static_configuration[0] ^= 1;
    candidates.push((axes, PhysicalRecoveryEntryBindingDrift::StaticConfiguration));
    let mut axes = base.clone();
    axes.recovery_limits = other.recovery_limits;
    candidates.push((axes, PhysicalRecoveryEntryBindingDrift::RecoveryLimits));

    for (axes, expected) in candidates {
        assert_eq!(
            primary.binding.ensure_matches(&Binding::new(axes)),
            Err(expected)
        );
    }
    assert_eq!(primary.binding.axes(), &base);

    let primary_admission = primary
        .into_admitted()
        .unwrap_or_else(|_| panic!("primary admitted world"));
    let alternate_admission = alternate
        .into_admitted()
        .unwrap_or_else(|_| panic!("alternate admitted world"));
    let admitted_base = primary_admission.authority._world_binding.axes().clone();
    let admitted_other = alternate_admission.authority._world_binding.axes();
    assert_ne!(admitted_base.stable_store, admitted_other.stable_store);

    let mut admitted_candidates = Vec::new();
    let mut axes = admitted_base.clone();
    axes.root_ownership = admitted_other.root_ownership.clone();
    admitted_candidates.push((axes, AdmittedRecoveryWorldBindingDrift::RootOwnership));
    let mut axes = admitted_base.clone();
    axes.stable_store = admitted_other.stable_store;
    admitted_candidates.push((axes, AdmittedRecoveryWorldBindingDrift::StableStore));
    let mut axes = admitted_base.clone();
    axes.recovery_session = admitted_other.recovery_session;
    admitted_candidates.push((axes, AdmittedRecoveryWorldBindingDrift::RecoverySession));
    let mut axes = admitted_base.clone();
    axes.backend_profile = admitted_other.backend_profile.clone();
    admitted_candidates.push((axes, AdmittedRecoveryWorldBindingDrift::BackendProfile));
    let mut axes = admitted_base.clone();
    axes.qualified_media_generation = admitted_other.qualified_media_generation;
    admitted_candidates.push((
        axes,
        AdmittedRecoveryWorldBindingDrift::QualifiedMediaGeneration,
    ));
    let mut axes = admitted_base.clone();
    axes.static_configuration[0] ^= 1;
    admitted_candidates.push((axes, AdmittedRecoveryWorldBindingDrift::StaticConfiguration));
    let mut axes = admitted_base.clone();
    axes.recovery_limits[0] ^= 1;
    admitted_candidates.push((axes, AdmittedRecoveryWorldBindingDrift::RecoveryLimits));

    for (axes, expected) in admitted_candidates {
        assert_eq!(
            primary_admission
                .authority
                ._world_binding
                .ensure_matches(&Binding::new(axes)),
            Err(expected)
        );
    }
    assert_eq!(
        primary_admission.authority._world_binding.axes(),
        &admitted_base
    );
    assert_eq!(
        alternate_admission.authority._world_binding.axes(),
        admitted_other
    );
    primary_admission.authority.refuse();
    alternate_admission.authority.refuse();
}

fn initialize_store(root: &std::path::Path) {
    let runtime = PhysicalStore::admit(
        PhysicalRuntimeAdmission::new(root.to_path_buf()).expect("declared root"),
    )
    .expect("ordinary runtime admission");
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let media = match runtime.try_admit_filesystem_media(admission).into_raw() {
        TransitionOutcome::Success(media) => media,
        _ => panic!("ordinary media initialization failed"),
    };
    let _ = media.close();
}

fn limits(scale: u64) -> PhysicalRecoveryLimits {
    PhysicalRecoveryLimits::admit(PhysicalRecoveryLimitDeclaration {
        selector_candidates: 2,
        checkpoint_candidates: scale,
        manifest_bytes: scale * 1024,
        manifest_entries: scale,
        wal_segments: scale,
        wal_frames: scale * 8,
        wal_bytes: scale * 4096,
        redo_targets: scale,
        redo_bytes: scale * 4096,
        distinct_pages_and_extents: scale,
        operation_bindings: scale,
        staging_bytes: scale * 4096,
        recovery_memory_bytes: scale * 1024 * 1024,
        dirty_frames: scale,
        concurrent_commands: scale,
        publication_effects: 2,
        cleanup_candidates: scale,
        cleanup_bytes: scale * 4096,
        observation_bytes: scale * 1024,
    })
    .expect("bounded limits")
}
