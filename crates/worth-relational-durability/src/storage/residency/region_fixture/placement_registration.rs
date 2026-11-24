//! Original placement registration payloads and abandonment disposal.
use super::CountedSource;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_relational_storage::internal_port::durability::medium::member_custody::RelationalDurableMemberCustody;

// The real registration is disarmed before its original funded payload is reclaimed.
pub(super) struct ControlledCustodyOwner {
    pub(super) source_issuer: std::sync::Weak<CountedSource>,
    pub(super) active: Arc<AtomicUsize>,
    pub(super) registered: bool,
    pub(super) child: worth_relational_storage::internal_port::durability::medium::member_custody::retirement::RelationalDurableMemberRetirement,
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
