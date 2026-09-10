use std::sync::{Arc, Weak};

use worth_proof::AuthorityWitness;

use crate::branch::owner_services::basis_port::denial_mapping::map_observation_admission_denial;
use crate::branch::owner_services::owner::basis::map_basis_registry_denial;
use crate::branch::owner_services::{SignalOwner, SignalOwnerUnavailable};
use crate::branch::{AdmittedSignalBranchBasis, SignalBranchBasisObservationDenial};
use crate::data::aspect::SignalAspectLoweringOwner;

use super::SignalConditionalExecutionPort;

worth_proof::authority_marker!(pub(in crate::branch::owner_services) SignalConditionalServiceAuthorityMarker);
pub(in crate::branch::owner_services) type SignalConditionalServiceAuthority =
    AuthorityWitness<SignalConditionalServiceAuthorityMarker>;

#[derive(Debug)]
pub enum SignalConditionalServiceIssuanceDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    OwnerAdmission(SignalBranchBasisObservationDenial),
    ForeignBasis,
    BasisMismatch {
        axes: Vec<worth_foundational::FoundationalBranchReferenceMismatchAxis>,
    },
    StaleBasisAdmission,
    DefinitionReadmissionRequired,
    ClaimantMismatch,
    CaptureCapacityExhausted,
    CaptureWorkExhausted {
        maximum_visits: usize,
    },
    CaptureUnavailable,
}

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Reissue this claimant's service at a successor basis after an owner
    /// definition publication. The current owner cell is re-admitted, so this
    /// may run before product publication and cannot leave a performed product
    /// head without executable definition custody.
    pub fn reissue_for_successor_basis(
        &self,
        basis: &AdmittedSignalBranchBasis,
    ) -> Result<Self, SignalConditionalServiceIssuanceDenial> {
        Self::issue(
            self.owner.clone(),
            basis,
            &self.claimant,
            &self.source_authority,
        )
    }

    pub(crate) fn issue(
        weak: Weak<SignalOwner<D, I, T>>,
        basis: &AdmittedSignalBranchBasis,
        claimant: &SignalAspectLoweringOwner,
        source_authority: &worth_proof::ConditionalSourceObservationAuthority,
    ) -> Result<Self, SignalConditionalServiceIssuanceDenial> {
        use SignalConditionalServiceIssuanceDenial as Denial;
        let owner = SignalOwner::upgrade(&weak).map_err(Denial::OwnerUnavailable)?;
        if !owner.basis_has_owner_affinity(basis) {
            return Err(Denial::ForeignBasis);
        }
        let admission = owner
            .admit()
            .map_err(map_observation_admission_denial)
            .map_err(Denial::OwnerAdmission)?;
        let branch = basis.owner_branch_id();
        let cell = owner
            .lookup_cell(&admission, branch)
            .map_err(|denial| Denial::OwnerAdmission(map_basis_registry_denial(denial, branch)))?;
        let (definition, observation, issuance_basis_custody) = cell.admit_conditional_definition(
            &admission,
            basis,
            claimant,
            &owner.conditional_retention,
        )?;
        if !owner.is_current_canonical_basis(basis, branch, cell.incarnation().get(), &observation)
        {
            return Err(Denial::StaleBasisAdmission);
        }
        Ok(Self {
            owner: Arc::downgrade(&owner),
            basis: basis.clone(),
            definition,
            incarnation: cell.incarnation(),
            authority: Arc::new(SignalConditionalServiceAuthorityMarker::witness()),
            source_authority: source_authority.clone(),
            claimant: claimant.clone(),
            _issuance_basis_custody: issuance_basis_custody,
            next_evaluation_ordinal: std::sync::atomic::AtomicU64::new(0),
            next_installation_ordinal: std::sync::atomic::AtomicU64::new(0),
        })
    }
}
