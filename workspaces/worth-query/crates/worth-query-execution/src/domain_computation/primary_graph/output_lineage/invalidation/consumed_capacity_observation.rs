//! Passive weak observation of the actual consumed-edge ledger tickets.
use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, OnceLock, Weak},
};
type Tickets = BTreeMap<u64, Vec<Weak<RetainedInvalidationCapacity>>>;
fn tickets() -> &'static Mutex<Tickets> {
    static TICKETS: OnceLock<Mutex<Tickets>> = OnceLock::new();
    TICKETS.get_or_init(Default::default)
}
pub(super) fn observe(runtime: u64, ticket: &Arc<RetainedInvalidationCapacity>) {
    if let Some(tickets) = tickets().lock().unwrap().get_mut(&runtime) {
        tickets.push(Arc::downgrade(ticket));
    }
}
fn live(runtime: u64) -> (usize, u64) {
    let mut all = tickets().lock().unwrap();
    let tickets = all.entry(runtime).or_default();
    let mut count = 0;
    let mut bytes = 0;
    tickets.retain(|ticket| {
        if let Some(ticket) = ticket.upgrade() {
            count += 1;
            bytes += ticket.bytes();
            true
        } else {
            false
        }
    });
    (count, bytes)
}
impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// Actual consumed-edge reservations on the invalidation ledger. Reading
    /// enables passive weak observation; it never retains an evidence ticket.
    #[doc(hidden)]
    pub fn consumed_output_custody_for_test(&self) -> (usize, u64) {
        live(
            self.product_runtime
                .source
                .authoritative_source_profile()
                .runtime_instance_id(),
        )
    }
}
