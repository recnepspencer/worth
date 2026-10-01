use crate::physical_runtime::{
    PhysicalMutationIdempotencyKeyIdentity, PhysicalMutationIdentity,
    PhysicalMutationRequestFingerprint,
};
use std::sync::Arc;

mod root_preparation;
pub use root_preparation::{
    PhysicalMutationRootPreparationFailure, PhysicalRootPreparationEffectPosture,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalMutationIndeterminateStage {
    WalAppend,
    WalDurabilityBarrier,
    DataDispatch,
    DataSettlement,
    RootPreparation,
    RootReplacement,
    RootNamespaceDurability,
    CurrentRootAdvance,
    WorkerPanicked,
    RuntimeUnavailable,
}

impl PhysicalMutationIndeterminateStage {
    pub(in crate::physical_runtime) const fn encoding_code(self) -> u8 {
        match self {
            Self::WalAppend => 1,
            Self::WalDurabilityBarrier => 2,
            Self::DataDispatch => 3,
            Self::DataSettlement => 4,
            Self::RootPreparation => 5,
            Self::RootReplacement => 6,
            Self::RootNamespaceDurability => 7,
            Self::CurrentRootAdvance => 8,
            Self::WorkerPanicked => 9,
            Self::RuntimeUnavailable => 10,
        }
    }

    pub(in crate::physical_runtime) const fn decode(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::WalAppend),
            2 => Some(Self::WalDurabilityBarrier),
            3 => Some(Self::DataDispatch),
            4 => Some(Self::DataSettlement),
            5 => Some(Self::RootPreparation),
            6 => Some(Self::RootReplacement),
            7 => Some(Self::RootNamespaceDurability),
            8 => Some(Self::CurrentRootAdvance),
            9 => Some(Self::WorkerPanicked),
            10 => Some(Self::RuntimeUnavailable),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndeterminatePhysicalMutation {
    mutation: PhysicalMutationIdentity,
    idempotency: PhysicalMutationIdempotencyKeyIdentity,
    fingerprint: PhysicalMutationRequestFingerprint,
    stage: PhysicalMutationIndeterminateStage,
    completed_effects: u32,
    root_preparation_failure: Option<Arc<PhysicalMutationRootPreparationFailure>>,
}

impl IndeterminatePhysicalMutation {
    pub(in crate::physical_runtime) fn possible_effect(
        mutation: PhysicalMutationIdentity,
        idempotency: PhysicalMutationIdempotencyKeyIdentity,
        fingerprint: PhysicalMutationRequestFingerprint,
        stage: PhysicalMutationIndeterminateStage,
        completed_effects: usize,
    ) -> Self {
        Self {
            mutation,
            idempotency,
            fingerprint,
            stage,
            completed_effects: u32::try_from(completed_effects)
                .expect("bounded physical effect count fits u32"),
            root_preparation_failure: None,
        }
    }

    pub const fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.mutation
    }

    pub const fn idempotency_identity(&self) -> PhysicalMutationIdempotencyKeyIdentity {
        self.idempotency
    }

    pub const fn request_fingerprint(&self) -> PhysicalMutationRequestFingerprint {
        self.fingerprint
    }

    pub const fn stage(&self) -> PhysicalMutationIndeterminateStage {
        self.stage
    }

    pub const fn completed_effect_count(&self) -> u32 {
        self.completed_effects
    }

    pub fn diagnostic_evidence(
        &self,
    ) -> crate::physical_runtime::IndeterminatePhysicalMutationEvidence {
        crate::physical_runtime::IndeterminatePhysicalMutationEvidence::from_fate(self)
    }

    pub(in crate::physical_runtime) fn with_root_preparation_failure(
        mut self,
        cause: crate::physical_runtime::PhysicalRootPublicationPreparationFailureCause,
        posture: PhysicalRootPreparationEffectPosture,
    ) -> Self {
        assert_eq!(
            self.stage,
            PhysicalMutationIndeterminateStage::RootPreparation
        );
        self.root_preparation_failure = Some(Arc::new(
            PhysicalMutationRootPreparationFailure::new(cause, posture),
        ));
        self
    }

    /// Present for a live managed root-preparation failure. Legacy persisted
    /// fates retain recovery identity and stage, but do not store this detail.
    pub fn root_preparation_failure(&self) -> Option<&PhysicalMutationRootPreparationFailure> {
        self.root_preparation_failure.as_deref()
    }
}
