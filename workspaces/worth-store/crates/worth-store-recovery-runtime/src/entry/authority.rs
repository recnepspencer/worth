use std::path::PathBuf;

use worth_proof::AuthorityWitness;
use worth_store::physical_runtime::{
    AdmittedRecoveryFilesystemMedia, PhysicalRecoveryFreshnessPort,
    PhysicalRecoveryRegisteredSessionAuthority, QualifiedPhysicalBackendProfile,
    QualifiedRecoveryFilesystemMedia, RecoveryFilesystemQualificationError,
};
use worth_store_physical_format::PhysicalRecordFormatDeclaration;

use super::authority_binding::{
    admitted_world_binding, entry_binding, entry_presentation, AdmittedRecoveryWorldBinding,
    PhysicalRecoveryEntryBinding, PhysicalRecoveryEntryPresentation,
};
use super::{
    counters, PhysicalRecoveryAdmissionCounters, PhysicalRecoveryEntryBindingDrift,
    PhysicalRecoveryLimits, PhysicalRecoverySessionIdentity, PhysicalRecoveryStaticConfiguration,
    RecoverySession,
};

worth_proof::authority_marker!(pub PhysicalRecoveryPlatformMarker);

pub struct PhysicalRecoveryPlatformAuthority {
    _witness: AuthorityWitness<PhysicalRecoveryPlatformMarker>,
    media: QualifiedRecoveryFilesystemMedia,
    registered_session: PhysicalRecoveryRegisteredSessionAuthority,
    session: RecoverySession,
    binding: PhysicalRecoveryEntryBinding,
    limits: PhysicalRecoveryLimits,
    record_format: PhysicalRecordFormatDeclaration,
}

#[cfg(test)]
mod binding_contract_tests {
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
        let admission = FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        );
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
}

