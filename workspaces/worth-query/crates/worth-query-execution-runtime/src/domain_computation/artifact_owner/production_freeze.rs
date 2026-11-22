use super::registry::WorthQueryWorkflowArtifactRegistryPosture;
use super::{
    WorthQueryArtifactDenial, WorthQueryArtifactProductionGeneration,
    WorthQueryFrozenWorkflowArtifactAuthority, WorthQueryWorkflowArtifactAuthority,
    WorthQueryWorkflowArtifactRegistry, WorthQueryWorkflowArtifactRegistryEvidence,
};

pub struct WorthQueryWorkflowArtifactFreezePending {
    authority: Option<WorthQueryWorkflowArtifactAuthority>,
    production_generation: WorthQueryArtifactProductionGeneration,
    active: bool,
}

impl WorthQueryWorkflowArtifactFreezePending {
    pub fn prepare(
        authority: WorthQueryWorkflowArtifactAuthority,
    ) -> Result<
        Self,
        (
            WorthQueryWorkflowArtifactAuthority,
            WorthQueryArtifactDenial,
        ),
    > {
        let registry = authority.registry();
        match registry.prepare_yield_freeze() {
            Ok(production_generation) => Ok(Self {
                authority: Some(authority),
                production_generation,
                active: true,
            }),
            Err(denial) => Err((authority, denial)),
        }
    }

    pub fn evidence(&self) -> WorthQueryWorkflowArtifactRegistryEvidence {
        self.authority().registry().evidence()
    }

    pub fn abort(mut self) -> WorthQueryWorkflowArtifactAuthority {
        self.authority()
            .registry()
            .abort_yield_freeze(self.production_generation);
        self.active = false;
        self.authority
            .take()
            .expect("active freeze authority must remain owned until abort")
    }

    pub fn commit(mut self) -> WorthQueryFrozenWorkflowArtifactAuthority {
        self.authority()
            .registry()
            .commit_yield_freeze(self.production_generation);
        self.active = false;
        WorthQueryFrozenWorkflowArtifactAuthority::new(
            self.authority
                .take()
                .expect("active freeze authority must remain owned until commit"),
            self.production_generation,
        )
    }

    fn authority(&self) -> &WorthQueryWorkflowArtifactAuthority {
        self.authority
            .as_ref()
            .expect("active freeze authority must remain owned")
    }
}

impl Drop for WorthQueryWorkflowArtifactFreezePending {
    fn drop(&mut self) {
        if self.active {
            self.authority()
                .registry()
                .abort_yield_freeze(self.production_generation);
        }
    }
}

impl WorthQueryWorkflowArtifactRegistry {
    pub(super) fn prepare_yield_freeze(
        &self,
    ) -> Result<WorthQueryArtifactProductionGeneration, WorthQueryArtifactDenial> {
        let mut state = self
            .state
            .lock()
            .expect("workflow artifact registry lock must remain available");
        let WorthQueryWorkflowArtifactRegistryPosture::Producing(generation) = state.posture else {
            return Err(WorthQueryArtifactDenial::new(
                super::WorthQueryArtifactDenialKind::StaleLifecycleGeneration,
                None,
                "yield freeze requires the active artifact production generation",
            ));
        };
        state.posture = WorthQueryWorkflowArtifactRegistryPosture::YieldFreezePending(generation);
        Ok(generation)
    }

    pub(super) fn abort_yield_freeze(&self, generation: WorthQueryArtifactProductionGeneration) {
        let mut state = self
            .state
            .lock()
            .expect("workflow artifact registry lock must remain available");
        assert!(
            state.posture
                == WorthQueryWorkflowArtifactRegistryPosture::YieldFreezePending(generation),
            "yield freeze authority must exclusively own its abort transition",
        );
        state.posture = WorthQueryWorkflowArtifactRegistryPosture::Producing(generation);
    }

    pub(super) fn commit_yield_freeze(&self, generation: WorthQueryArtifactProductionGeneration) {
        let mut state = self
            .state
            .lock()
            .expect("workflow artifact registry lock must remain available");
        assert!(
            state.posture
                == WorthQueryWorkflowArtifactRegistryPosture::YieldFreezePending(generation),
            "yield freeze authority must exclusively own its commit transition",
        );
        state.posture = WorthQueryWorkflowArtifactRegistryPosture::Frozen(generation);
    }
}
