//! Original placement registration payloads and abandonment disposal.
use super::CountedSource;
use crate::durability::medium::RelationalDurableMemberCustody;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

// The real registration is disarmed before its original funded payload is reclaimed.
pub(super) struct ControlledCustodyOwner {
    pub(super) source_issuer: std::sync::Weak<CountedSource>,
    pub(super) active: Arc<AtomicUsize>,
    pub(super) registered: bool,
    pub(super) child: crate::durability::medium::RelationalDurableMemberRetirement,
}
impl Drop for ControlledCustodyOwner {
    fn drop(&mut self) {
        if self.registered {
            self.active.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

pub(super) struct CustodyOwner {
    pub(super) active: Arc<AtomicUsize>,
    pub(super) _source: RelationalDurableMemberCustody,
}

impl Drop for CustodyOwner {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::SeqCst);
    }
}
