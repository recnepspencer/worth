//! Actual Native companion images expose their own final destruction posture.
use super::*;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Mutex,
};
use worth_relational::facade::mvcc::{
    CompanionBranchCellSlot, CompanionPreflightStop, PreparedPublicationCompanionEffect,
    PublicationCompanionPreflight, RelationalPublicationCompanion,
};

#[derive(Debug, Default)]
pub(super) struct Probe {
    pub active: AtomicBool,
    pub prepares: AtomicUsize,
    pub dropped: Mutex<Vec<(usize, bool)>>,
}

pub(super) struct Active<'a>(&'a Probe);
impl Probe {
    pub fn enter(&self) -> Active<'_> {
        assert!(!self.active.swap(true, Ordering::SeqCst));
        Active(self)
    }
    pub fn events(&self) -> Vec<(usize, bool)> {
        self.dropped.lock().unwrap().clone()
    }
}
impl Drop for Active<'_> {
    fn drop(&mut self) {
        self.0.active.store(false, Ordering::SeqCst);
    }
}

#[derive(Debug)]
struct Payload {
    id: usize,
    probe: Arc<Probe>,
}
impl Drop for Payload {
    fn drop(&mut self) {
        self.probe
            .dropped
            .lock()
            .unwrap()
            .push((self.id, self.probe.active.load(Ordering::SeqCst)));
    }
}

struct Companion {
    cell: Mutex<Option<CompanionBranchCellSlot<Payload>>>,
    probe: Arc<Probe>,
}
impl std::fmt::Debug for Companion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Companion").finish_non_exhaustive()
    }
}
impl RelationalPublicationCompanion for Companion {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        assert!(
            !self.probe.active.load(Ordering::SeqCst),
            "user preparation ran inside the cutover handoff"
        );
        self.probe.prepares.fetch_add(1, Ordering::SeqCst);
        let mut slot = self.cell.lock().unwrap();
        let reserved = match slot.as_ref().and_then(CompanionBranchCellSlot::admitted) {
            Some(cell) => cell.reserve_preflight(context)?,
            None => {
                let cell = context.mint_selected_branch_cell(Arc::new(Payload {
                    id: 0,
                    probe: Arc::clone(&self.probe),
                }))?;
                *slot = Some(cell.lookup_slot());
                cell.reserve_preflight(context)?
            }
        };
        context.seal_replacement(
            reserved,
            Arc::new(Payload {
                id: 1,
                probe: Arc::clone(&self.probe),
            }),
        )
    }
}

pub(super) fn register(runtime: &RelationalRuntime, probe: Arc<Probe>) -> impl Sized + use<> {
    runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(Arc::new(Companion {
            cell: Mutex::new(None),
            probe,
        }))
        .unwrap()
}
