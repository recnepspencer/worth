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
    state: SignalOwnerRootState<D, I, T>,
    definition_publication_issued: bool,
}

enum SignalOwnerRootState<D, I, T>
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
            SignalOwnerRootState::Sealed(_) => {
                panic!("Signal owner root cannot consume a second canonical partition")
            }
        };
        let owner = SignalOwner::from_partition(
            runtime_instance_id,
            definition_basis,
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
            SignalOwnerRootState::Unsealed { .. } => Err(SignalOwnerUnavailable),
            SignalOwnerRootState::Sealed(owner) => Ok(Arc::downgrade(owner)),
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
            SignalOwnerRootState::Unsealed { .. } => None,
            SignalOwnerRootState::Sealed(owner) => Some(Arc::clone(owner)),
        }
    }

    #[cfg(feature = "test-operation-control")]
    pub(crate) fn operation_control(
        &self,
    ) -> Result<
        crate::branch::owner_services::operation_control::SignalOwnerOperationControl,
        SignalOwnerUnavailable,
    > {
        match &self.state {
            SignalOwnerRootState::Unsealed { .. } => Err(SignalOwnerUnavailable),
            SignalOwnerRootState::Sealed(owner) => Ok(owner.operation_control()),
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
