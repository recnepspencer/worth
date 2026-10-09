use worth_signal::facade::ResourceManagedQueueBinding;

use super::BridgeManagedExecutionStepContract;

/// Execution capabilities carried by an admitted Bridge request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeExecutionPosture {
    Atomic,
    Managed,
}

pub(super) enum BridgeExecutionBasisPosture {
    Atomic,
    Managed(BridgeManagedExecutionBasis),
}

pub(super) struct BridgeManagedExecutionBasis {
    pub step_contract: BridgeManagedExecutionStepContract,
    pub queue: ResourceManagedQueueBinding,
    pub occupancy_width: u64,
}

impl BridgeExecutionBasisPosture {
    pub fn managed(&self) -> Option<&BridgeManagedExecutionBasis> {
        match self {
            Self::Atomic => None,
            Self::Managed(managed) => Some(managed),
        }
    }

    pub fn managed_mut(&mut self) -> Option<&mut BridgeManagedExecutionBasis> {
        match self {
            Self::Atomic => None,
            Self::Managed(managed) => Some(managed),
        }
    }
}