impl PhysicalRecoveryPlatformAuthority {
    pub fn acquire(
        root: PathBuf,
        configuration: PhysicalRecoveryStaticConfiguration,
        limits: PhysicalRecoveryLimits,
    ) -> Result<Self, PhysicalRecoveryPlatformAdmissionError> {
        let media = QualifiedRecoveryFilesystemMedia::qualify_existing(&root)
            .map_err(PhysicalRecoveryPlatformAdmissionError::BackendQualification)?;
        let Some(freshness) = PhysicalRecoveryFreshnessPort::admit(&media) else {
            return Err(PhysicalRecoveryPlatformAdmissionError::FreshnessUnavailable);
        };
        Self::from_qualified_media(root, configuration, limits, media, freshness)
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn acquire_for_certification(
        root: PathBuf,
        configuration: PhysicalRecoveryStaticConfiguration,
        limits: PhysicalRecoveryLimits,
        schedule: worth_store::physical_runtime::certification::MediaFaultSchedule,
    ) -> Result<Self, PhysicalRecoveryPlatformAdmissionError> {
        let media = QualifiedRecoveryFilesystemMedia::qualify_existing_for_certification(
            &root,
            schedule.clone(),
        )
        .map_err(PhysicalRecoveryPlatformAdmissionError::BackendQualification)?;
        let Some(freshness) = PhysicalRecoveryFreshnessPort::admit_for_certification(&media) else {
            return Err(PhysicalRecoveryPlatformAdmissionError::FreshnessUnavailable);
        };
        Self::from_qualified_media(root, configuration, limits, media, freshness)
    }

    fn from_qualified_media(
        root: PathBuf,
        configuration: PhysicalRecoveryStaticConfiguration,
        limits: PhysicalRecoveryLimits,
        media: QualifiedRecoveryFilesystemMedia,
        freshness: worth_store::physical_runtime::PhysicalRecoveryFreshnessAuthority,
    ) -> Result<Self, PhysicalRecoveryPlatformAdmissionError> {
        let Some(registered_session) = freshness.register_session() else {
            drop(media);
            return Err(PhysicalRecoveryPlatformAdmissionError::SessionIdentityUnavailable);
        };
        let session = match RecoverySession::issue(registered_session.session_identity_bytes()) {
            Ok(session) => session,
            Err(()) => {
                drop(media);
                return Err(PhysicalRecoveryPlatformAdmissionError::SessionIdentityUnavailable);
            }
        };
        let session_identity = session.identity();
        let binding = entry_binding(
            root.clone(),
            media.root_ownership_identity(),
            session_identity,
            media.backend_profile(),
            media.media_generation(),
            &configuration,
            limits,
        );
        let record_format = configuration.record_format();
        Ok(Self {
            _witness: PhysicalRecoveryPlatformMarker::witness(),
            media,
            registered_session,
            session,
            binding,
            limits,
            record_format,
        })
    }

    pub fn qualified_backend_profile(&self) -> &QualifiedPhysicalBackendProfile {
        self.media.backend_profile()
    }

    pub const fn session_identity(&self) -> PhysicalRecoverySessionIdentity {
        self.session.identity()
    }

    pub fn process_counters() -> PhysicalRecoveryAdmissionCounters {
        counters::snapshot(None)
    }

    pub(crate) fn recovery_effect_count(&self) -> u64 {
        self.media.recovery_effect_count()
    }

    pub(crate) fn present_request(
        &self,
        root: PathBuf,
        profile: &QualifiedPhysicalBackendProfile,
        configuration: &PhysicalRecoveryStaticConfiguration,
        limits: PhysicalRecoveryLimits,
    ) -> PhysicalRecoveryEntryPresentation {
        entry_presentation(
            root,
            self.media.root_ownership_identity(),
            self.session.identity(),
            profile,
            self.media.media_generation(),
            configuration,
            limits,
        )
    }

    pub(crate) fn compare_request(
        &self,
        presentation: &PhysicalRecoveryEntryPresentation,
    ) -> Result<(), PhysicalRecoveryEntryBindingDrift> {
        presentation.compare_with(&self.binding)
    }

    pub(crate) fn into_admitted(self) -> Result<AdmittedPlatformAdmission, RefusedAuthority> {
        let Self {
            media,
            registered_session,
            session,
            limits,
            record_format,
            binding,
            ..
        } = self;
        let recovery_effects = media.recovery_effect_count();
        match media.admit_persisted_store() {
            Ok(media) => {
                let world_binding = admitted_world_binding(&binding, media.store_identity());
                Ok(AdmittedPlatformAdmission {
                    authority: AdmittedPlatformAuthority {
                        media,
                        session,
                        _world_binding: world_binding,
                        limits,
                        record_format,
                    },
                    registered_session,
                })
            }
            Err(error) => {
                session.refuse();
                Err(RefusedAuthority {
                    error,
                    recovery_effects,
                })
            }
        }
    }

    pub(crate) fn refuse(self) {
        let Self { media, session, .. } = self;
        drop(media);
        session.refuse();
    }
}

pub(crate) struct AdmittedPlatformAuthority {
    pub(crate) media: AdmittedRecoveryFilesystemMedia,
    pub(crate) session: RecoverySession,
    pub(crate) _world_binding: AdmittedRecoveryWorldBinding,
    pub(crate) limits: PhysicalRecoveryLimits,
    pub(crate) record_format: PhysicalRecordFormatDeclaration,
}

pub(crate) struct AdmittedPlatformAdmission {
    pub(crate) authority: AdmittedPlatformAuthority,
    pub(crate) registered_session: PhysicalRecoveryRegisteredSessionAuthority,
}

impl AdmittedPlatformAuthority {
    pub(crate) fn refuse(self) {
        let Self { media, session, .. } = self;
        drop(media);
        session.refuse();
    }
}

pub(crate) struct RefusedAuthority {
    pub(crate) error: RecoveryFilesystemQualificationError,
    pub(crate) recovery_effects: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryPlatformAdmissionError {
    BackendQualification(RecoveryFilesystemQualificationError),
    SessionIdentityUnavailable,
    FreshnessUnavailable,
}
