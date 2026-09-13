use crate::orchestration::RecoveryCoordination;
use crate::progression::AdmittedPhysicalRecovery;

use super::{
    record_binding_comparison, record_binding_denial, AdmittedPlatformAdmission,
    PhysicalRecoveryOpenRequest, PhysicalRecoveryRefusal, PhysicalRecoveryRefusalKind,
};

pub(crate) fn admit_request(
    request: PhysicalRecoveryOpenRequest,
    yieldpoint: Option<worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint>,
) -> Result<AdmittedPhysicalRecovery, PhysicalRecoveryRefusal> {
    let PhysicalRecoveryOpenRequest {
        presentation,
        authority,
    } = request;
    record_binding_comparison();
    if let Err(drift) = authority.compare_request(&presentation) {
        record_binding_denial();
        let recovery_effects = authority.recovery_effect_count();
        authority.refuse();
        return Err(PhysicalRecoveryRefusal::new(
            PhysicalRecoveryRefusalKind::EntryBindingDrift(drift),
            recovery_effects,
        ));
    }
    let admitted = authority.into_admitted().map_err(|refused| {
        PhysicalRecoveryRefusal::new(
            PhysicalRecoveryRefusalKind::PersistedStoreAdmission(refused.error),
            refused.recovery_effects,
        )
    })?;
    let AdmittedPlatformAdmission {
        authority: admitted,
        registered_session,
    } = admitted;
    let mut admitted = admitted;
    let coordination = match RecoveryCoordination::fresh(
        &mut admitted.media,
        registered_session,
        admitted.limits,
        yieldpoint,
    ) {
        Ok(coordination) => coordination,
        Err(_) => {
            let recovery_effects = admitted.media.recovery_effect_count();
            admitted.refuse();
            return Err(PhysicalRecoveryRefusal::new(
                PhysicalRecoveryRefusalKind::CoordinationUnavailable,
                recovery_effects,
            ));
        }
    };
    if !coordination.is_ready() {
        let recovery_effects = admitted.media.recovery_effect_count();
        admitted.refuse();
        return Err(PhysicalRecoveryRefusal::new(
            PhysicalRecoveryRefusalKind::CoordinationUnavailable,
            recovery_effects,
        ));
    }
    Ok(AdmittedPhysicalRecovery::from_admission(
        admitted,
        coordination,
    ))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use worth_proof::TransitionOutcome;
    use worth_store::physical_runtime::FilesystemAccessPosture;
    use worth_store::physical_runtime::{
        FilesystemMediaAdmission, PhysicalRuntimeAdmission, PhysicalStore,
    };

    use crate::entry::{
        PhysicalRecoveryEntryBindingDrift, PhysicalRecoveryLimitDeclaration,
        PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest, PhysicalRecoveryPlatformAuthority,
        PhysicalRecoveryRefusalKind, PhysicalRecoveryStaticConfiguration,
    };

    #[test]
    fn fresh_existing_store_enters_one_admitted_world_without_effects() {
        let parent = tempfile::tempdir().expect("test root parent");
        let root = parent.path().join("store");
        let alternate_root = parent.path().join("alternate-store");
        initialize_store(&root);
        initialize_store(&alternate_root);
        let limits = limits(8);
        let configuration = PhysicalRecoveryStaticConfiguration::current();
        let authority =
            PhysicalRecoveryPlatformAuthority::acquire(root.clone(), configuration.clone(), limits)
                .expect("fresh recovery authority");
        let session = authority.session_identity();
        let profile = authority.qualified_backend_profile().clone();
        let request =
            PhysicalRecoveryOpenRequest::declare(root, configuration, profile, limits, authority);
        let admitted = request.admit().expect("admitted recovery");

        assert_eq!(admitted.session_identity(), session);
        assert_ne!(admitted.store_identity().bytes(), [0; 16]);
        assert_eq!(admitted.limits(), limits);
        assert_eq!(admitted.counters().recovery_effects, Some(0));
        assert!(admitted.proves_live_coordination_for_test());
        let _ = admitted.cancel_before_discovery();
    }

    #[test]
    fn independently_presentable_entry_axis_drift_is_refused_before_effects() {
        let parent = tempfile::tempdir().expect("test root parent");
        let root = parent.path().join("store");
        let alternate_root = parent.path().join("alternate-store");
        initialize_store(&root);
        initialize_store(&alternate_root);
        let alternate_limits = limits(9);
        let alternate = PhysicalRecoveryPlatformAuthority::acquire(
            alternate_root.clone(),
            PhysicalRecoveryStaticConfiguration::current(),
            alternate_limits,
        )
        .expect("alternate bound authority");
        let drifts = [
            PhysicalRecoveryEntryBindingDrift::RootOwnership,
            PhysicalRecoveryEntryBindingDrift::BackendProfile,
            PhysicalRecoveryEntryBindingDrift::RecoveryLimits,
        ];

        for drift in drifts {
            let limits = limits(8);
            let configuration = PhysicalRecoveryStaticConfiguration::current();
            let authority = PhysicalRecoveryPlatformAuthority::acquire(
                root.clone(),
                configuration.clone(),
                limits,
            )
            .expect("fresh recovery authority");
            let request_root = if drift == PhysicalRecoveryEntryBindingDrift::RootOwnership {
                alternate_root.clone()
            } else {
                root.clone()
            };
            let profile = if drift == PhysicalRecoveryEntryBindingDrift::BackendProfile {
                alternate.qualified_backend_profile().clone()
            } else {
                authority.qualified_backend_profile().clone()
            };
            let request_limits = if drift == PhysicalRecoveryEntryBindingDrift::RecoveryLimits {
                alternate_limits
            } else {
                limits
            };
            let request = PhysicalRecoveryOpenRequest::declare(
                request_root,
                configuration,
                profile,
                request_limits,
                authority,
            );
            let refusal = match request.admit() {
                Err(refusal) => refusal,
                Ok(admitted) => {
                    let _ = admitted.cancel_before_discovery();
                    panic!("axis drift must refuse")
                }
            };
            assert_eq!(
                refusal.kind,
                PhysicalRecoveryRefusalKind::EntryBindingDrift(drift)
            );
            assert_eq!(refusal.recovery_effects(), 0);
        }
        assert_eq!(
            PhysicalRecoveryPlatformAuthority::process_counters().recovery_effects,
            None
        );
        alternate.refuse();
    }

    #[test]
    fn abandoned_authority_is_owner_visible_and_releases_the_root() {
        let parent = tempfile::tempdir().expect("test root parent");
        let root = parent.path().join("store");
        initialize_store(&root);
        let before = PhysicalRecoveryPlatformAuthority::process_counters();
        let authority = PhysicalRecoveryPlatformAuthority::acquire(
            root.clone(),
            PhysicalRecoveryStaticConfiguration::current(),
            limits(4),
        )
        .expect("fresh recovery authority");
        drop(authority);
        let after = PhysicalRecoveryPlatformAuthority::process_counters();
        assert_eq!(
            after.owner_detected_non_terminal_drops,
            before.owner_detected_non_terminal_drops + 1
        );
        let reacquired = PhysicalRecoveryPlatformAuthority::acquire(
            root,
            PhysicalRecoveryStaticConfiguration::current(),
            limits(4),
        )
        .expect("dropped authority released root lease");
        drop(reacquired);
    }

    #[test]
    fn a_second_process_cannot_enter_the_owned_recovery_root() {
        let parent = tempfile::tempdir().expect("test root parent");
        let root = parent.path().join("store");
        initialize_store(&root);
        let ready = parent.path().join("ready");
        let release = parent.path().join("release");
        let mut child =
            std::process::Command::new(std::env::current_exe().expect("recovery test executable"))
                .args([
                    "--exact",
                    "entry::admission::tests::recovery_qualification_process_role",
                    "--nocapture",
                    "--test-threads=1",
                ])
                .env("WORTH_C8_RECOVERY_ROOT", &root)
                .env("WORTH_C8_RECOVERY_READY", &ready)
                .env("WORTH_C8_RECOVERY_RELEASE", &release)
                .spawn()
                .expect("spawn recovery owner");
        wait_for(&ready);

        let denial = PhysicalRecoveryPlatformAuthority::acquire(
            root,
            PhysicalRecoveryStaticConfiguration::current(),
            limits(4),
        )
        .err()
        .expect("second process must be refused");
        assert_eq!(
            denial,
            crate::PhysicalRecoveryPlatformAdmissionError::BackendQualification(
                worth_store::physical_runtime::RecoveryFilesystemQualificationError::OwnershipContended
            )
        );
        std::fs::write(&release, b"release").expect("release child");
        assert!(child.wait().expect("child status").success());
    }

    #[test]
    fn recovery_qualification_process_role() {
        let Some(root) = std::env::var_os("WORTH_C8_RECOVERY_ROOT") else {
            return;
        };
        let ready = std::path::PathBuf::from(
            std::env::var_os("WORTH_C8_RECOVERY_READY").expect("ready path"),
        );
        let release = std::path::PathBuf::from(
            std::env::var_os("WORTH_C8_RECOVERY_RELEASE").expect("release path"),
        );
        let authority = PhysicalRecoveryPlatformAuthority::acquire(
            root.into(),
            PhysicalRecoveryStaticConfiguration::current(),
            limits(4),
        )
        .expect("child recovery authority");
        std::fs::write(&ready, b"owned").expect("ready marker");
        wait_for(&release);
        authority.refuse();
    }

    fn wait_for(path: &Path) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while !path.exists() {
            assert!(
                std::time::Instant::now() < deadline,
                "timed out waiting for {path:?}"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    fn initialize_store(
        root: &Path,
    ) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        let runtime = PhysicalStore::admit(
            PhysicalRuntimeAdmission::new(root.to_path_buf()).expect("declared root"),
        )
        .expect("ordinary runtime admission");
        let admission = FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        );
        let media = match runtime.try_admit_filesystem_media(admission).into_raw() {
            TransitionOutcome::Success(media) => media,
            _ => panic!("ordinary media initialization failed"),
        };
        let identity = media.store_identity();
        let _ = media.close();
        identity
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
}
