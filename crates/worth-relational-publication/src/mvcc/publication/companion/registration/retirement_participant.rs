//! Publication callback used to observe actual companion reclamation.
use crate::mvcc::publication::companion::PreparedPublicationCompanionEffect;
use crate::mvcc::publication::companion::PublicationCompanionPreflight;
use crate::mvcc::publication::companion::RelationalPublicationCompanion;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_relational_contracts::internal_port::mvcc::publication::companion::preflight_stop::CompanionPreflightStop;

#[derive(Debug)]
pub struct Participant {
    drops: Arc<AtomicUsize>,
    prepares: Arc<AtomicUsize>,
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

impl Participant {
    pub fn reclamation_observer_for_test(
        drops: Arc<AtomicUsize>,
        prepares: Arc<AtomicUsize>,
    ) -> Self {
        Self { drops, prepares }
    }
}
