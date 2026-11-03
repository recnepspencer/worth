use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};

use super::registry_evidence::WorthQueryArtifactLifecycleSnapshotGate;
use super::{
    WorthQueryArtifactDenial, WorthQueryArtifactDenialKind, WorthQueryArtifactDisposition,
    WorthQueryArtifactLifecycleRecord, WorthQueryArtifactProductionGeneration,
    WorthQueryArtifactProductionGenerationPending, WorthQueryMoveOnlyArtifactHandle,
    WorthQueryRuntimeArtifactOwner, WorthQueryWorkflowArtifactRegistryEvidence,
};

pub struct WorthQueryWorkflowArtifactRegistry {
    run_identity: String,
    pub(super) state: Mutex<WorthQueryWorkflowArtifactRegistryState>,
    pub(super) snapshot_gate: Arc<WorthQueryArtifactLifecycleSnapshotGate>,
}

pub(super) struct WorthQueryWorkflowArtifactRegistryState {
    pub(super) posture: WorthQueryWorkflowArtifactRegistryPosture,
    pub(super) owners: BTreeMap<String, WorthQueryWorkflowArtifactRegistryEntry>,
}

pub(super) struct WorthQueryWorkflowArtifactRegistryEntry {
    owner: Weak<WorthQueryRuntimeArtifactOwner>,
    pub(super) lifecycle: Arc<WorthQueryArtifactLifecycleRecord>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum WorthQueryWorkflowArtifactRegistryPosture {
    Producing(WorthQueryArtifactProductionGeneration),
    YieldFreezePending(WorthQueryArtifactProductionGeneration),
    Frozen(WorthQueryArtifactProductionGeneration),
    ReadmissionPending {
        prior: WorthQueryArtifactProductionGeneration,
        next: WorthQueryArtifactProductionGeneration,
    },
    Closed(WorthQueryArtifactProductionGeneration),
}

impl WorthQueryWorkflowArtifactRegistry {
    pub(super) fn new(run_identity: String) -> Self {
        Self {
            run_identity,
            state: Mutex::new(WorthQueryWorkflowArtifactRegistryState {
                posture: WorthQueryWorkflowArtifactRegistryPosture::Producing(
                    WorthQueryArtifactProductionGeneration::initial(),
                ),
                owners: BTreeMap::new(),
            }),
            snapshot_gate: Arc::new(WorthQueryArtifactLifecycleSnapshotGate::new()),
        }
    }

    pub fn run_identity(&self) -> &str {
        &self.run_identity
    }

    pub fn register(
        &self,
        handle: &WorthQueryMoveOnlyArtifactHandle,
        production_generation: WorthQueryArtifactProductionGeneration,
    ) -> Result<(), WorthQueryArtifactDenial> {
        if handle.core.owner.binding().run_identity != self.run_identity {
            return Err(WorthQueryArtifactDenial::new(
                WorthQueryArtifactDenialKind::RunMismatch,
                Some(
                    handle
                        .core
                        .owner
                        .binding()
                        .contract
                        .contract()
                        .family()
                        .as_str(),
                ),
                "artifact registry accepts owners from its exact workflow run",
            ));
        }
        let owner = &handle.core.owner;
        let _snapshot = self.snapshot_gate.lifecycle_mutation();
        let mut state = self
            .state
            .lock()
            .expect("workflow artifact registry lock must remain available");
        validate_registration_posture(state.posture, production_generation)?;
        let replaced = state.owners.insert(
            owner.binding().owner_identity.clone(),
            WorthQueryWorkflowArtifactRegistryEntry {
                owner: Arc::downgrade(owner),
                lifecycle: owner.lifecycle_record(),
            },
        );
        debug_assert!(replaced.is_none(), "artifact owner identity is unique");
        Ok(())
    }

    pub(super) fn admit_registration(
        &self,
        production_generation: WorthQueryArtifactProductionGeneration,
    ) -> Result<(), WorthQueryArtifactDenial> {
        let posture = self
            .state
            .lock()
            .expect("workflow artifact registry lock must remain available")
            .posture;
        validate_registration_posture(posture, production_generation)
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.state
            .lock()
            .expect("workflow artifact registry lock must remain available")
            .posture
            .is_closed()
    }

    pub(crate) fn freeze_production(&self) -> WorthQueryWorkflowArtifactRegistryEvidence {
        let mut state = self
            .state
            .lock()
            .expect("workflow artifact registry lock must remain available");
        if let WorthQueryWorkflowArtifactRegistryPosture::Producing(generation) = state.posture {
            state.posture = WorthQueryWorkflowArtifactRegistryPosture::Frozen(generation);
        }
        drop(state);
        self.evidence()
    }

    pub(super) fn lifecycle_snapshot_gate(&self) -> Arc<WorthQueryArtifactLifecycleSnapshotGate> {
        Arc::clone(&self.snapshot_gate)
    }

    pub(super) fn current_production_generation(
        &self,
    ) -> Result<WorthQueryArtifactProductionGeneration, WorthQueryArtifactDenial> {
        let posture = self
            .state
            .lock()
            .expect("workflow artifact registry lock must remain available")
            .posture;
        match posture {
            WorthQueryWorkflowArtifactRegistryPosture::Producing(generation) => Ok(generation),
            posture => Err(registration_denial(posture)),
        }
    }

