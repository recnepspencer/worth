//! Linear repair custody for an unpublished, performed installation.
use crate::domain_computation::primary_graph::{
    ApplicationHome, WorthQueryApplicationCheckpoint, WorthQueryPrimaryGraphIntegrationHandle,
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
/// captures the successor image into a home an ordinary open resumes; a refusal returns this
/// same capsule, including the exact native repair custody, for another attempt.
#[must_use = "repair the performed native transition or explicitly discard this unpublished installation"]
pub struct WorthQueryOpenAdoptionRecovery {
    graph: WorthQueryPrimaryGraphIntegrationHandle,
    publication: WorthQueryPrimaryGraphPublication,
    settlement: Option<Settlement>,
    detail: String,
    handle_denial: Option<crate::facade::primary_graph::WorthQueryHandleDenial>,
}

impl std::fmt::Debug for WorthQueryOpenAdoptionRecovery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryOpenAdoptionRecovery")
            .field("detail", &self.detail)
            .finish_non_exhaustive()
    }
}

impl WorthQueryOpenAdoptionRecovery {
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
            handle_denial: None,
            detail,
        }
    }

    #[cfg(feature = "test-durability-faults")]
    #[doc(hidden)]
    pub fn fail_next_durable_append_for_test(
        &self,
    ) -> Result<(), crate::facade::primary_graph::WorthQueryHandleDenial> {
        self.graph
            .with_runtime(|runtime| runtime.fail_next_durable_append_for_test())
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
            handle_denial: None,
            detail,
        }
    }

    pub fn handle_denial(&self) -> Option<crate::facade::primary_graph::WorthQueryHandleDenial> {
        self.handle_denial
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn repair(mut self) -> Result<ApplicationHome, Self> {
        match self
            .settlement
            .take()
            .expect("repair capsule always carries its phase")
        {
            Settlement::Deferred(deferred) => {
                let mut deferred = Some(deferred);
                let repaired = self.graph.with_runtime_mut(|runtime| {
                    runtime.durability_recovery().repair_checkpoint_transition(
                        deferred
                            .take()
                            .expect("repair owns its deferred transition"),
                    )
                });
                let repaired = match repaired {
                    Ok(repaired) => repaired,
                    Err(denial) => {
                        self.handle_denial = Some(denial);
                        self.detail = denial.to_string();
                        self.settlement = deferred.map(Settlement::Deferred);
                        return Err(self);
                    }
                };
                match repaired {
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
            Ok(Ok(native)) => Ok(ApplicationHome::holding(
                WorthQueryApplicationCheckpoint::encode(native, &self.publication, &[]).0,
            )),
            Err(denial) => {
                self.handle_denial = Some(denial);
                self.detail = denial.to_string();
                Err(self)
            }
            Ok(Err(error)) => {
                self.handle_denial = None;
                self.detail = format!("{error:?}");
                Err(self)
            }
        }
    }
}
