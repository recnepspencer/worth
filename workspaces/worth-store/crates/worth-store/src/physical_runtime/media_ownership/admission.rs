use worth_proof::TransitionOutcome;
#[cfg(feature = "certification-test-authority")]
use worth_store_physical_backend::MediaFaultSchedule;
use worth_store_physical_backend::{
    FilesystemAccessPosture, FilesystemQualificationMode, FilesystemQualificationRequest,
    RootProfileQualificationReport,
};

use super::{
    admission_outcome::{
        MediaAdmissionDeferred, MediaAdmissionDenial, MediaAdmissionInspectionRequired,
        MediaAdmissionRebindRequired, MediaAdmissionStale,
    },
    MediaAdmissionOutcome, MediaOwnedPhysicalRuntime,
};
use crate::physical_runtime::runtime::AdmittedPhysicalRuntime;

#[derive(Debug, Clone)]
pub struct FilesystemMediaAdmission {
    mode: FilesystemQualificationMode,
    access: FilesystemAccessPosture,
    #[cfg(feature = "certification-test-authority")]
    fault_schedule: MediaFaultSchedule,
    expected_profile: Option<RootProfileQualificationReport>,
}

impl FilesystemMediaAdmission {
    pub fn production(access: FilesystemAccessPosture) -> Self {
        Self {
            mode: FilesystemQualificationMode::Production,
            access,
            #[cfg(feature = "certification-test-authority")]
            fault_schedule: MediaFaultSchedule::default(),
            expected_profile: None,
        }
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification(access: FilesystemAccessPosture) -> Self {
        Self {
            mode: FilesystemQualificationMode::Certification,
            access,
            fault_schedule: MediaFaultSchedule::default(),
            expected_profile: None,
        }
    }

    pub fn require_current_profile(mut self, report: RootProfileQualificationReport) -> Self {
        self.expected_profile = Some(report);
        self
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn with_fault_schedule(mut self, schedule: MediaFaultSchedule) -> Self {
        self.fault_schedule = schedule;
        self
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn fault_schedule_authority(
        &self,
    ) -> crate::physical_runtime::certification::CertificationMediaFaultAuthority {
        worth_store_physical_backend::certification_media_fault_authority()
    }
}

pub(in crate::physical_runtime) fn try_admit(
    runtime: AdmittedPhysicalRuntime,
    admission: FilesystemMediaAdmission,
) -> MediaAdmissionOutcome {
    let core = runtime.into_core();
    let request = admission.into_request(&core);
    let qualified = worth_store_physical_backend::qualify_filesystem_media(request);
    finish_admission(core, qualified)
}

#[cfg(feature = "recovery-runtime-owner")]
pub(in crate::physical_runtime) fn admit_recovered(
    runtime: AdmittedPhysicalRuntime,
    admission: FilesystemMediaAdmission,
    media: worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
) -> MediaAdmissionOutcome {
    let core = runtime.into_core();
    let request = admission.into_request(&core);
    let qualified = media.qualify_serving(request);
    finish_admission(core, qualified)
}

#[cfg(feature = "recovery-runtime-owner")]
pub(in crate::physical_runtime) fn admit_home(
    runtime: AdmittedPhysicalRuntime,
    admission: FilesystemMediaAdmission,
    home: worth_store_physical_backend::QualifiedRecoveryFilesystemMedia,
) -> MediaAdmissionOutcome {
    let core = runtime.into_core();
    let request = admission.into_request(&core);
    finish_admission(core, home.qualify_serving(request))
}

impl FilesystemMediaAdmission {
    fn into_request(
        self,
        core: &crate::physical_runtime::runtime::PhysicalRuntimeCore,
    ) -> FilesystemQualificationRequest {
        self.into_request_at(
            core.declared_store_root().as_path().to_owned(),
            Some(core.runtime_identity().get()),
        )
    }
    #[cfg(feature = "recovery-runtime-owner")]
    pub fn qualify_home(
        &self,
        root: impl Into<std::path::PathBuf>,
    ) -> Result<
        worth_store_physical_backend::QualifiedRecoveryFilesystemMedia,
        worth_store_physical_backend::RecoveryFilesystemQualificationError,
    > {
        worth_store_physical_backend::QualifiedRecoveryFilesystemMedia::qualify_home(
            self.clone().into_request_at(root.into(), None),
        )
    }
    fn into_request_at(
        self,
        root: std::path::PathBuf,
        runtime: Option<u64>,
    ) -> FilesystemQualificationRequest {
        let request = match self.mode {
            FilesystemQualificationMode::Production => {
                FilesystemQualificationRequest::production(root, self.access)
            }
            #[cfg(feature = "certification-test-authority")]
            FilesystemQualificationMode::Certification => {
                FilesystemQualificationRequest::certification(root, self.access)
            }
            #[cfg(not(feature = "certification-test-authority"))]
            FilesystemQualificationMode::Certification => {
                unreachable!("no certification constructor")
            }
        };
        let request = match runtime {
            Some(runtime) => request.for_runtime_incarnation(runtime),
            None => request,
        };
        #[cfg(feature = "certification-test-authority")]
        let request = request.with_fault_schedule(self.fault_schedule);
        match self.expected_profile {
            Some(report) => request.require_current_profile(report),
            None => request,
        }
    }
}

fn finish_admission(
    core: crate::physical_runtime::runtime::PhysicalRuntimeCore,
    qualified: worth_store_physical_backend::AdmittedFilesystemMedia,
) -> MediaAdmissionOutcome {
    let runtime_identity = core.runtime_identity();
    match qualified.into_raw() {
        TransitionOutcome::Success(media) => {
            core.progress_to_media_owned();
            TransitionOutcome::success(MediaOwnedPhysicalRuntime::new(core, media)).into()
        }
        TransitionOutcome::Denied(denial) => TransitionOutcome::denied(MediaAdmissionDenial::new(
            AdmittedPhysicalRuntime::from_core(core),
            denial,
        ))
        .into(),
        TransitionOutcome::Deferred(deferred) => TransitionOutcome::deferred(
            MediaAdmissionDeferred::new(AdmittedPhysicalRuntime::from_core(core), deferred),
        )
        .into(),
        TransitionOutcome::Stale(stale) => TransitionOutcome::stale(MediaAdmissionStale::new(
            AdmittedPhysicalRuntime::from_core(core),
            stale,
        ))
        .into(),
        TransitionOutcome::RebindRequired(rebind) => TransitionOutcome::rebind_required(
            MediaAdmissionRebindRequired::new(AdmittedPhysicalRuntime::from_core(core), rebind),
        )
        .into(),
        TransitionOutcome::Failed(failure) => {
            let terminal = core.abort();
            TransitionOutcome::failed(MediaAdmissionInspectionRequired::backend_failure(
                runtime_identity,
                terminal,
                failure,
            ))
            .into()
        }
    }
}