    pub(crate) fn prepare_next_generation(
        self: &Arc<Self>,
    ) -> Result<WorthQueryArtifactProductionGenerationPending, WorthQueryArtifactDenial> {
        let mut state = self
            .state
            .lock()
            .expect("workflow artifact registry lock must remain available");
        let WorthQueryWorkflowArtifactRegistryPosture::Frozen(prior) = state.posture else {
            return Err(generation_transition_denial(
                "artifact production generation can advance only from a frozen yielded run",
            ));
        };
        let next = prior.next().ok_or_else(|| {
            generation_transition_denial("artifact production generation is exhausted")
        })?;
        state.posture =
            WorthQueryWorkflowArtifactRegistryPosture::ReadmissionPending { prior, next };
        Ok(WorthQueryArtifactProductionGenerationPending::new(
            Arc::clone(self),
            prior,
            next,
        ))
    }

    pub(super) fn abort_generation(
        &self,
        prior: WorthQueryArtifactProductionGeneration,
        next: WorthQueryArtifactProductionGeneration,
    ) -> Result<(), WorthQueryArtifactDenial> {
        let mut state = self
            .state
            .lock()
            .expect("workflow artifact registry lock must remain available");
        if state.posture
            != (WorthQueryWorkflowArtifactRegistryPosture::ReadmissionPending { prior, next })
        {
            return Err(generation_transition_denial(
                "artifact generation abort no longer owns the pending transition",
            ));
        }
        state.posture = WorthQueryWorkflowArtifactRegistryPosture::Frozen(prior);
        Ok(())
    }

    pub(super) fn commit_generation(
        &self,
        prior: WorthQueryArtifactProductionGeneration,
        next: WorthQueryArtifactProductionGeneration,
    ) {
        let mut state = self
            .state
            .lock()
            .expect("workflow artifact registry lock must remain available");
        assert!(
            state.posture
                == (WorthQueryWorkflowArtifactRegistryPosture::ReadmissionPending { prior, next }),
            "artifact generation pending authority must exclusively own its commit transition",
        );
        state.posture = WorthQueryWorkflowArtifactRegistryPosture::Producing(next);
    }

    pub fn close_released(&self) {
        self.close(WorthQueryArtifactDisposition::Released);
    }

    pub fn close_cancelled(&self) {
        self.close(WorthQueryArtifactDisposition::Cancelled);
    }

    fn close(&self, disposition: WorthQueryArtifactDisposition) {
        let owners = {
            let mut state = self
                .state
                .lock()
                .expect("workflow artifact registry lock must remain available");
            let generation = production_generation(state.posture);
            state.posture = WorthQueryWorkflowArtifactRegistryPosture::Closed(generation);

            state
                .owners
                .values()
                .filter_map(|entry| entry.owner.upgrade())
                .collect::<Vec<_>>()
        };
        for owner in owners {
            owner.request_registry_close(disposition);
        }
    }
}

fn registration_denial(
    posture: WorthQueryWorkflowArtifactRegistryPosture,
) -> WorthQueryArtifactDenial {
    match posture {
        WorthQueryWorkflowArtifactRegistryPosture::Producing(_) => {
            unreachable!("producing registries admit artifact registration")
        }
        WorthQueryWorkflowArtifactRegistryPosture::Frozen(_)
        | WorthQueryWorkflowArtifactRegistryPosture::YieldFreezePending(_)
        | WorthQueryWorkflowArtifactRegistryPosture::ReadmissionPending { .. } => {
            WorthQueryArtifactDenial::new(
                WorthQueryArtifactDenialKind::ProductionClosed,
                None,
                "workflow artifact production is frozen at a terminal lifecycle boundary",
            )
        }
        WorthQueryWorkflowArtifactRegistryPosture::Closed(_) => WorthQueryArtifactDenial::new(
            WorthQueryArtifactDenialKind::AlreadyDisposed,
            None,
            "workflow artifact registry is closed",
        ),
    }
}

fn validate_registration_posture(
    posture: WorthQueryWorkflowArtifactRegistryPosture,
    production_generation: WorthQueryArtifactProductionGeneration,
) -> Result<(), WorthQueryArtifactDenial> {
    match posture {
        WorthQueryWorkflowArtifactRegistryPosture::Producing(current)
            if current == production_generation =>
        {
            Ok(())
        }
        WorthQueryWorkflowArtifactRegistryPosture::Producing(_) => {
            Err(generation_transition_denial(
                "artifact producer belongs to a stale production generation",
            ))
        }
        posture => Err(registration_denial(posture)),
    }
}

pub(super) fn production_generation(
    posture: WorthQueryWorkflowArtifactRegistryPosture,
) -> WorthQueryArtifactProductionGeneration {
    match posture {
        WorthQueryWorkflowArtifactRegistryPosture::Producing(generation)
        | WorthQueryWorkflowArtifactRegistryPosture::YieldFreezePending(generation)
        | WorthQueryWorkflowArtifactRegistryPosture::Frozen(generation) => generation,
        WorthQueryWorkflowArtifactRegistryPosture::ReadmissionPending { prior, .. } => prior,
        WorthQueryWorkflowArtifactRegistryPosture::Closed(generation) => generation,
    }
}

impl WorthQueryWorkflowArtifactRegistryPosture {
    const fn is_closed(self) -> bool {
        matches!(self, Self::Closed(_))
    }
}

fn generation_transition_denial(detail: &'static str) -> WorthQueryArtifactDenial {
    WorthQueryArtifactDenial::new(
        WorthQueryArtifactDenialKind::StaleLifecycleGeneration,
        None,
        detail,
    )
}

impl Drop for WorthQueryWorkflowArtifactRegistry {
    fn drop(&mut self) {
        self.close(WorthQueryArtifactDisposition::Cancelled);
    }
}
