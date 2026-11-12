//! Memory and authored owners keep their original roots through strong and weak handoffs.
use super::conditional_execution::ConditionalExecutionOwner;
use std::sync::{Arc, LockResult, PoisonError, RwLock, Weak};
use worth_runtime_bridge::facade::{BridgeReadmittedAuthoredAssembly, BridgeSealedRuntimeAssembly};

#[derive(Clone)]
pub(in crate::domain_computation) enum ConditionalOwner {
    Memory(Arc<RwLock<BridgeSealedRuntimeAssembly>>),
    Authored(Arc<BridgeReadmittedAuthoredAssembly>),
    Stopped,
}
#[derive(Clone)]
pub(in crate::domain_computation) enum ConditionalOwnerObservation {
    Memory(Weak<RwLock<BridgeSealedRuntimeAssembly>>),
    Authored(Weak<BridgeReadmittedAuthoredAssembly>),
    Stopped,
}
impl ConditionalOwner {
    pub(in crate::domain_computation) fn downgrade(&self) -> ConditionalOwnerObservation {
        match self {
            Self::Memory(owner) => ConditionalOwnerObservation::Memory(Arc::downgrade(owner)),
            Self::Authored(owner) => ConditionalOwnerObservation::Authored(Arc::downgrade(owner)),
            Self::Stopped => ConditionalOwnerObservation::Stopped,
        }
    }
    pub(in crate::domain_computation) fn shares_original(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Memory(left), Self::Memory(right)) => Arc::ptr_eq(left, right),
            (Self::Authored(left), Self::Authored(right)) => Arc::ptr_eq(left, right),
            (Self::Memory(_), Self::Authored(_))
            | (Self::Authored(_), Self::Memory(_))
            | (Self::Stopped, Self::Memory(_))
            | (Self::Stopped, Self::Authored(_))
            | (Self::Memory(_), Self::Stopped)
            | (Self::Authored(_), Self::Stopped)
            | (Self::Stopped, Self::Stopped) => false,
        }
    }
    pub(in crate::domain_computation) fn read(&self) -> LockResult<ConditionalExecutionOwner<'_>> {
        match self {
            Self::Memory(owner) => match owner.read() {
                Ok(owner) => Ok(ConditionalExecutionOwner::Memory(owner)),
                Err(cause) => Err(PoisonError::new(ConditionalExecutionOwner::Memory(
                    cause.into_inner(),
                ))),
            },
            Self::Authored(owner) => Ok(ConditionalExecutionOwner::Authored(owner)),
            Self::Stopped => Ok(ConditionalExecutionOwner::Stopped),
        }
    }
}
impl ConditionalOwnerObservation {
    pub(in crate::domain_computation) fn shares_original(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Memory(left), Self::Memory(right)) => Weak::ptr_eq(left, right),
            (Self::Authored(left), Self::Authored(right)) => Weak::ptr_eq(left, right),
            (Self::Stopped, Self::Stopped) => true,
            (Self::Memory(_), Self::Authored(_))
            | (Self::Authored(_), Self::Memory(_))
            | (Self::Stopped, Self::Memory(_))
            | (Self::Stopped, Self::Authored(_))
            | (Self::Memory(_), Self::Stopped)
            | (Self::Authored(_), Self::Stopped) => false,
        }
    }

    pub(in crate::domain_computation) fn upgrade(&self) -> Option<ConditionalOwner> {
        match self {
            Self::Memory(owner) => owner.upgrade().map(ConditionalOwner::Memory),
            Self::Authored(owner) => owner.upgrade().map(ConditionalOwner::Authored),
            Self::Stopped => None,
        }
    }
}
