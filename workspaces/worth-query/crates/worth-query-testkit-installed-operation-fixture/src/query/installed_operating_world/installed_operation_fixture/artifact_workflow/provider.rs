pub use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

pub use worth_query::facade::domain;

pub use super::native_observation::ArtifactNativeObservation;
pub use super::native_provider::{CandidateNativeRows, NativeProviderMode};

#[derive(Clone, Default)]
pub struct ArtifactProbe {
    pub inner: Arc<ArtifactProbeInner>,
}

#[derive(Default)]
pub struct ArtifactProbeInner {
    pub allocations: AtomicUsize,
    pub projection_calls: AtomicUsize,
    pub borrow_observations: AtomicUsize,
    pub disposals: AtomicUsize,
    pub replacements: AtomicUsize,
    pub cancellations: AtomicUsize,
    pub native_row_counts: AtomicUsize,
    pub native_row_batches: AtomicUsize,
    pub native_field_slices: AtomicUsize,
    pub native_projections: AtomicUsize,
    pub native_scalars: AtomicUsize,
    pub native_observations: Mutex<Vec<ArtifactNativeObservation>>,
    pub denials: Mutex<Vec<domain::WorthQueryArtifactDenialKind>>,
    pub lifecycle_snapshots: Mutex<Vec<domain::WorthQueryArtifactOwnerSnapshot>>,
    pub consumer_mode: Mutex<Option<String>>,
    pub retained_admission: Mutex<Option<domain::WorthQueryArtifactProductionAdmission>>,
    pub escaped_handle: Mutex<Option<domain::WorthQueryTransferredArtifactHandle>>,
    pub escaped_lease: Mutex<Option<domain::WorthQueryRetainedArtifactLease>>,
}

impl ArtifactProbe {
    pub fn allocations(&self) -> usize {
        self.inner.allocations.load(Ordering::SeqCst)
    }

    pub fn projection_calls(&self) -> usize {
        self.inner.projection_calls.load(Ordering::SeqCst)
    }

    pub fn borrow_observations(&self) -> usize {
        self.inner.borrow_observations.load(Ordering::SeqCst)
    }

    pub fn disposals(&self) -> usize {
        self.inner.disposals.load(Ordering::SeqCst)
    }

    pub fn replacements(&self) -> usize {
        self.inner.replacements.load(Ordering::SeqCst)
    }

    pub fn cancellations(&self) -> usize {
        self.inner.cancellations.load(Ordering::SeqCst)
    }

    pub fn native_row_counts(&self) -> usize {
        self.inner.native_row_counts.load(Ordering::SeqCst)
    }

    pub fn native_row_batches(&self) -> usize {
        self.inner.native_row_batches.load(Ordering::SeqCst)
    }

    pub fn native_field_slices(&self) -> usize {
        self.inner.native_field_slices.load(Ordering::SeqCst)
    }

    pub fn native_projections(&self) -> usize {
        self.inner.native_projections.load(Ordering::SeqCst)
    }

    pub fn native_scalars(&self) -> usize {
        self.inner.native_scalars.load(Ordering::SeqCst)
    }

    pub fn take_native_observations(&self) -> Vec<ArtifactNativeObservation> {
        std::mem::take(
            &mut *self
                .inner
                .native_observations
                .lock()
                .expect("native artifact observation probe lock remains available"),
        )
    }

    pub fn denials(&self) -> Vec<domain::WorthQueryArtifactDenialKind> {
        self.inner
            .denials
            .lock()
            .expect("artifact denial probe lock remains available")
            .clone()
    }

    pub fn lifecycle_snapshots(&self) -> Vec<domain::WorthQueryArtifactOwnerSnapshot> {
        self.inner
            .lifecycle_snapshots
            .lock()
            .expect("artifact lifecycle snapshot probe lock remains available")
            .clone()
    }

    pub fn take_escaped_handle(&self) -> Option<domain::WorthQueryTransferredArtifactHandle> {
        self.inner
            .escaped_handle
            .lock()
            .expect("escaped artifact handle probe lock remains available")
            .take()
    }

    pub fn take_escaped_lease(&self) -> Option<domain::WorthQueryRetainedArtifactLease> {
        self.inner
            .escaped_lease
            .lock()
            .expect("escaped artifact lease probe lock remains available")
            .take()
    }

    pub fn candidate(&self, projection: &[u8]) -> CandidateArtifactResource {
        self.candidate_with_mode(projection, NativeProviderMode::Standard)
    }

    pub fn native_candidate(&self, projection: &[u8], scenario: &str) -> CandidateArtifactResource {
        self.candidate_with_mode(projection, NativeProviderMode::for_scenario(scenario))
    }

    pub fn candidate_with_mode(
        &self,
        projection: &[u8],
        mode: NativeProviderMode,
    ) -> CandidateArtifactResource {
        self.inner.allocations.fetch_add(1, Ordering::SeqCst);
        CandidateArtifactResource {
            probe: self.clone(),
            projection: projection.to_vec().into_boxed_slice(),
            native: CandidateNativeRows::new(self.clone(), mode),
        }
    }

    pub fn foreign(&self, projection: &[u8]) -> ForeignArtifactResource {
        self.inner.allocations.fetch_add(1, Ordering::SeqCst);
        ForeignArtifactResource {
            probe: self.clone(),
            projection: projection.to_vec(),
        }
    }

