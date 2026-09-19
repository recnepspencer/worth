use std::sync::Arc;

use crate::branch::owner_services::basis_port::denial_mapping::map_observation_admission_denial;
use crate::branch::owner_services::owner::basis::map_basis_registry_denial;
use crate::branch::owner_services::{
    SignalBranchCellIncarnation, SignalOwner, SignalOwnerUnavailable,
};
use crate::branch::{AdmittedSignalBranchBasis, SignalBranchBasisObservationDenial};
use crate::data::conditional_execution::{
    InstalledSignalConditionalContract, PreparedSignalConditionalContract,
    SignalConditionalContractDefinition,
};
use crate::data::handle::NodeId;
use crate::data::retained_storage::RetainedStoragePreparation;
use crate::data::retained_storage::SignalConditionalRetentionReservation;
use crate::logic::transaction::SignalTransaction;

use super::{
    SignalConditionalDefinitionPublicationScope, SignalConditionalExecutionPort,
    SignalConditionalServiceAuthority, SignalInstalledDefinitionBinding,
};

mod applied_binding;
mod port;
pub use applied_binding::SignalConditionalDefinitionAdvanceBinding;
pub(in crate::branch::owner_services) use applied_binding::SignalConditionalDefinitionAdvanceMint;

/// One preflighted Signal definition change and its publication operation.
#[must_use = "a prepared installation must be published or discarded"]
pub struct SignalPreparedConditionalInstallationExtension {
    operation: SignalConditionalDefinitionPublicationOperation,
    request: SignalConditionalInstallationExtensionRequest,
}

impl SignalPreparedConditionalInstallationExtension {
    pub fn into_parts(
        self,
    ) -> (
        SignalConditionalDefinitionPublicationOperation,
        SignalConditionalInstallationExtensionRequest,
    ) {
        (self.operation, self.request)
    }
}

/// Move-only authority consumed by Runtime World's exact Signal advance.
pub struct SignalConditionalDefinitionPublicationOperation {
    predecessor: AdmittedSignalBranchBasis,
    incarnation: SignalBranchCellIncarnation,
    scope: SignalConditionalDefinitionPublicationScope,
}

impl SignalConditionalDefinitionPublicationOperation {
    pub(in crate::branch::owner_services) fn into_advance_parts(
        self,
    ) -> (
        SignalConditionalDefinitionPublicationScope,
        SignalConditionalDefinitionAdvanceMint,
    ) {
        let mint = SignalConditionalDefinitionAdvanceMint::new(
            Arc::clone(&self.scope.authority),
            self.scope.clone(),
            self.predecessor,
            self.incarnation,
        );
        (self.scope, mint)
    }

    pub(in crate::branch::owner_services) fn admits_predecessor(
        &self,
        candidate: &AdmittedSignalBranchBasis,
    ) -> bool {
        self.predecessor.admission_identity() == candidate.admission_identity()
            && self.predecessor.observation() == candidate.observation()
    }

    pub(in crate::branch::owner_services) const fn incarnation(
        &self,
    ) -> SignalBranchCellIncarnation {
        self.incarnation
    }
}

/// Opaque request applied only in the transaction carrying its paired scope.
#[must_use = "a prepared installation must be applied or discarded"]
pub struct SignalConditionalInstallationExtensionRequest {
    prepared: SignalPreparedInstallationTarget,
    service_authority: Arc<SignalConditionalServiceAuthority>,
    definition_binding: SignalInstalledDefinitionBinding,
    publication_scope: SignalConditionalDefinitionPublicationScope,
    predecessor: AdmittedSignalBranchBasis,
    incarnation: SignalBranchCellIncarnation,
    custody: SignalConditionalInstallationCustody,
}

pub(in crate::branch::owner_services) enum SignalConditionalInstallationTarget {
    Existing {
        node: NodeId,
        expected_contract_generation: u64,
    },
    Allocate,
}

pub(in crate::branch::owner_services) enum SignalPreparedInstallationTarget {
    Existing(PreparedSignalConditionalContract),
    Allocate(SignalConditionalContractDefinition),
}

