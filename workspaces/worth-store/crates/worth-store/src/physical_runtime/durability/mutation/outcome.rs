use std::sync::Arc;

use super::progression::CompletedPhysicalMutation;
use super::progression::CompletedPhysicalMutationFact;
use crate::physical_runtime::durability::settlement::{
    IndeterminatePhysicalMutation, ProvenNoEffectPhysicalMutation,
};
use crate::physical_runtime::PhysicalMutationIdentity;

pub enum PhysicalMutationOutcome {
    Completed(CompletedPhysicalMutation),
    ProvenNoEffect(ProvenNoEffectPhysicalMutation),
    Indeterminate(IndeterminatePhysicalMutation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalMutationProgressPhase {
    Admitted,
    WalAppend,
    WalDurabilityBarrier,
    DataDispatch,
    DataSettlement,
    RootPreparation,
    RootReplacement,
    RootNamespaceDurability,
    CurrentRootAdvance,
    Terminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalMutationProgress {
    identity: PhysicalMutationIdentity,
    phase: PhysicalMutationProgressPhase,
    cancellation_requested: bool,
    runtime_closing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalMutationTerminalObservation {
    Completed(PhysicalMutationIdentity),
    ProvenNoEffect(PhysicalMutationIdentity),
    Indeterminate(PhysicalMutationIdentity),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalMutationPoll {
    Pending(PhysicalMutationProgress),
    Terminal(PhysicalMutationTerminalObservation),
}

pub enum PhysicalMutationCancellationOutcome {
    AcceptedBeforeEffect {
        identity: PhysicalMutationIdentity,
    },
    SettlementAlreadyEffectful {
        identity: PhysicalMutationIdentity,
        phase: PhysicalMutationProgressPhase,
    },
    AlreadyTerminal(PhysicalMutationTerminalObservation),
    StaleHandle {
        identity: PhysicalMutationIdentity,
    },
    RuntimeClosing {
        identity: PhysicalMutationIdentity,
    },
}

pub(in crate::physical_runtime) enum PhysicalMutationTerminalFact {
    Completed(Arc<CompletedPhysicalMutationFact>),
    ProvenNoEffect(ProvenNoEffectPhysicalMutation),
    Indeterminate(IndeterminatePhysicalMutation),
}

impl PhysicalMutationProgress {
    pub(in crate::physical_runtime) const fn admitted(identity: PhysicalMutationIdentity) -> Self {
        Self {
            identity,
            phase: PhysicalMutationProgressPhase::Admitted,
            cancellation_requested: false,
            runtime_closing: false,
        }
    }

    pub const fn identity(self) -> PhysicalMutationIdentity {
        self.identity
    }

    pub const fn phase(self) -> PhysicalMutationProgressPhase {
        self.phase
    }

    pub const fn cancellation_requested(self) -> bool {
        self.cancellation_requested
    }

    pub const fn runtime_closing(self) -> bool {
        self.runtime_closing
    }

    pub(in crate::physical_runtime) fn enter(&mut self, phase: PhysicalMutationProgressPhase) {
        self.phase = phase;
    }

    pub(in crate::physical_runtime) fn request_cancellation(&mut self) {
        self.cancellation_requested = true;
    }

    pub(in crate::physical_runtime) fn mark_runtime_closing(&mut self) {
        self.runtime_closing = true;
    }
}

impl PhysicalMutationTerminalFact {
    pub(in crate::physical_runtime) fn outcome(&self) -> PhysicalMutationOutcome {
        match self {
            Self::Completed(fact) => {
                PhysicalMutationOutcome::Completed(CompletedPhysicalMutation::from_fact(fact))
            }
            Self::ProvenNoEffect(fate) => PhysicalMutationOutcome::ProvenNoEffect(fate.clone()),
            Self::Indeterminate(fate) => PhysicalMutationOutcome::Indeterminate(fate.clone()),
        }
    }

    pub(in crate::physical_runtime) fn observation(&self) -> PhysicalMutationTerminalObservation {
        match self {
            Self::Completed(fact) => {
                PhysicalMutationTerminalObservation::Completed(fact.mutation_identity())
            }
            Self::ProvenNoEffect(fate) => {
                PhysicalMutationTerminalObservation::ProvenNoEffect(fate.mutation_identity())
            }
            Self::Indeterminate(fate) => {
                PhysicalMutationTerminalObservation::Indeterminate(fate.mutation_identity())
            }
        }
    }
}