    pub fn panic_during_projection(&self) -> PanicDuringProjectionResource {
        self.inner.allocations.fetch_add(1, Ordering::SeqCst);
        PanicDuringProjectionResource {
            probe: self.clone(),
        }
    }

    pub fn observe_borrow(&self) {
        self.inner
            .borrow_observations
            .fetch_add(1, Ordering::SeqCst);
    }

    pub fn observe_replacement(&self) {
        self.inner.replacements.fetch_add(1, Ordering::SeqCst);
    }

    pub fn observe_cancellation(&self) {
        self.inner.cancellations.fetch_add(1, Ordering::SeqCst);
    }

    pub fn observe_native_row_count(&self) {
        self.inner.native_row_counts.fetch_add(1, Ordering::SeqCst);
    }

    pub fn observe_native_row_batch(&self) {
        self.inner.native_row_batches.fetch_add(1, Ordering::SeqCst);
    }

    pub fn observe_native_field_slice(&self) {
        self.inner
            .native_field_slices
            .fetch_add(1, Ordering::SeqCst);
    }

    pub fn observe_native_projection(&self) {
        self.inner.native_projections.fetch_add(1, Ordering::SeqCst);
    }

    pub fn observe_native_scalar(&self) {
        self.inner.native_scalars.fetch_add(1, Ordering::SeqCst);
    }

    pub fn observe_native_access(&self, observation: ArtifactNativeObservation) {
        self.inner
            .native_observations
            .lock()
            .expect("native artifact observation probe lock remains available")
            .push(observation);
    }

    pub fn observe_denial(&self, kind: domain::WorthQueryArtifactDenialKind) {
        self.inner
            .denials
            .lock()
            .expect("artifact denial probe lock remains available")
            .push(kind);
    }

    pub fn observe_lifecycle(&self, snapshot: domain::WorthQueryArtifactOwnerSnapshot) {
        self.inner
            .lifecycle_snapshots
            .lock()
            .expect("artifact lifecycle snapshot probe lock remains available")
            .push(snapshot);
    }

    pub fn arm_consumer(&self, mode: String) {
        *self
            .inner
            .consumer_mode
            .lock()
            .expect("artifact consumer mode probe lock remains available") = Some(mode);
    }

    pub fn take_consumer_mode(&self) -> Option<String> {
        self.inner
            .consumer_mode
            .lock()
            .expect("artifact consumer mode probe lock remains available")
            .take()
    }

    pub fn retain_admission(&self, admission: domain::WorthQueryArtifactProductionAdmission) {
        *self
            .inner
            .retained_admission
            .lock()
            .expect("retained artifact admission probe lock remains available") = Some(admission);
    }

    pub fn take_retained_admission(&self) -> Option<domain::WorthQueryArtifactProductionAdmission> {
        self.inner
            .retained_admission
            .lock()
            .expect("retained artifact admission probe lock remains available")
            .take()
    }

    pub fn escape_handle(&self, handle: domain::WorthQueryTransferredArtifactHandle) {
        *self
            .inner
            .escaped_handle
            .lock()
            .expect("escaped artifact handle probe lock remains available") = Some(handle);
    }

    pub fn escape_lease(&self, lease: domain::WorthQueryRetainedArtifactLease) {
        *self
            .inner
            .escaped_lease
            .lock()
            .expect("escaped artifact lease probe lock remains available") = Some(lease);
    }
}

pub struct CandidateArtifactResource {
    pub probe: ArtifactProbe,
    pub projection: Box<[u8]>,
    pub native: CandidateNativeRows,
}

impl domain::WorthQueryArtifactProviderResource for CandidateArtifactResource {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.artifact-workflow.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        self.probe
            .inner
            .projection_calls
            .fetch_add(1, Ordering::SeqCst);
        self.projection.to_vec()
    }

    fn retained_bytes(&self) -> usize {
        self.projection
            .len()
            .saturating_add(self.native.retained_bytes())
    }

    fn native_access_provider(
        &self,
    ) -> Option<&dyn domain::WorthQueryArtifactNativeAccessProvider> {
        Some(&self.native)
    }

    fn dispose(&mut self) {
        self.probe.inner.disposals.fetch_add(1, Ordering::SeqCst);
    }
}

pub struct ForeignArtifactResource {
    pub probe: ArtifactProbe,
    pub projection: Vec<u8>,
}

pub struct PanicDuringProjectionResource {
    pub probe: ArtifactProbe,
}

impl domain::WorthQueryArtifactProviderResource for ForeignArtifactResource {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.artifact-workflow.foreign-provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        self.probe
            .inner
            .projection_calls
            .fetch_add(1, Ordering::SeqCst);
        self.projection.clone()
    }

    fn retained_bytes(&self) -> usize {
        self.projection.len()
    }

    fn dispose(&mut self) {
        self.probe.inner.disposals.fetch_add(1, Ordering::SeqCst);
    }
}

impl domain::WorthQueryArtifactProviderResource for PanicDuringProjectionResource {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.artifact-workflow.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        self.probe
            .inner
            .projection_calls
            .fetch_add(1, Ordering::SeqCst);
        panic!("provider projection panic");
    }

    fn retained_bytes(&self) -> usize {
        1
    }

    fn dispose(&mut self) {
        self.probe.inner.disposals.fetch_add(1, Ordering::SeqCst);
    }
}
