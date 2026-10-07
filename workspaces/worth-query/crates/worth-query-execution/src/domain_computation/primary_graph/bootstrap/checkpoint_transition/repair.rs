//! Linear repair custody for an unpublished, performed installation.
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCheckpoint, WorthQueryPrimaryGraphIntegrationHandle,
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
}

impl std::fmt::Debug for WorthQueryCheckpointTransitionRecovery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryCheckpointTransitionRecovery")
            .field("detail", &self.detail)
            .finish_non_exhaustive()
    }
}

impl WorthQueryCheckpointTransitionRecovery {
    pub(super) fn acknowledged(
        graph: WorthQueryPrimaryGraphIntegrationHandle,
        publication: WorthQueryPrimaryGraphPublication,
        authority: RecoveredRelationalRuntimeAuthority,
        detail: String,
    ) -> Self {
        Self {
            graph,
            publication,
            settlement: Some(Settlement::Acknowledged(authority)),
            detail,
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
        }
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn repair_to_checkpoint(mut self) -> Result<WorthQueryApplicationCheckpoint, Self> {
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
        match self
            .graph
            .with_runtime(|runtime| runtime.durability_authority().native_checkpoint())
        {
            Ok(native) => {
                Ok(WorthQueryApplicationCheckpoint::encode(native, &self.publication, &[]).0)
            }
            Err(error) => {
                self.detail = format!("{error:?}");
                Err(self)
            }
        }
    }
}