#[derive(Debug)]
pub enum SignalConditionalInstallationExtensionDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    OwnerAdmission(SignalBranchBasisObservationDenial),
    ForeignBasis,
    StaleBasis {
        axes: Vec<worth_foundational::FoundationalBranchReferenceMismatchAxis>,
    },
    StaleCell,
    DefinitionMismatch,
    TargetGenerationMismatch {
        expected: u64,
        observed: u64,
    },
    TargetUnavailable,
    CapacityExhausted,
    RetentionUnavailable,
    WorkExhausted {
        maximum_visits: usize,
    },
    PublicationOrdinalExhausted,
    ForeignService,
    TransactionScopeMismatch,
    SuccessorBindingMismatch,
    SignalMutation(crate::data::error::SignalError),
}

pub struct SignalConditionalInstallationExtensionCompletion<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    contract: InstalledSignalConditionalContract,
    predecessor_generation: u64,
    predecessor_occurrence: u64,
    predecessor: AdmittedSignalBranchBasis,
    incarnation: SignalBranchCellIncarnation,
    service_authority: Arc<SignalConditionalServiceAuthority>,
    publication_scope: SignalConditionalDefinitionPublicationScope,
    custody: SignalConditionalInstallationCustody,
    successor_service: SignalPreparedConditionalExecutionService<D, I, T>,
}

struct SignalPreparedConditionalExecutionService<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    owner: std::sync::Weak<SignalOwner<D, I, T>>,
    definition: SignalInstalledDefinitionBinding,
    incarnation: SignalBranchCellIncarnation,
    authority: Arc<SignalConditionalServiceAuthority>,
    source_authority: worth_proof::ConditionalSourceObservationAuthority,
    claimant: crate::data::aspect::SignalAspectLoweringOwner,
    issuance_basis_custody: Arc<super::SignalRetainedExecutionBasis>,
}

/// Shared custody retained for exactly as long as an installed or unpublished
/// definition version remains reachable.
#[derive(Clone)]
pub struct SignalConditionalInstallationCustody {
    _reservation: Arc<SignalConditionalRetentionReservation>,
}

impl SignalConditionalInstallationCustody {
    pub(in crate::branch::owner_services) fn new(
        reservation: SignalConditionalRetentionReservation,
    ) -> Self {
        Self {
            _reservation: Arc::new(reservation),
        }
    }
}

impl<D, I, T> SignalConditionalInstallationExtensionCompletion<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn contract(&self) -> &InstalledSignalConditionalContract {
        &self.contract
    }

    pub const fn predecessor_generation(&self) -> u64 {
        self.predecessor_generation
    }

    pub const fn predecessor_occurrence(&self) -> u64 {
        self.predecessor_occurrence
    }

    pub fn predecessor_basis(&self) -> &AdmittedSignalBranchBasis {
        &self.predecessor
    }

    pub fn cell_incarnation(&self) -> u64 {
        self.incarnation.get()
    }

    pub fn into_retained_parts(
        self,
        binding: SignalConditionalDefinitionAdvanceBinding,
    ) -> Result<
        (
            InstalledSignalConditionalContract,
            SignalConditionalInstallationCustody,
            SignalConditionalExecutionPort<D, I, T>,
        ),
        SignalConditionalInstallationExtensionDenial,
    > {
        let successor_basis = binding
            .into_successor_if_matches(
                &self.service_authority,
                &self.publication_scope,
                &self.predecessor,
                self.incarnation,
            )
            .ok_or(SignalConditionalInstallationExtensionDenial::SuccessorBindingMismatch)?;
        let service = self.successor_service.complete(successor_basis);
        Ok((self.contract, self.custody, service))
    }
}

impl<D, I, T> SignalPreparedConditionalExecutionService<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn complete(self, basis: AdmittedSignalBranchBasis) -> SignalConditionalExecutionPort<D, I, T> {
        SignalConditionalExecutionPort {
            owner: self.owner,
            basis,
            definition: self.definition,
            incarnation: self.incarnation,
            authority: self.authority,
            source_authority: self.source_authority,
            claimant: self.claimant,
            _issuance_basis_custody: self.issuance_basis_custody,
            next_evaluation_ordinal: std::sync::atomic::AtomicU64::new(0),
            next_installation_ordinal: std::sync::atomic::AtomicU64::new(0),
        }
    }
}
