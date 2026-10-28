use worth_store_physical_backend::{
    MediaQualificationDeferred, MediaQualificationDenial, MediaQualificationFailure,
    MediaQualificationRebindRequired, MediaQualificationStale,
};

use crate::physical_runtime::{AbortedRuntime, AdmittedPhysicalRuntime, RuntimeIdentity};

use super::MediaOwnedPhysicalRuntime;

pub type MediaAdmissionOutcome = worth_proof::ProofOutcome<
    MediaOwnedPhysicalRuntime,
    MediaAdmissionDenial,
    MediaAdmissionDeferred,
    MediaAdmissionStale,
    MediaAdmissionRebindRequired,
    MediaAdmissionInspectionRequired,
>;

pub struct MediaAdmissionDenial {
    runtime: AdmittedPhysicalRuntime,
    reason: MediaQualificationDenial,
}

impl MediaAdmissionDenial {
    pub(super) const fn new(
        runtime: AdmittedPhysicalRuntime,
        reason: MediaQualificationDenial,
    ) -> Self {
        Self { runtime, reason }
    }

    pub const fn reason(&self) -> &MediaQualificationDenial {
        &self.reason
    }

    pub fn into_runtime(self) -> AdmittedPhysicalRuntime {
        self.runtime
    }
}

pub struct MediaAdmissionDeferred {
    runtime: AdmittedPhysicalRuntime,
    reason: MediaQualificationDeferred,
}

impl MediaAdmissionDeferred {
    pub(super) const fn new(
        runtime: AdmittedPhysicalRuntime,
        reason: MediaQualificationDeferred,
    ) -> Self {
        Self { runtime, reason }
    }

    pub const fn reason(&self) -> MediaQualificationDeferred {
        self.reason
    }

    pub fn into_runtime(self) -> AdmittedPhysicalRuntime {
        self.runtime
    }
}

pub struct MediaAdmissionStale {
    runtime: AdmittedPhysicalRuntime,
    reason: MediaQualificationStale,
}

impl MediaAdmissionStale {
    pub(super) const fn new(
        runtime: AdmittedPhysicalRuntime,
        reason: MediaQualificationStale,
    ) -> Self {
        Self { runtime, reason }
    }

    pub const fn reason(&self) -> MediaQualificationStale {
        self.reason
    }

    pub fn into_runtime(self) -> AdmittedPhysicalRuntime {
        self.runtime
    }
}

pub struct MediaAdmissionRebindRequired {
    runtime: AdmittedPhysicalRuntime,
    reason: MediaQualificationRebindRequired,
}

impl MediaAdmissionRebindRequired {
    pub(super) const fn new(
        runtime: AdmittedPhysicalRuntime,
        reason: MediaQualificationRebindRequired,
    ) -> Self {
        Self { runtime, reason }
    }

    pub const fn reason(&self) -> MediaQualificationRebindRequired {
        self.reason
    }

    pub fn into_runtime(self) -> AdmittedPhysicalRuntime {
        self.runtime
    }
}

#[derive(Debug)]
pub enum MediaAdmissionInspectionCause {
    BackendFailure(MediaQualificationFailure),
}

pub struct MediaAdmissionInspectionRequired {
    runtime_identity: RuntimeIdentity,
    terminal: AbortedRuntime,
    cause: MediaAdmissionInspectionCause,
}

impl MediaAdmissionInspectionRequired {
    /// A contender may have created the namespace before losing the lease.
    /// The original failure retains that partial setup posture.
    pub fn mutation_owner_contended(&self) -> bool {
        matches!(
            &self.cause,
            MediaAdmissionInspectionCause::BackendFailure(
                MediaQualificationFailure::OwnerAfterEffect {
                    denial:
                        worth_store_physical_backend::FilesystemMediaOwnerAdmissionDenial::Ownership(
                            worth_store_physical_backend::MutationOwnershipDenial::Contended
                        ),
                    ..
                }
            )
        )
    }

    pub(super) const fn backend_failure(
        runtime_identity: RuntimeIdentity,
        terminal: AbortedRuntime,
        failure: MediaQualificationFailure,
    ) -> Self {
        Self {
            runtime_identity,
            terminal,
            cause: MediaAdmissionInspectionCause::BackendFailure(failure),
        }
    }

    pub const fn runtime_identity(&self) -> RuntimeIdentity {
        self.runtime_identity
    }

    pub const fn cause(&self) -> &MediaAdmissionInspectionCause {
        &self.cause
    }

    pub const fn terminal(&self) -> &AbortedRuntime {
        &self.terminal
    }
}
