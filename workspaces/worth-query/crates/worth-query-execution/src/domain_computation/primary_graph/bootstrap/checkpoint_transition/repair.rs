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
    outputs: super::preserved_outputs::PreservedOutputs,
    detail: String,
    capture_denial: Option<WorthQueryCheckpointCaptureDenial>,
    finalization_denial: Option<
        Box<crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial>,
    >,
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
        outputs: super::preserved_outputs::PreservedOutputs,
        denial: WorthQueryCheckpointCaptureDenial,
    ) -> Self {
        Self {
            graph,
            publication,
            settlement: Some(Settlement::Acknowledged(authority)),
            outputs,
            detail: format!("acknowledged target checkpoint capture stopped: {denial:?}"),
            capture_denial: Some(denial),
            finalization_denial: None,
        }
    }

    pub(super) fn new(
        graph: WorthQueryPrimaryGraphIntegrationHandle,
        publication: WorthQueryPrimaryGraphPublication,
        deferred: DeferredRecoveredCheckpointTransition,
        outputs: super::preserved_outputs::PreservedOutputs,
    ) -> Self {
        let detail = format!("{:?}", deferred.cause());
        Self {
            graph,
            publication,
            settlement: Some(Settlement::Deferred(deferred)),
            outputs,
            detail,
            capture_denial: None,
            finalization_denial: None,
        }
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub(super) fn finalization_stopped(
        graph: WorthQueryPrimaryGraphIntegrationHandle,
        publication: WorthQueryPrimaryGraphPublication,
        authority: RecoveredRelationalRuntimeAuthority,
        outputs: super::preserved_outputs::PreservedOutputs,
        cause: crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    ) -> Self {
        Self {
            graph,
            publication,
            settlement: Some(Settlement::Acknowledged(authority)),
            outputs,
            detail: format!("acknowledged target finalization stopped: {cause:?}"),
            capture_denial: None,
            finalization_denial: Some(Box::new(cause)),
        }
    }

    /// Original post-performance refusal. Capture retries never replace it.
    pub fn finalization_denial(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial>
    {
        self.finalization_denial.as_deref()
    }

    /// Exact capture refusal from this capsule's latest attempt, if capture
    /// rather than native settlement stopped that attempt.
    pub fn capture_denial(&self) -> Option<&WorthQueryCheckpointCaptureDenial> {
        self.capture_denial.as_ref()
    }

    pub fn repair_to_checkpoint(
        mut self,
        phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<WorthQueryApplicationCheckpoint, Self> {
        let request = phase.execution_request();
        let execution = &request;
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
                match self.graph.with_runtime_mut(execution, |runtime| {
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
                    .with_runtime(execution, |runtime| {
                        runtime.durability_authority().native_checkpoint(policy)
                    })
                    .map_err(WorthQueryCheckpointCaptureDenial::from)
            })
            .and_then(|native| {
                WorthQueryApplicationCheckpoint::encode(
                    native,
                    &self.publication,
                    self.outputs.rows(),
                    policy,
                )
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
        if self.finalization_denial.is_none() {
            self.detail = format!("checkpoint capture stopped: {denial:?}");
        }
        self.capture_denial = Some(denial);
    }
}

mod certification_controls;
