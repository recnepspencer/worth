use std::path::PathBuf;

use worth_proof::AuthorityWitness;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordResidencyPolicy, AdmittedRecoveryFilesystemMedia,
    ConfiguredPhysicalDurabilityDeclaration, PhysicalRecoveryFreshnessPort,
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
    residency_policy: AdmittedPhysicalRecordResidencyPolicy,
    durability: Option<ConfiguredPhysicalDurabilityDeclaration>,
}

#[cfg(test)]
#[path = "authority/binding_contract_tests.rs"]
mod binding_contract_tests;

impl PhysicalRecoveryPlatformAuthority {
    pub fn acquire(
        root: PathBuf,
        configuration: PhysicalRecoveryStaticConfiguration,
        limits: PhysicalRecoveryLimits,
    ) -> Result<Self, PhysicalRecoveryPlatformAdmissionError> {
        let media = QualifiedRecoveryFilesystemMedia::qualify_existing(&root)
            .map_err(PhysicalRecoveryPlatformAdmissionError::BackendQualification)?;
        Self::from_held_media(root, configuration, limits, media)
    }

    /// Admit recovery from the media owner the home already holds. No lease
    /// is reacquired, and the persisted identity still enters through recovery.
    pub fn from_held_media(
        root: PathBuf,
        configuration: PhysicalRecoveryStaticConfiguration,
        limits: PhysicalRecoveryLimits,
        media: QualifiedRecoveryFilesystemMedia,
    ) -> Result<Self, PhysicalRecoveryPlatformAdmissionError> {
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
        let residency_policy = configuration.residency_policy();
        let durability = configuration.durability();
        Ok(Self {
            _witness: PhysicalRecoveryPlatformMarker::witness(),
            media,
            registered_session,
            session,
            binding,
            limits,
            record_format,
            residency_policy,
            durability,
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
            residency_policy,
            durability,
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
                        durability,
                    },
                    registered_session,
                    residency_policy,
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
    pub(crate) durability: Option<ConfiguredPhysicalDurabilityDeclaration>,
}

pub(crate) struct AdmittedPlatformAdmission {
    pub(crate) authority: AdmittedPlatformAuthority,
    pub(crate) registered_session: PhysicalRecoveryRegisteredSessionAuthority,
    pub(crate) residency_policy: AdmittedPhysicalRecordResidencyPolicy,
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
