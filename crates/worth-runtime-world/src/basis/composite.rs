use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_runtime_bridge::facade::AdmittedRuntimeWorldCorrespondenceBasis;
use worth_signal::facade::branch::AdmittedSignalBranchBasis;

/// The exact ordered component tuple that constitutes one Runtime World
/// basis. It is descriptive storage behind the admitted wrapper; its
/// owner-issued component admission identities are bound separately by the
/// Runtime World identity issuer. It has no public equality or constructor.
#[derive(Debug, Clone)]
pub(crate) struct CompositeRuntimeWorldBasis {
    relational: AdmittedRelationalBranchBasis,
    signal: AdmittedSignalBranchBasis,
    correspondence: AdmittedRuntimeWorldCorrespondenceBasis,
}

impl CompositeRuntimeWorldBasis {
    pub(crate) fn relational_basis(&self) -> &AdmittedRelationalBranchBasis {
        &self.relational
    }

    pub(crate) fn signal_basis(&self) -> &AdmittedSignalBranchBasis {
        &self.signal
    }

    pub(crate) fn correspondence_basis(&self) -> &AdmittedRuntimeWorldCorrespondenceBasis {
        &self.correspondence
    }

    pub(crate) fn admit(
        relational: AdmittedRelationalBranchBasis,
        signal: AdmittedSignalBranchBasis,
        correspondence: AdmittedRuntimeWorldCorrespondenceBasis,
    ) -> Self {
        Self {
            relational,
            signal,
            correspondence,
        }
    }
}
