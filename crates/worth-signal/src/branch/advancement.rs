use worth_foundational::FoundationalBranchReferenceMismatchAxis;

use crate::data::error::SignalError;
use crate::logic::transaction::TransactionResult;
use crate::state::SignalBranchId;

use super::{
    AdmittedSignalBranchBasis, SignalBranchRetentionAcquisitionDenial, SignalOwnerUnavailable,
};

/// Owner-issued result of one canonical Signal branch mutation.
#[derive(Debug)]
pub struct SignalBranchAdvanceOutcome {
    advanced_basis: AdmittedSignalBranchBasis,
    transaction: TransactionResult,
    conditional_definition:
        Option<super::owner_services::SignalConditionalDefinitionAdvanceBinding>,
}

impl SignalBranchAdvanceOutcome {
    pub(crate) fn owner_issued(
        advanced_basis: AdmittedSignalBranchBasis,
        transaction: TransactionResult,
        conditional_definition: Option<
            super::owner_services::SignalConditionalDefinitionAdvanceBinding,
        >,
    ) -> Self {
        Self {
            advanced_basis,
            transaction,
            conditional_definition,
        }
    }

    pub fn advanced_basis(&self) -> &AdmittedSignalBranchBasis {
        &self.advanced_basis
    }

    pub fn transaction(&self) -> &TransactionResult {
        &self.transaction
    }

    pub fn into_parts(self) -> (AdmittedSignalBranchBasis, TransactionResult) {
        (self.advanced_basis, self.transaction)
    }

    pub fn take_conditional_definition_advance_binding(
        &mut self,
    ) -> Option<super::owner_services::SignalConditionalDefinitionAdvanceBinding> {
        self.conditional_definition.take()
    }

    pub fn into_basis(self) -> AdmittedSignalBranchBasis {
        self.advanced_basis
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalBranchAdvanceEngineDenial {
    UnknownTargetBranch {
        branch_id: SignalBranchId,
    },
    ActiveBranchTarget {
        branch_id: SignalBranchId,
    },
    CrossBranchHead {
        target_branch_id: SignalBranchId,
        head_branch_id: SignalBranchId,
    },
    StaleTargetHead,
    CanonicalBasisMismatch,
}

#[derive(Debug)]
pub enum SignalBranchAdvanceDenial {
    OwnerUnavailable(SignalOwnerUnavailable),
    OperationCapacityExhausted {
        maximum_in_flight_operations: usize,
    },
    OwnerReentry,
    CancelledNoMovement,
    UnknownBranch {
        branch_id: SignalBranchId,
    },
    RetirementInProgress {
        branch_id: SignalBranchId,
    },
    RetiredBranch {
        branch_id: SignalBranchId,
    },
    QuarantinedBranch {
        branch_id: SignalBranchId,
    },
    OwnerCellMisuse {
        branch_id: SignalBranchId,
    },
    BasisMismatch {
        axes: Vec<FoundationalBranchReferenceMismatchAxis>,
    },
    ConditionalDefinitionPublicationMismatch,
    RetentionUnavailable {
        denial: SignalBranchRetentionAcquisitionDenial,
    },
    MutationDeniedNoMovement {
        denial: SignalBranchAdvanceEngineDenial,
    },
    MutationFailedNoMovement {
        error: SignalError,
    },
}
