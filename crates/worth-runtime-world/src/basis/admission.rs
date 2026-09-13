use std::sync::Arc;

use worth_proof::AuthorityWitness;
use worth_relational::facade::branch::{
    AdmittedRelationalBranchBasis, RelationalBranchBasisDenial, RelationalBranchBasisPort,
};
use worth_runtime_bridge::facade::{
    AdmittedRuntimeWorldCorrespondenceBasis, RuntimeWorldCorrespondenceAdmissionDenial,
    RuntimeWorldCorrespondencePort,
};
use worth_signal::facade::branch::{
    AdmittedSignalBranchBasis, SignalBranchBasisPort, SignalBranchBasisReadmissionDenial,
};

use super::composite::CompositeRuntimeWorldBasis;
use crate::identity::{CompositeBasisKey, RuntimeWorldIdentityIssuer};

worth_proof::authority_marker!(pub(crate) CompositeBasisAdmissionAuthorityMarker);

/// Owner-admitted composite basis. The private proof prevents descriptors,
/// equal-looking component values, or a consumer projection from minting it.
#[derive(Debug, Clone)]
pub struct AdmittedCompositeRuntimeWorldBasis {
    inner: Arc<AdmittedCompositeRuntimeWorldBasisInner>,
}

#[derive(Debug, Clone)]
struct AdmittedCompositeRuntimeWorldBasisInner {
    basis: CompositeRuntimeWorldBasis,
    admission: CompositeBasisAdmissionBinding,
}

#[derive(Debug, Clone)]
struct CompositeBasisAdmissionBinding {
    identity: CompositeBasisKey,
    _authority: Arc<AuthorityWitness<CompositeBasisAdmissionAuthorityMarker>>,
}

impl PartialEq for AdmittedCompositeRuntimeWorldBasis {
    fn eq(&self, other: &Self) -> bool {
        self.inner.admission.identity == other.inner.admission.identity
    }
}

impl Eq for AdmittedCompositeRuntimeWorldBasis {}

impl AdmittedCompositeRuntimeWorldBasis {
    fn new(basis: CompositeRuntimeWorldBasis, identity: CompositeBasisKey) -> Self {
        let authority =
            AuthorityWitness::from_authority_marker(CompositeBasisAdmissionAuthorityMarker::seal());
        Self {
            inner: Arc::new(AdmittedCompositeRuntimeWorldBasisInner {
                basis,
                admission: CompositeBasisAdmissionBinding {
                    identity,
                    _authority: Arc::new(authority),
                },
            }),
        }
    }

    pub fn owner_identity(&self) -> crate::identity::RuntimeWorldOwnerIdentity {
        self.identity().owner_identity()
    }

    pub fn identity(&self) -> &CompositeBasisKey {
        &self.inner.admission.identity
    }

    pub fn relational_basis(&self) -> &AdmittedRelationalBranchBasis {
        self.inner.basis.relational_basis()
    }

    pub fn signal_basis(&self) -> &AdmittedSignalBranchBasis {
        self.inner.basis.signal_basis()
    }

    pub fn correspondence_basis(&self) -> &AdmittedRuntimeWorldCorrespondenceBasis {
        self.inner.basis.correspondence_basis()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CompositeBasisAdmissionDenial {
    Relational(RelationalBranchBasisDenial),
    Signal(SignalBranchBasisReadmissionDenial),
    Correspondence(RuntimeWorldCorrespondenceAdmissionDenial),
}

/// Admit a live component tuple only after the Signal owner has checked the
/// exact basis against its current owner cell. The World identity is issued
/// only after that owner-side admission succeeds.
pub(crate) fn admit_current<D, I, T>(
    identities: &RuntimeWorldIdentityIssuer,
    relational_port: &RelationalBranchBasisPort,
    signal_port: &SignalBranchBasisPort<D, I, T>,
    correspondence_port: &RuntimeWorldCorrespondencePort,
    relational: AdmittedRelationalBranchBasis,
    signal: AdmittedSignalBranchBasis,
    correspondence: AdmittedRuntimeWorldCorrespondenceBasis,
) -> Result<AdmittedCompositeRuntimeWorldBasis, CompositeBasisAdmissionDenial>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    validate_current(
        relational_port,
        signal_port,
        correspondence_port,
        &relational,
        &signal,
        &correspondence,
    )?;
    Ok(admit_validated(
        identities,
        relational,
        signal,
        correspondence,
    ))
}

/// Recheck all component owners without holding the Runtime World identity
/// mutex. Identity issuance is deliberately a separate, non-owner step.
pub(crate) fn validate_current<D, I, T>(
    relational_port: &RelationalBranchBasisPort,
    signal_port: &SignalBranchBasisPort<D, I, T>,
    correspondence_port: &RuntimeWorldCorrespondencePort,
    relational: &AdmittedRelationalBranchBasis,
    signal: &AdmittedSignalBranchBasis,
    correspondence: &AdmittedRuntimeWorldCorrespondenceBasis,
) -> Result<(), CompositeBasisAdmissionDenial>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    relational_port
        .compare_current_exact(relational)
        .map_err(CompositeBasisAdmissionDenial::Relational)?;
    signal_port
        .compare_current_exact(signal)
        .map_err(CompositeBasisAdmissionDenial::Signal)?;
    correspondence_port
        .compare_current_exact(correspondence)
        .map_err(CompositeBasisAdmissionDenial::Correspondence)?;
    Ok(())
}

pub(crate) fn admit_validated(
    identities: &RuntimeWorldIdentityIssuer,
    relational: AdmittedRelationalBranchBasis,
    signal: AdmittedSignalBranchBasis,
    correspondence: AdmittedRuntimeWorldCorrespondenceBasis,
) -> AdmittedCompositeRuntimeWorldBasis {
    let identity = identities.composite_basis(
        relational.admission_identity().clone(),
        signal.admission_identity().clone(),
        correspondence.admission_identity().clone(),
    );
    let basis = CompositeRuntimeWorldBasis::admit(relational, signal, correspondence);
    AdmittedCompositeRuntimeWorldBasis::new(basis, identity)
}

#[cfg(test)]
#[path = "admission_tests.rs"]
mod tests;
