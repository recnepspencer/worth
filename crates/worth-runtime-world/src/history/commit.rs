use worth_relational::facade::history::RelationalCommitIdentity;
use worth_runtime_bridge::facade::AdmittedRuntimeWorldCorrespondenceBasis;
use worth_signal::facade::branch::SignalBranchBasisAdmissionIdentity;

use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::identity::{
    CompositeCommitIdentity, CompositePublicationAttemptIdentity,
    RuntimeWorldBootstrapAttemptIdentity,
};
use crate::publication::CompositeOwnerExecutionResults;

use super::OrdinaryParent;

#[path = "commit/component_evidence.rs"]
mod component_evidence;

/// Explicit change posture derived from owner-issued component evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeComponentChangePosture {
    RetainExact,
    Published,
}

/// Root bootstrap or one ordinary publication attempt is the complete
/// occurrence provenance of a composite commit. Variants are only materialized
/// by the coherent commit constructors below.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositeCommitParent {
    Root,
    Ordinary(OrdinaryParent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositeCommitProvenance {
    Bootstrap(RuntimeWorldBootstrapAttemptIdentity),
    Publication(CompositePublicationAttemptIdentity),
}

/// Descriptive correlation supplied by a caller. It cannot authorize, select,
/// or identify a commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompositeCallerCorrelation(u128);

impl CompositeCallerCorrelation {
    pub const fn new(value: u128) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompositeCommitConstructionDenial {
    OwnerMismatch,
    BasisMismatch,
}

#[derive(Debug, PartialEq, Eq)]
enum RelationalComponentEvidence {
    RetainedExact {
        basis: worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity,
    },
    Published {
        commit: RelationalCommitIdentity,
        basis: worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity,
    },
    Forked {
        target: worth_relational::facade::branch::RelationalBranchIdentity,
        basis: worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity,
    },
}

#[derive(Debug, PartialEq, Eq)]
enum SignalComponentEvidence {
    RetainedExact {
        basis: SignalBranchBasisAdmissionIdentity,
    },
    Published(CompositeSignalPublicationIdentity),
}

/// Exact owner-issued Signal result identity carried by a changed composite
/// component. The resulting admitted basis is the only identity shared by
/// advance and fork outcomes; the operation kind remains explicit for later
/// history consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompositeSignalPublicationIdentity {
    Advanced(SignalBranchBasisAdmissionIdentity),
    Forked(SignalBranchBasisAdmissionIdentity),
}

impl CompositeSignalPublicationIdentity {
    fn basis_identity(&self) -> &SignalBranchBasisAdmissionIdentity {
        match self {
            Self::Advanced(identity) | Self::Forked(identity) => identity,
        }
    }
}

/// Component evidence is a coherent sum: a component is either retained at
/// the admitted basis or carries the owner-issued result of its movement.
/// There is no independent posture or correspondence field to mix in.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CompositeComponentEvidence {
    relational: RelationalComponentEvidence,
    signal: SignalComponentEvidence,
}

#[derive(Debug)]
enum CompositeCommitLineage {
    Root {
        parent: CompositeCommitParent,
        provenance: CompositeCommitProvenance,
    },
    Ordinary {
        parent: CompositeCommitParent,
        provenance: CompositeCommitProvenance,
    },
}

impl CompositeCommitLineage {
    fn parent(&self) -> &CompositeCommitParent {
        match self {
            Self::Root { parent, .. } | Self::Ordinary { parent, .. } => parent,
        }
    }

    fn provenance(&self) -> &CompositeCommitProvenance {
        match self {
            Self::Root { provenance, .. } | Self::Ordinary { provenance, .. } => provenance,
        }
    }
}

/// One immutable single-parent composite commit. Its lineage and component
/// evidence are created as coherent variants, and correspondence is always
/// projected from the admitted basis rather than stored a second time.
#[derive(Debug)]
pub struct CompositeRuntimeWorldCommit {
    identity: CompositeCommitIdentity,
    basis: AdmittedCompositeRuntimeWorldBasis,
    lineage: CompositeCommitLineage,
    component_evidence: CompositeComponentEvidence,
    caller_correlation: Option<CompositeCallerCorrelation>,
}

