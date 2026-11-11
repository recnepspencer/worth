use super::{SignalOwner, SignalOwnerPartition};
use crate::branch::owner_services::SignalOwnerUnavailable;
use crate::branch::SignalBranchBasisRegistry;
use std::sync::{Arc, Weak};

/// Non-cloneable root field. Before sealing it contains no competing owner.
pub(crate) struct SignalOwnerRoot<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) state: SignalOwnerRootState<D, I, T>,
    definition_publication_issued: bool,
}

pub(super) enum SignalOwnerRootState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    Unsealed {
        runtime_instance_id: u64,
        definition_basis: u64,
        basis_registry: SignalBranchBasisRegistry,
    },
    Sealed(Arc<SignalOwner<D, I, T>>),
    Transferred,
}

impl<D, I, T> SignalOwnerRoot<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn new(
        runtime_instance_id: u64,
        definition_basis: u64,
        basis_registry: SignalBranchBasisRegistry,
    ) -> Self {
        Self {
            state: SignalOwnerRootState::Unsealed {
                runtime_instance_id,
                definition_basis,
                basis_registry,
            },
            definition_publication_issued: false,
        }
    }

    pub(crate) fn is_sealed(&self) -> bool {
        matches!(self.state, SignalOwnerRootState::Sealed(_))
    }

    pub(crate) fn seal(
        &mut self,
        partition: SignalOwnerPartition<D, I, T>,
        schema_commitment: String,
        conditional_budget: crate::runtime_policy::SignalConditionalEvaluationBudget,
        temporal_budget: crate::runtime_policy::SignalConditionalTemporalBudget,
    ) {
        let (runtime_instance_id, definition_basis, basis_registry) = match &self.state {
            SignalOwnerRootState::Unsealed {
                runtime_instance_id,
                definition_basis,
                basis_registry,
            } => (
                *runtime_instance_id,
                *definition_basis,
                basis_registry.clone(),
            ),
            SignalOwnerRootState::Sealed(_) | SignalOwnerRootState::Transferred => {
                panic!("Signal owner root cannot consume a second canonical partition")
            }
        };
        let owner = SignalOwner::from_partition(
            runtime_instance_id,
            definition_basis,
            schema_commitment,
            partition,
            basis_registry,
            conditional_budget,
            temporal_budget,
        );
        self.state = SignalOwnerRootState::Sealed(owner);
    }

    pub(crate) fn downgrade_owner(
        &self,
    ) -> Result<Weak<SignalOwner<D, I, T>>, SignalOwnerUnavailable> {
        match &self.state {
            SignalOwnerRootState::Unsealed { .. } | SignalOwnerRootState::Transferred => {
                Err(SignalOwnerUnavailable)
            }
            SignalOwnerRootState::Sealed(owner) => Ok(Arc::downgrade(owner)),
        }
    }

    pub(crate) fn transfer_to_durable_owner(
        &mut self,
    ) -> Option<super::SignalSealedOwnerRoot<D, I, T>> {
        if !matches!(self.state, SignalOwnerRootState::Sealed(_)) {
            return None;
        }
        match std::mem::replace(&mut self.state, SignalOwnerRootState::Transferred) {
            SignalOwnerRootState::Sealed(owner) => Some(super::SignalSealedOwnerRoot { owner }),
            SignalOwnerRootState::Unsealed { .. } | SignalOwnerRootState::Transferred => {
                unreachable!("same original was sealed")
            }
        }
    }
    pub(crate) fn claim_definition_publication(&mut self) -> bool {
        if self.definition_publication_issued {
            return false;
        }
        self.definition_publication_issued = true;
        true
    }

    pub(super) fn sealed_owner(&self) -> Option<Arc<SignalOwner<D, I, T>>> {
        match &self.state {
            SignalOwnerRootState::Unsealed { .. } | SignalOwnerRootState::Transferred => None,
            SignalOwnerRootState::Sealed(owner) => Some(Arc::clone(owner)),
        }
    }

    pub(crate) fn operation_control_for_test(
        &self,
    ) -> Result<
        crate::branch::owner_services::operation_control::SignalOwnerOperationControlForTest,
        SignalOwnerUnavailable,
    > {
        match &self.state {
            SignalOwnerRootState::Unsealed { .. } | SignalOwnerRootState::Transferred => {
                Err(SignalOwnerUnavailable)
            }
            SignalOwnerRootState::Sealed(owner) => Ok(owner.operation_control_for_test()),
        }
    }
}

impl<D, I, T> Drop for SignalOwnerRoot<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn drop(&mut self) {
        if let SignalOwnerRootState::Sealed(owner) = &self.state {
            let _ = owner.request_close();
        }
    }
}
