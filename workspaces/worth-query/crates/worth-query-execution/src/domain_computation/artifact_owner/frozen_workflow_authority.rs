use std::sync::Arc;

use super::registry::WorthQueryWorkflowArtifactRegistryPosture;
use super::{
    WorthQueryArtifactDenial, WorthQueryArtifactProductionAuthority,
    WorthQueryArtifactProductionGeneration, WorthQueryArtifactProductionGenerationCommitted,
    WorthQueryArtifactProductionGenerationPending, WorthQueryWorkflowArtifactAuthority,
    WorthQueryWorkflowArtifactRegistry, WorthQueryWorkflowArtifactRegistryEvidence,
};

pub(crate) struct WorthQueryFrozenWorkflowArtifactAuthority {
    authority: Option<WorthQueryWorkflowArtifactAuthority>,
    production_generation: WorthQueryArtifactProductionGeneration,
}

impl WorthQueryWorkflowArtifactRegistry {
    pub(super) fn is_frozen_at(
        &self,
        production_generation: WorthQueryArtifactProductionGeneration,
    ) -> bool {
        self.state
            .lock()
            .expect("workflow artifact registry lock must remain available")
            .posture
            == WorthQueryWorkflowArtifactRegistryPosture::Frozen(production_generation)
    }
}

impl WorthQueryFrozenWorkflowArtifactAuthority {
    pub(super) const fn new(
        authority: WorthQueryWorkflowArtifactAuthority,
        production_generation: WorthQueryArtifactProductionGeneration,
    ) -> Self {
        Self {
            authority: Some(authority),
            production_generation,
        }
    }

    pub(crate) fn registry(&self) -> Arc<WorthQueryWorkflowArtifactRegistry> {
        self.authority().registry()
    }

    pub(crate) fn evidence(&self) -> WorthQueryWorkflowArtifactRegistryEvidence {
        self.authority().registry().evidence()
    }

    pub(crate) const fn production_generation(&self) -> WorthQueryArtifactProductionGeneration {
        self.production_generation
    }

    pub(crate) fn registry_is_frozen_at_owned_generation(&self) -> bool {
        self.authority()
            .registry()
            .is_frozen_at(self.production_generation)
    }

    pub(crate) fn production_authority_for_readmission(
        &self,
        stage_identity: &str,
        pending: &WorthQueryArtifactProductionGenerationPending,
    ) -> Result<Option<Arc<WorthQueryArtifactProductionAuthority>>, WorthQueryArtifactDenial> {
        self.authority()
            .production_authority_for_readmission(stage_identity, pending)
    }

    pub(crate) fn activate_after_readmission(
        mut self,
        committed: WorthQueryArtifactProductionGenerationCommitted,
    ) -> WorthQueryWorkflowArtifactAuthority {
        assert!(
            committed.belongs_to(&self.authority().registry())
                && committed.prior() == self.production_generation,
            "artifact generation commit must advance this frozen workflow authority",
        );
        self.authority
            .take()
            .expect("valid readmission consumes the frozen workflow authority")
    }

    fn authority(&self) -> &WorthQueryWorkflowArtifactAuthority {
        self.authority
            .as_ref()
            .expect("frozen workflow authority remains owned until cleanup or readmission")
    }
}

impl Drop for WorthQueryFrozenWorkflowArtifactAuthority {
    fn drop(&mut self) {
        if let Some(authority) = &self.authority {
            authority.registry().close_cancelled();
        }
    }
}
