//! Type-erased strong and weak handles to `RelationalRuntimeState`.
use super::RelationalRuntimeState;
use std::fmt;
use std::ops::Deref;
use std::sync::{Arc, Weak};

trait RelationalRuntimeStateBacking: Send + Sync {
    fn as_state(&self) -> &RelationalRuntimeState;
    fn as_state_mut(&mut self) -> &mut RelationalRuntimeState;
}
impl RelationalRuntimeStateBacking for RelationalRuntimeState {
    fn as_state(&self) -> &RelationalRuntimeState {
        self
    }
    fn as_state_mut(&mut self) -> &mut RelationalRuntimeState {
        self
    }
}

#[derive(Clone)]
pub(crate) struct StrongRelationalRuntimeState(Arc<dyn RelationalRuntimeStateBacking>);

#[derive(Clone)]
pub(crate) struct WeakRelationalRuntimeState(Weak<dyn RelationalRuntimeStateBacking>);

impl StrongRelationalRuntimeState {
    pub(crate) fn new(state: RelationalRuntimeState) -> Self {
        Self(Arc::new(state))
    }
    pub(crate) fn downgrade(&self) -> WeakRelationalRuntimeState {
        WeakRelationalRuntimeState(Arc::downgrade(&self.0))
    }
    pub(crate) fn get_mut(&mut self) -> Option<&mut RelationalRuntimeState> {
        Arc::get_mut(&mut self.0).map(RelationalRuntimeStateBacking::as_state_mut)
    }
}
impl Deref for StrongRelationalRuntimeState {
    type Target = RelationalRuntimeState;
    fn deref(&self) -> &Self::Target {
        RelationalRuntimeStateBacking::as_state(&*self.0)
    }
}
impl fmt::Debug for StrongRelationalRuntimeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}
impl WeakRelationalRuntimeState {
    pub(crate) fn upgrade(&self) -> Option<StrongRelationalRuntimeState> {
        self.0.upgrade().map(StrongRelationalRuntimeState)
    }
    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}
impl fmt::Debug for WeakRelationalRuntimeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WeakRelationalRuntimeState")
            .finish_non_exhaustive()
    }
}
