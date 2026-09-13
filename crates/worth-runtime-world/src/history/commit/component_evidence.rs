use worth_relational::facade::history::RelationalCommitIdentity;

use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::publication::CompositeOwnerExecutionResults;

use super::{
    CompositeCommitConstructionDenial, CompositeComponentChangePosture, CompositeComponentEvidence,
    CompositeSignalPublicationIdentity, RelationalComponentEvidence, SignalComponentEvidence,
};

impl CompositeComponentEvidence {
    pub(super) fn retained(basis: &AdmittedCompositeRuntimeWorldBasis) -> Self {
        Self {
            relational: RelationalComponentEvidence::RetainedExact {
                basis: basis.relational_basis().admission_identity().clone(),
            },
            signal: SignalComponentEvidence::RetainedExact {
                basis: basis.signal_basis().admission_identity().clone(),
            },
        }
    }

    pub(super) fn from_owner_results(
        results: &CompositeOwnerExecutionResults,
        predecessor: &AdmittedCompositeRuntimeWorldBasis,
        successor: &AdmittedCompositeRuntimeWorldBasis,
    ) -> Result<Self, CompositeCommitConstructionDenial> {
        let relational = if let Some(target) = results.relational_fork_target_identity() {
            let basis = results
                .relational_publication_basis_identity()
                .expect("a forked Relational result carries its resulting basis");
            if basis != successor.relational_basis().admission_identity() {
                return Err(CompositeCommitConstructionDenial::BasisMismatch);
            }
            RelationalComponentEvidence::Forked {
                target: target.clone(),
                basis: basis.clone(),
            }
        } else if let Some(commit) = results.relational_publication_identity() {
            let basis = results
                .relational_publication_basis_identity()
                .expect("a published Relational result carries its resulting basis");
            if basis != successor.relational_basis().admission_identity() {
                return Err(CompositeCommitConstructionDenial::BasisMismatch);
            }
            RelationalComponentEvidence::Published {
                commit,
                basis: basis.clone(),
            }
        } else {
            if predecessor.relational_basis().admission_identity()
                != successor.relational_basis().admission_identity()
            {
                return Err(CompositeCommitConstructionDenial::BasisMismatch);
            }
            RelationalComponentEvidence::RetainedExact {
                basis: successor.relational_basis().admission_identity().clone(),
            }
        };

        let signal = if let Some(publication) = results.signal_publication_identity() {
            if publication.basis_identity() != successor.signal_basis().admission_identity() {
                return Err(CompositeCommitConstructionDenial::BasisMismatch);
            }
            SignalComponentEvidence::Published(publication)
        } else {
            if predecessor.signal_basis().admission_identity()
                != successor.signal_basis().admission_identity()
            {
                return Err(CompositeCommitConstructionDenial::BasisMismatch);
            }
            SignalComponentEvidence::RetainedExact {
                basis: successor.signal_basis().admission_identity().clone(),
            }
        };

        Ok(Self { relational, signal })
    }

    pub(super) fn matches_owner_results(
        &self,
        results: &CompositeOwnerExecutionResults,
        predecessor: &AdmittedCompositeRuntimeWorldBasis,
        successor: &AdmittedCompositeRuntimeWorldBasis,
    ) -> bool {
        Self::from_owner_results(results, predecessor, successor)
            .is_ok_and(|evidence| evidence == *self)
    }

    pub(crate) fn relational_posture(&self) -> CompositeComponentChangePosture {
        match self.relational {
            RelationalComponentEvidence::RetainedExact { .. } => {
                CompositeComponentChangePosture::RetainExact
            }
            RelationalComponentEvidence::Published { .. }
            | RelationalComponentEvidence::Forked { .. } => {
                CompositeComponentChangePosture::Published
            }
        }
    }

    pub(crate) fn signal_posture(&self) -> CompositeComponentChangePosture {
        match self.signal {
            SignalComponentEvidence::RetainedExact { .. } => {
                CompositeComponentChangePosture::RetainExact
            }
            SignalComponentEvidence::Published(_) => CompositeComponentChangePosture::Published,
        }
    }

    pub(crate) fn relational_publication_identity(&self) -> Option<&RelationalCommitIdentity> {
        match &self.relational {
            RelationalComponentEvidence::RetainedExact { .. }
            | RelationalComponentEvidence::Forked { .. } => None,
            RelationalComponentEvidence::Published { commit, .. } => Some(commit),
        }
    }

    pub(crate) fn relational_fork_target_identity(
        &self,
    ) -> Option<&worth_relational::facade::branch::RelationalBranchIdentity> {
        match &self.relational {
            RelationalComponentEvidence::Forked { target, .. } => Some(target),
            RelationalComponentEvidence::RetainedExact { .. }
            | RelationalComponentEvidence::Published { .. } => None,
        }
    }

    pub(crate) fn signal_publication_identity(
        &self,
    ) -> Option<&CompositeSignalPublicationIdentity> {
        match &self.signal {
            SignalComponentEvidence::RetainedExact { .. } => None,
            SignalComponentEvidence::Published(identity) => Some(identity),
        }
    }
}
