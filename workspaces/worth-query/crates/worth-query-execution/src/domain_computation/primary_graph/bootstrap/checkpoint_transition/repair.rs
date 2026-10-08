//! Linear repair custody for an unpublished, performed installation.
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCheckpoint, WorthQueryCheckpointCaptureDenial,
    WorthQueryCheckpointCapturePolicy, WorthQueryPrimaryGraphIntegrationHandle,
    WorthQueryPrimaryGraphPublication,
};
use worth_relational::facade::durability::{
    DeferredRecoveredCheckpointTransition, RecoveredRelationalRuntimeAuthority,
};

enum Settlement {
    Deferred(DeferredRecoveredCheckpointTransition),
    Acknowledged(RecoveredRelationalRuntimeAuthority),
}

/// A performed transition awaiting native settlement or checkpoint capture.
/// This capsule never exposes a World, including in its acknowledged phase.
/// Consuming repair never reruns authoring or republishes. On acknowledgment it
/// captures a target checkpoint for ordinary restore; a refusal returns this
/// same capsule, including the exact native repair custody, for another attempt.
#[must_use = "repair the performed native transition or explicitly discard this unpublished installation"]
pub struct WorthQueryCheckpointTransitionRecovery {
    graph: WorthQueryPrimaryGraphIntegrationHandle,
    publication: WorthQueryPrimaryGraphPublication,
    settlement: Option<Settlement>,
    detail: String,
    capture_denial: Option<WorthQueryCheckpointCaptureDenial>,
}

impl std::fmt::Debug for WorthQueryCheckpointTransitionRecovery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryCheckpointTransitionRecovery")
            .field("detail", &self.detail)
            .field("capture_denial", &self.capture_denial)
            .finish_non_exhaustive()
    }
}

impl WorthQueryCheckpointTransitionRecovery {
    pub(super) fn acknowledged(
        graph: WorthQueryPrimaryGraphIntegrationHandle,
        publication: WorthQueryPrimaryGraphPublication,
        authority: RecoveredRelationalRuntimeAuthority,
        denial: WorthQueryCheckpointCaptureDenial,
    ) -> Self {
        Self {
            graph,
            publication,
            settlement: Some(Settlement::Acknowledged(authority)),
            detail: format!("acknowledged target checkpoint capture stopped: {denial:?}"),
            capture_denial: Some(denial),
        }
    }

    #[cfg(feature = "test-durability-faults")]
    #[doc(hidden)]
    pub fn fail_next_durable_append_for_test(&self) {
        self.graph
            .with_runtime(|runtime| runtime.fail_next_durable_append_for_test());
    }

    pub(super) fn new(
        graph: WorthQueryPrimaryGraphIntegrationHandle,
        publication: WorthQueryPrimaryGraphPublication,
        deferred: DeferredRecoveredCheckpointTransition,
    ) -> Self {
        let detail = format!("{:?}", deferred.cause());
        Self {
            graph,
            publication,
            settlement: Some(Settlement::Deferred(deferred)),
            detail,
            capture_denial: None,
        }
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// Exact capture refusal from this capsule's latest attempt, if capture
    /// rather than native settlement stopped that attempt.
    pub fn capture_denial(&self) -> Option<&WorthQueryCheckpointCaptureDenial> {
        self.capture_denial.as_ref()
    }

    pub fn repair_to_checkpoint(
        mut self,
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<WorthQueryApplicationCheckpoint, Self> {
        self.capture_denial = None;
        if let Err(error) = policy.check_live() {
            self.retain_capture_denial(error.into());
            return Err(self);
        }
        match self
            .settlement
            .take()
            .expect("repair capsule always carries its phase")
        {
            Settlement::Deferred(deferred) => {
                match self.graph.with_runtime_mut(|runtime| {
                    runtime
                        .durability_recovery()
                        .repair_checkpoint_transition(deferred)
                }) {
                    Ok(acknowledged) => {
                        self.settlement =
                            Some(Settlement::Acknowledged(acknowledged.into_parts().1))
                    }
                    Err(error) => {
                        self.detail = format!("{:?}", error.cause());
                        self.settlement = Some(Settlement::Deferred(error.into_transition()));
                        return Err(self);
                    }
                }
            }
            Settlement::Acknowledged(authority) => {
                self.settlement = Some(Settlement::Acknowledged(authority))
            }
        }
        match policy
            .check_live()
            .map_err(WorthQueryCheckpointCaptureDenial::from)
            .and_then(|()| {
                self.graph
                    .with_runtime(|runtime| runtime.durability_authority().native_checkpoint())
                    .map_err(WorthQueryCheckpointCaptureDenial::from)
            })
            .and_then(|native| {
                WorthQueryApplicationCheckpoint::encode(native, &self.publication, &[], policy)
                    .map(|(checkpoint, _)| checkpoint)
            }) {
            Ok(checkpoint) => Ok(checkpoint),
            Err(error) => {
                self.retain_capture_denial(error);
                Err(self)
            }
        }
    }

    fn retain_capture_denial(&mut self, denial: WorthQueryCheckpointCaptureDenial) {
        self.detail = format!("checkpoint capture stopped: {denial:?}");
        self.capture_denial = Some(denial);
    }
}
