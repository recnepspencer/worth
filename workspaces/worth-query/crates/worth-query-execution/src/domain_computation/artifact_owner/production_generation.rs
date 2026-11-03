use std::sync::Arc;

use super::{WorthQueryArtifactDenial, WorthQueryWorkflowArtifactRegistry};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryArtifactProductionGeneration(u64);

impl WorthQueryArtifactProductionGeneration {
    pub(super) const fn initial() -> Self {
        Self(1)
    }

    pub(super) fn next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }

    pub const fn ordinal(self) -> u64 {
        self.0
    }
}

pub(crate) struct WorthQueryArtifactProductionGenerationPending {
    pub(super) registry: Arc<WorthQueryWorkflowArtifactRegistry>,
    pub(super) prior: WorthQueryArtifactProductionGeneration,
    pub(super) next: WorthQueryArtifactProductionGeneration,
    active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryArtifactProductionGenerationAbortFailure {
    prior: WorthQueryArtifactProductionGeneration,
    pending: WorthQueryArtifactProductionGeneration,
    denial: WorthQueryArtifactDenial,
}

impl WorthQueryArtifactProductionGenerationPending {
    pub(super) fn new(
        registry: Arc<WorthQueryWorkflowArtifactRegistry>,
        prior: WorthQueryArtifactProductionGeneration,
        next: WorthQueryArtifactProductionGeneration,
    ) -> Self {
        Self {
            registry,
            prior,
            next,
            active: true,
        }
    }

    pub(crate) const fn generation(&self) -> WorthQueryArtifactProductionGeneration {
        self.next
    }

    pub(crate) fn belongs_to(&self, registry: &Arc<WorthQueryWorkflowArtifactRegistry>) -> bool {
        Arc::ptr_eq(&self.registry, registry)
    }

    pub(crate) fn abort(
        mut self,
    ) -> Result<(), WorthQueryArtifactProductionGenerationAbortFailure> {
        let outcome = self
            .registry
            .abort_generation(self.prior, self.next)
            .map_err(
                |denial| WorthQueryArtifactProductionGenerationAbortFailure {
                    prior: self.prior,
                    pending: self.next,
                    denial,
                },
            );
        self.active = false;
        outcome
    }

    pub(crate) fn commit(mut self) -> WorthQueryArtifactProductionGenerationCommitted {
        self.registry.commit_generation(self.prior, self.next);
        self.active = false;
        WorthQueryArtifactProductionGenerationCommitted {
            registry: Arc::clone(&self.registry),
            prior: self.prior,
        }
    }
}

impl WorthQueryArtifactProductionGenerationAbortFailure {
    pub(crate) const fn prior_generation(&self) -> WorthQueryArtifactProductionGeneration {
        self.prior
    }

    pub(crate) const fn pending_generation(&self) -> WorthQueryArtifactProductionGeneration {
        self.pending
    }

    pub(crate) const fn denial(&self) -> &WorthQueryArtifactDenial {
        &self.denial
    }

    pub(crate) const fn detail(&self) -> &'static str {
        self.denial.detail()
    }
}

impl Drop for WorthQueryArtifactProductionGenerationPending {
    fn drop(&mut self) {
        if self.active {
            let _ = self.registry.abort_generation(self.prior, self.next);
        }
    }
}

pub(crate) struct WorthQueryArtifactProductionGenerationCommitted {
    registry: Arc<WorthQueryWorkflowArtifactRegistry>,
    prior: WorthQueryArtifactProductionGeneration,
}

impl WorthQueryArtifactProductionGenerationCommitted {
    pub(crate) fn belongs_to(&self, registry: &Arc<WorthQueryWorkflowArtifactRegistry>) -> bool {
        Arc::ptr_eq(&self.registry, registry)
    }

    pub(crate) const fn prior(&self) -> WorthQueryArtifactProductionGeneration {
        self.prior
    }
}
