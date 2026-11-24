//! Type-erased shared handle to output-demand registry state.
//!
//! Uses unsized coercion on the existing `Arc<Mutex<DemandRegistryState>>`
//! allocation so rustdoc auto-trait synthesis does not traverse demand records
//! and lineage cells through runtime handles.

use std::ops::Deref;
use std::sync::{Arc, Mutex, Weak};

use super::DemandRegistryState;

trait DemandRegistryStateBacking:
    Send + Sync + std::panic::UnwindSafe + std::panic::RefUnwindSafe
{
    fn as_mutex(&self) -> &Mutex<DemandRegistryState>;
    fn as_mutex_mut(&mut self) -> &mut Mutex<DemandRegistryState>;
}

impl DemandRegistryStateBacking for Mutex<DemandRegistryState> {
    fn as_mutex(&self) -> &Mutex<DemandRegistryState> {
        self
    }

    fn as_mutex_mut(&mut self) -> &mut Mutex<DemandRegistryState> {
        self
    }
}

#[derive(Clone)]
pub struct SharedDemandRegistryState(Arc<dyn DemandRegistryStateBacking>);

#[derive(Clone)]
pub struct WeakDemandRegistryState(Weak<dyn DemandRegistryStateBacking>);

impl Default for SharedDemandRegistryState {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(DemandRegistryState::default())))
    }
}

impl SharedDemandRegistryState {
    pub fn ptr_eq(&self, other: &Self) -> bool {
        std::ptr::eq(&**self, &**other)
    }

    pub fn get_mut(&mut self) -> Option<&mut Mutex<DemandRegistryState>> {
        Arc::get_mut(&mut self.0).map(DemandRegistryStateBacking::as_mutex_mut)
    }

    pub fn downgrade(&self) -> WeakDemandRegistryState {
        WeakDemandRegistryState(Arc::downgrade(&self.0))
    }
}

impl WeakDemandRegistryState {
    pub fn upgrade(&self) -> Option<SharedDemandRegistryState> {
        self.0.upgrade().map(SharedDemandRegistryState)
    }
}

impl Deref for SharedDemandRegistryState {
    type Target = Mutex<DemandRegistryState>;

    fn deref(&self) -> &Self::Target {
        DemandRegistryStateBacking::as_mutex(&*self.0)
    }
}
