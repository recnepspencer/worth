use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::branch::{ProductBranchCreationIntent, ProductBranchObservation};
use crate::identity::{ProductBranchReferenceGeneration, RuntimeWorldBootstrapAttemptIdentity};
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::durability::RecoveredRelationalBranchBasis;
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
    recovered_authoritative_root: bool,
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
            recovered_authoritative_root: false,
        }
    }

    /// Builds a recovered root intent only from a Relational owner-issued basis
    /// that proves successful native recovery. Checkpoint presence alone cannot
    /// select this path.
    pub fn recovered(
        creation: ProductBranchCreationIntent,
        relational_basis: RecoveredRelationalBranchBasis,
        signal_basis: AdmittedSignalBranchBasis,
        correspondence_basis: AdmittedRuntimeWorldCorrespondenceBasis,
    ) -> Self {
        Self {
            creation,
            relational_basis: relational_basis.into_basis(),
            signal_basis,
            correspondence_basis,
            cancellation: None,
            initial_generation: ProductBranchReferenceGeneration::initial(),
            recovered_authoritative_root: true,
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

    pub fn relational_basis(&self) -> &AdmittedRelationalBranchBasis {
        &self.relational_basis
    }

    pub fn signal_basis(&self) -> &AdmittedSignalBranchBasis {
        &self.signal_basis
    }

    pub fn correspondence_basis(&self) -> &AdmittedRuntimeWorldCorrespondenceBasis {
        &self.correspondence_basis
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        ProductBranchCreationIntent,
        AdmittedRelationalBranchBasis,
        AdmittedSignalBranchBasis,
        AdmittedRuntimeWorldCorrespondenceBasis,
        ProductBranchReferenceGeneration,
        bool,
    ) {
        (
            self.creation,
            self.relational_basis,
            self.signal_basis,
            self.correspondence_basis,
            self.initial_generation,
            self.recovered_authoritative_root,
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
    recovered_authoritative_root: bool,
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

    /// Consumes the bootstrap proof into fresh, runtime-scoped authority for
    /// adopting accepted derived outputs at this recovered root. A fresh
    /// bootstrap cannot issue recovery authority.
    pub fn into_recovered_root_authority(self) -> Option<RecoveredRuntimeWorldRootAuthority> {
        self.recovered_authoritative_root
            .then_some(RecoveredRuntimeWorldRootAuthority {
                attempt: self.attempt,
                basis: self.basis,
                product_branch: self.product_branch,
            })
    }

    pub(crate) fn new(
        attempt: RuntimeWorldBootstrapAttemptIdentity,
        basis: AdmittedCompositeRuntimeWorldBasis,
        product_branch: ProductBranchObservation,
        recovered_authoritative_root: bool,
    ) -> Self {
        Self {
            attempt,
            basis,
            product_branch,
            recovered_authoritative_root,
        }
    }
}

/// World-issued authority to adopt checkpoint-accepted derived outputs at the
/// exact recovered root. It is runtime-scoped and deliberately non-serializable.
#[derive(Debug)]
pub struct RecoveredRuntimeWorldRootAuthority {
    attempt: RuntimeWorldBootstrapAttemptIdentity,
    basis: AdmittedCompositeRuntimeWorldBasis,
    product_branch: ProductBranchObservation,
}

impl RecoveredRuntimeWorldRootAuthority {
    pub fn attempt(&self) -> &RuntimeWorldBootstrapAttemptIdentity {
        &self.attempt
    }

    pub fn basis(&self) -> &AdmittedCompositeRuntimeWorldBasis {
        &self.basis
    }

    pub fn product_branch(&self) -> &ProductBranchObservation {
        &self.product_branch
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
