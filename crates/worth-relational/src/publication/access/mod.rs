mod artifact_snapshots;
mod bundle_surface;
mod diagnostics;
mod failure_diagnostics;
mod stream_reads;

use crate::capabilities::PublicationDiagnosticsSource;
use crate::diagnostics::data::RelationalDiagnosticArtifact;
use crate::diagnostics::data::RelationalDiagnosticsFacade;
use crate::publication::bundle::PublicationBundle;
use crate::publication::cdc::data::{
    SubscriberResumeRequest, SubscriberStreamBatch, SubscriberStreamFailure,
};
use crate::publication::data::{
    PublicationArtifactSnapshot, PublicationDiagnosticsSnapshot, PublicationObservationSnapshot,
};
use crate::publication::patch::data::{
    PatchStreamBatch, PatchStreamReadError, PatchStreamRequest, PublishedAuthoritativePatchEnvelope,
};
use crate::runtime::RelationalReplayRecord;
use crate::runtime::RelationalRuntime;

pub(crate) use failure_diagnostics::invariant_failure_diagnostic;

pub struct PublicationSurface<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

pub struct PublicationArtifactsAccess<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

pub struct PublicationDiagnosticsAccess<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

pub struct PublicationPatchStreamAccess<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

pub struct PublicationSubscriberStreamAccess<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

impl RelationalRuntime {
    pub(crate) fn publication_access(&self) -> PublicationSurface<'_> {
        PublicationSurface::new(self)
    }
}

impl<'runtime> PublicationSurface<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }

    pub fn artifacts(&self) -> PublicationArtifactsAccess<'runtime> {
        PublicationArtifactsAccess::new(self.runtime)
    }

    pub fn diagnostic_access(&self) -> PublicationDiagnosticsAccess<'runtime> {
        PublicationDiagnosticsAccess::new(self.runtime)
    }

    pub fn patch_stream(&self) -> PublicationPatchStreamAccess<'runtime> {
        PublicationPatchStreamAccess::new(self.runtime)
    }

    pub fn subscriber_stream(&self) -> PublicationSubscriberStreamAccess<'runtime> {
        PublicationSubscriberStreamAccess::new(self.runtime)
    }

    pub fn latest_bundle(
        &self,
    ) -> Option<std::sync::Arc<PublicationBundle<RelationalReplayRecord>>> {
        self.runtime.publication.latest_bundle()
    }

    pub fn latest_patch(&self) -> Option<PublishedAuthoritativePatchEnvelope> {
        self.latest_bundle().map(|bundle| bundle.patch.clone())
    }

    pub fn latest_replay(&self) -> Option<RelationalReplayRecord> {
        self.latest_bundle().map(|bundle| bundle.replay.clone())
    }

    pub fn observation_snapshot(&self) -> PublicationObservationSnapshot {
        self.artifacts().observation()
    }

    pub fn artifact_snapshot(&self) -> PublicationArtifactSnapshot {
        self.artifacts().snapshot()
    }

    pub fn diagnostics(&self) -> RelationalDiagnosticsFacade {
        self.diagnostic_access().facade()
    }

    pub fn diagnostic_artifacts(&self) -> Vec<RelationalDiagnosticArtifact> {
        self.runtime.publication_diagnostics()
    }

    pub fn diagnostic_artifact_count(&self) -> usize {
        self.diagnostic_access().artifact_count()
    }

    pub fn diagnostics_since(&self, start: usize) -> Vec<RelationalDiagnosticArtifact> {
        self.diagnostic_access().artifacts_since(start)
    }

    pub fn diagnostics_snapshot(&self) -> PublicationDiagnosticsSnapshot {
        self.diagnostic_access().snapshot()
    }

    /// Read one bounded canonical batch, demanding only its indexed envelopes.
    pub fn read_patch_stream(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: PatchStreamRequest,
    ) -> Result<PatchStreamBatch, PatchStreamReadError> {
        self.patch_stream().read(execution, request)
    }

    pub fn read_subscriber_stream(
        &self,
        request: SubscriberResumeRequest,
    ) -> Result<SubscriberStreamBatch, SubscriberStreamFailure> {
        self.subscriber_stream().read(request)
    }
}

impl<'runtime> PublicationArtifactsAccess<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}

impl<'runtime> PublicationDiagnosticsAccess<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}

impl<'runtime> PublicationPatchStreamAccess<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}

impl<'runtime> PublicationSubscriberStreamAccess<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}
