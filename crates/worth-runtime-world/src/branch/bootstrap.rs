use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::branch::{ProductBranchCreationIntent, ProductBranchObservation};
use crate::identity::{ProductBranchReferenceGeneration, RuntimeWorldBootstrapAttemptIdentity};
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_runtime_bridge::facade::AdmittedRuntimeWorldCorrespondenceBasis;
use worth_signal::facade::branch::AdmittedSignalBranchBasis;

/// Exact inputs needed for the one root bootstrap operation. All component
/// values are already owner-admitted; this intent cannot read ambient heads.
#[derive(Debug, Clone)]
pub struct RuntimeWorldBootstrapIntent {
    creation: ProductBranchCreationIntent,
    relational_basis: AdmittedRelationalBranchBasis,
    signal_basis: AdmittedSignalBranchBasis,
    correspondence_basis: AdmittedRuntimeWorldCorrespondenceBasis,
    cancellation: Option<crate::publication::RuntimeWorldCancellationToken>,
    initial_generation: ProductBranchReferenceGeneration,
}

impl RuntimeWorldBootstrapIntent {
    pub fn new(
        creation: ProductBranchCreationIntent,
        relational_basis: AdmittedRelationalBranchBasis,
        signal_basis: AdmittedSignalBranchBasis,
        correspondence_basis: AdmittedRuntimeWorldCorrespondenceBasis,
    ) -> Self {
        Self {
            creation,
            relational_basis,
            signal_basis,
            correspondence_basis,
            cancellation: None,
            initial_generation: ProductBranchReferenceGeneration::initial(),
        }
    }

    pub fn with_cancellation(
        mut self,
        cancellation: crate::publication::RuntimeWorldCancellationToken,
    ) -> Self {
        self.cancellation = Some(cancellation);
        self
    }
    pub(crate) fn cancellation(
        &self,
    ) -> Option<&crate::publication::RuntimeWorldCancellationToken> {
        self.cancellation.as_ref()
    }
    pub fn creation(&self) -> &ProductBranchCreationIntent {
        &self.creation
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        ProductBranchCreationIntent,
        AdmittedRelationalBranchBasis,
        AdmittedSignalBranchBasis,
        AdmittedRuntimeWorldCorrespondenceBasis,
        ProductBranchReferenceGeneration,
    ) {
        (
            self.creation,
            self.relational_basis,
            self.signal_basis,
            self.correspondence_basis,
            self.initial_generation,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeWorldBootstrapNoEffectCause {
    AlreadyBootstrapped,
    ForeignBasis,
    IncompatibleCorrespondence,
    CapacityExhausted,
    IdentityExhausted,
    Cancelled,
    OwnerUnavailable,
}

/// Linear proof that the root commit and first product reference were
/// installed together by the Runtime World owner.
#[must_use = "a performed bootstrap must be retained by its owner"]
#[derive(Debug)]
pub struct PerformedRuntimeWorldBootstrap {
    attempt: RuntimeWorldBootstrapAttemptIdentity,
    basis: AdmittedCompositeRuntimeWorldBasis,
    product_branch: ProductBranchObservation,
}

impl PerformedRuntimeWorldBootstrap {
    pub fn attempt(&self) -> &RuntimeWorldBootstrapAttemptIdentity {
        &self.attempt
    }

    pub fn basis(&self) -> &AdmittedCompositeRuntimeWorldBasis {
        &self.basis
    }

    pub fn product_branch(&self) -> &ProductBranchObservation {
        &self.product_branch
    }

    pub(crate) fn new(
        attempt: RuntimeWorldBootstrapAttemptIdentity,
        basis: AdmittedCompositeRuntimeWorldBasis,
        product_branch: ProductBranchObservation,
    ) -> Self {
        Self {
            attempt,
            basis,
            product_branch,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoEffectRuntimeWorldBootstrap {
    cause: RuntimeWorldBootstrapNoEffectCause,
}

impl NoEffectRuntimeWorldBootstrap {
    pub const fn cause(self) -> RuntimeWorldBootstrapNoEffectCause {
        self.cause
    }

    pub(crate) const fn new(cause: RuntimeWorldBootstrapNoEffectCause) -> Self {
        Self { cause }
    }
}

#[must_use = "bootstrap outcomes carry the only root-installation decision"]
#[derive(Debug)]
pub enum RuntimeWorldBootstrapOutcome {
    Performed(PerformedRuntimeWorldBootstrap),
    NoEffect(NoEffectRuntimeWorldBootstrap),
}

#[cfg(test)]
#[path = "bootstrap_tests.rs"]
mod tests;
