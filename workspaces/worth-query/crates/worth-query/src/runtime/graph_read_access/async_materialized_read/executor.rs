use super::{
    WorthQueryGraphReadMaterializationAdmittedLimits, WorthQueryGraphReadMaterializationCheckpoint,
    WorthQueryGraphReadMaterializationCounters, WorthQueryGraphReadMaterializationJob,
    WorthQueryGraphReadMaterializationProgress, WorthQueryGraphReadMaterializationRequest,
};
use crate::runtime::{WorthQueryRuntime, WorthQueryRuntimeError};

pub struct WorthQueryGraphReadMaterializationRuntime<'a> {
    runtime: &'a mut WorthQueryRuntime,
}

impl<'a> WorthQueryGraphReadMaterializationRuntime<'a> {
    pub(crate) fn new(runtime: &'a mut WorthQueryRuntime) -> Self {
        Self { runtime }
    }

    pub fn admit(
        self,
        request: WorthQueryGraphReadMaterializationRequest,
    ) -> Result<WorthQueryGraphReadMaterializationAdmittedJob, WorthQueryRuntimeError> {
        self.runtime
            .admit_facade_family(crate::runtime::WorthQueryRuntimeFacadeFamily::Read)?;
        Ok(WorthQueryGraphReadMaterializationAdmittedJob {
            snapshot_identity: self
                .runtime
                .current_snapshot_identity()?
                .evidence_identity()
                .as_str()
                .to_string(),
            request,
        })
    }
}

pub struct WorthQueryGraphReadMaterializationAdmittedJob {
    request: WorthQueryGraphReadMaterializationRequest,
    snapshot_identity: String,
}

impl WorthQueryGraphReadMaterializationAdmittedJob {
    pub fn request(&self) -> &WorthQueryGraphReadMaterializationRequest {
        &self.request
    }

    pub fn snapshot_identity(&self) -> &str {
        &self.snapshot_identity
    }

    pub fn start(self) -> Result<WorthQueryGraphReadMaterializationJob, WorthQueryRuntimeError> {
        let target_progress = materialization_progress_for_request(&self.request);
        let initial_progress = initial_progress_for_target(&self.request, &target_progress);
        let initial_checkpoint = WorthQueryGraphReadMaterializationCheckpoint::from_progress(
            self.request.digest(),
            0,
            0,
            0,
            0,
        );
        Ok(WorthQueryGraphReadMaterializationJob::running(
            self.request,
            self.snapshot_identity,
            initial_progress,
            target_progress,
            initial_checkpoint,
        ))
    }
}

fn initial_progress_for_target(
    request: &WorthQueryGraphReadMaterializationRequest,
    target_progress: &WorthQueryGraphReadMaterializationProgress,
) -> WorthQueryGraphReadMaterializationProgress {
    WorthQueryGraphReadMaterializationProgress::from_request_parts(
        request.digest(),
        target_progress.admitted_limits().clone(),
        WorthQueryGraphReadMaterializationCounters::new(0, 0, 0, 0, 0, 0),
    )
}

fn materialization_progress_for_request(
    request: &WorthQueryGraphReadMaterializationRequest,
) -> WorthQueryGraphReadMaterializationProgress {
    let admitted_limits =
        WorthQueryGraphReadMaterializationAdmittedLimits::from_policy(request.policy());
    let touched_edges = request.estimated_touched_edges();
    let frontier_pages = touched_edges.div_ceil(64).max(1);
    let allocated_bytes = request.estimated_resident_bytes();
    let emitted_rows = request.estimated_emitted_rows();
    let checkpoint_count = frontier_pages
        .div_ceil(request.policy().checkpoint_interval().frontier_page_count())
        .max(1);
    let counters = WorthQueryGraphReadMaterializationCounters::new(
        touched_edges,
        frontier_pages,
        allocated_bytes,
        emitted_rows,
        checkpoint_count,
        1,
    );
    WorthQueryGraphReadMaterializationProgress::from_request_parts(
        request.digest(),
        admitted_limits,
        counters,
    )
}
