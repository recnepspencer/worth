//! Memory and authored owners keep their original roots through strong and weak handoffs.
use super::conditional_execution::ConditionalExecutionOwner;
use std::ops::Deref;
use std::sync::{Arc, LockResult, PoisonError, RwLock, Weak};
use worth_runtime_bridge::facade::{BridgeReadmittedAuthoredAssembly, BridgeSealedRuntimeAssembly};

trait MemoryConditionalOwnerBacking:
    Send + Sync + std::panic::UnwindSafe + std::panic::RefUnwindSafe
{
    fn as_lock(&self) -> &RwLock<BridgeSealedRuntimeAssembly>;
}

impl MemoryConditionalOwnerBacking for RwLock<BridgeSealedRuntimeAssembly> {
    fn as_lock(&self) -> &RwLock<BridgeSealedRuntimeAssembly> {
        self
    }
}

#[derive(Clone)]
pub struct SharedMemoryConditionalOwner(Arc<dyn MemoryConditionalOwnerBacking>);

impl SharedMemoryConditionalOwner {
    pub fn from_arc(owner: Arc<RwLock<BridgeSealedRuntimeAssembly>>) -> Self {
        Self(owner)
    }

    pub fn downgrade(&self) -> WeakMemoryConditionalOwner {
        WeakMemoryConditionalOwner(Arc::downgrade(&self.0))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Deref for SharedMemoryConditionalOwner {
    type Target = RwLock<BridgeSealedRuntimeAssembly>;

    fn deref(&self) -> &Self::Target {
        self.0.as_lock()
    }
}

#[derive(Clone)]
pub struct WeakMemoryConditionalOwner(Weak<dyn MemoryConditionalOwnerBacking>);

impl WeakMemoryConditionalOwner {
    pub fn upgrade(&self) -> Option<SharedMemoryConditionalOwner> {
        self.0.upgrade().map(SharedMemoryConditionalOwner)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

trait AuthoredConditionalOwnerBacking: Send + Sync {
    fn as_authored(&self) -> &BridgeReadmittedAuthoredAssembly;
}

impl AuthoredConditionalOwnerBacking for BridgeReadmittedAuthoredAssembly {
    fn as_authored(&self) -> &BridgeReadmittedAuthoredAssembly {
        self
    }
}

#[derive(Clone)]
pub struct SharedAuthoredConditionalOwner(Arc<dyn AuthoredConditionalOwnerBacking>);

impl SharedAuthoredConditionalOwner {
    pub fn from_arc(owner: Arc<BridgeReadmittedAuthoredAssembly>) -> Self {
        Self(owner)
    }

    pub fn downgrade(&self) -> WeakAuthoredConditionalOwner {
        WeakAuthoredConditionalOwner(Arc::downgrade(&self.0))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Deref for SharedAuthoredConditionalOwner {
    type Target = BridgeReadmittedAuthoredAssembly;

    fn deref(&self) -> &Self::Target {
        self.0.as_authored()
    }
}

#[derive(Clone)]
pub struct WeakAuthoredConditionalOwner(Weak<dyn AuthoredConditionalOwnerBacking>);

impl WeakAuthoredConditionalOwner {
    pub fn upgrade(&self) -> Option<SharedAuthoredConditionalOwner> {
        self.0.upgrade().map(SharedAuthoredConditionalOwner)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone)]
pub enum ConditionalOwner {
    Memory(SharedMemoryConditionalOwner),
    Authored(SharedAuthoredConditionalOwner),
    Stopped,
}
#[derive(Clone)]
pub enum ConditionalOwnerObservation {
    Memory(WeakMemoryConditionalOwner),
    Authored(WeakAuthoredConditionalOwner),
    Stopped,
}
impl ConditionalOwner {
    pub fn memory(owner: Arc<RwLock<BridgeSealedRuntimeAssembly>>) -> Self {
        Self::Memory(SharedMemoryConditionalOwner::from_arc(owner))
    }

    pub fn authored(owner: Arc<BridgeReadmittedAuthoredAssembly>) -> Self {
        Self::Authored(SharedAuthoredConditionalOwner::from_arc(owner))
    }

    pub fn downgrade(&self) -> ConditionalOwnerObservation {
        match self {
            Self::Memory(owner) => ConditionalOwnerObservation::Memory(owner.downgrade()),
            Self::Authored(owner) => ConditionalOwnerObservation::Authored(owner.downgrade()),
            Self::Stopped => ConditionalOwnerObservation::Stopped,
        }
    }
    pub fn shares_original(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Memory(left), Self::Memory(right)) => left.ptr_eq(right),
            (Self::Authored(left), Self::Authored(right)) => left.ptr_eq(right),
            (Self::Memory(_), Self::Authored(_))
            | (Self::Authored(_), Self::Memory(_))
            | (Self::Stopped, Self::Memory(_))
            | (Self::Stopped, Self::Authored(_))
            | (Self::Memory(_), Self::Stopped)
            | (Self::Authored(_), Self::Stopped)
            | (Self::Stopped, Self::Stopped) => false,
        }
    }
    pub fn read(&self) -> LockResult<ConditionalExecutionOwner<'_>> {
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
    pub fn shares_original(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Memory(left), Self::Memory(right)) => left.ptr_eq(right),
            (Self::Authored(left), Self::Authored(right)) => left.ptr_eq(right),
            (Self::Stopped, Self::Stopped) => true,
            (Self::Memory(_), Self::Authored(_))
            | (Self::Authored(_), Self::Memory(_))
            | (Self::Stopped, Self::Memory(_))
            | (Self::Stopped, Self::Authored(_))
            | (Self::Memory(_), Self::Stopped)
            | (Self::Authored(_), Self::Stopped) => false,
        }
    }

    pub fn upgrade(&self) -> Option<ConditionalOwner> {
        match self {
            Self::Memory(owner) => owner.upgrade().map(ConditionalOwner::Memory),
            Self::Authored(owner) => owner.upgrade().map(ConditionalOwner::Authored),
            Self::Stopped => None,
        }
    }
}
