//! Publication callback used to observe actual companion reclamation.
use crate::mvcc::publication::companion::{
    CompanionPreflightStop, PreparedPublicationCompanionEffect, PublicationCompanionPreflight,
    RelationalPublicationCompanion,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Debug)]
pub(super) struct Participant {
    pub(super) drops: Arc<AtomicUsize>,
    pub(super) prepares: Arc<AtomicUsize>,
}
impl Drop for Participant {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
impl RelationalPublicationCompanion for Participant {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        self.prepares.fetch_add(1, Ordering::SeqCst);
        let cell = context.mint_selected_branch_cell(Arc::new(0_u64))?;
        let reserved = cell.reserve_preflight(context)?;
        context.seal_replacement(reserved, Arc::new(1_u64))
    }
}