impl CompositeRuntimeWorldCommit {
    pub(crate) fn from_root_bootstrap(
        identity: CompositeCommitIdentity,
        basis: AdmittedCompositeRuntimeWorldBasis,
        bootstrap: RuntimeWorldBootstrapAttemptIdentity,
        caller_correlation: Option<CompositeCallerCorrelation>,
    ) -> Result<Self, CompositeCommitConstructionDenial> {
        require_owner(identity.owner_identity(), basis.owner_identity())?;
        require_owner(identity.owner_identity(), bootstrap.owner_identity())?;
        let component_evidence = CompositeComponentEvidence::retained(&basis);
        Ok(Self {
            identity,
            basis,
            lineage: CompositeCommitLineage::Root {
                parent: CompositeCommitParent::Root,
                provenance: CompositeCommitProvenance::Bootstrap(bootstrap),
            },
            component_evidence,
            caller_correlation,
        })
    }

    pub(crate) fn from_ordinary_publication(
        identity: CompositeCommitIdentity,
        predecessor: &CompositeRuntimeWorldCommit,
        basis: AdmittedCompositeRuntimeWorldBasis,
        publication: CompositePublicationAttemptIdentity,
        owner_results: &CompositeOwnerExecutionResults,
        caller_correlation: Option<CompositeCallerCorrelation>,
    ) -> Result<Self, CompositeCommitConstructionDenial> {
        require_owner(identity.owner_identity(), basis.owner_identity())?;
        require_owner(
            identity.owner_identity(),
            predecessor.identity().owner_identity(),
        )?;
        require_owner(identity.owner_identity(), publication.owner_identity())?;
        require_owner(
            identity.owner_identity(),
            predecessor.basis().owner_identity(),
        )?;
        let component_evidence = CompositeComponentEvidence::from_owner_results(
            owner_results,
            predecessor.basis(),
            &basis,
        )?;
        Ok(Self {
            identity,
            basis,
            lineage: CompositeCommitLineage::Ordinary {
                parent: CompositeCommitParent::Ordinary(OrdinaryParent::new(
                    predecessor.identity().clone(),
                )),
                provenance: CompositeCommitProvenance::Publication(publication),
            },
            component_evidence,
            caller_correlation,
        })
    }

    pub fn identity(&self) -> &CompositeCommitIdentity {
        &self.identity
    }

    pub fn parent(&self) -> &CompositeCommitParent {
        self.lineage.parent()
    }

    pub fn basis(&self) -> &AdmittedCompositeRuntimeWorldBasis {
        &self.basis
    }

    pub fn relational_change(&self) -> CompositeComponentChangePosture {
        self.component_evidence.relational_posture()
    }

    pub fn signal_change(&self) -> CompositeComponentChangePosture {
        self.component_evidence.signal_posture()
    }

    /// Correspondence is derived from the one admitted composite basis. No
    /// independently supplied or stored correspondence authority exists.
    pub fn correspondence_basis(&self) -> &AdmittedRuntimeWorldCorrespondenceBasis {
        self.basis.correspondence_basis()
    }

    pub fn provenance(&self) -> &CompositeCommitProvenance {
        self.lineage.provenance()
    }

    pub fn caller_correlation(&self) -> Option<CompositeCallerCorrelation> {
        self.caller_correlation
    }

    pub fn relational_publication_identity(&self) -> Option<&RelationalCommitIdentity> {
        self.component_evidence.relational_publication_identity()
    }

    pub fn relational_fork_target_identity(
        &self,
    ) -> Option<&worth_relational::facade::branch::RelationalBranchIdentity> {
        self.component_evidence.relational_fork_target_identity()
    }

    pub fn signal_publication_identity(&self) -> Option<&CompositeSignalPublicationIdentity> {
        self.component_evidence.signal_publication_identity()
    }

    pub(crate) fn matches_owner_results(
        &self,
        predecessor: &AdmittedCompositeRuntimeWorldBasis,
        results: &CompositeOwnerExecutionResults,
    ) -> bool {
        self.component_evidence
            .matches_owner_results(results, predecessor, &self.basis)
    }
}

fn require_owner(
    expected: crate::identity::RuntimeWorldOwnerIdentity,
    actual: crate::identity::RuntimeWorldOwnerIdentity,
) -> Result<(), CompositeCommitConstructionDenial> {
    if expected == actual {
        Ok(())
    } else {
        Err(CompositeCommitConstructionDenial::OwnerMismatch)
    }
}
