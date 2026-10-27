// Frontier writers compute both checked ends before replacing either scalar; unwinding cannot publish a partial pair.
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy)]
struct LineageIdentityFrontier {
    next_lineage_id: u64,
    next_event_id: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct LineageIdentityAllocator {
    frontier: Arc<Mutex<LineageIdentityFrontier>>,
}

impl LineageIdentityAllocator {
    pub(crate) fn new() -> Self {
        Self {
            frontier: Arc::new(Mutex::new(LineageIdentityFrontier {
                next_lineage_id: 1,
                next_event_id: 1,
            })),
        }
    }

    pub(crate) fn detached(&self) -> Self {
        let frontier = *self
            .frontier
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        Self {
            frontier: Arc::new(Mutex::new(frontier)),
        }
    }

    pub(crate) fn reserve(
        &self,
        lineage_width: u64,
        event_width: u64,
    ) -> Result<(u64, u64), String> {
        let mut frontier = self
            .frontier
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let lineage_end = checked_end(frontier.next_lineage_id, lineage_width, "lineage id")?;
        let event_end = checked_end(frontier.next_event_id, event_width, "lineage event id")?;
        let starts = (frontier.next_lineage_id, frontier.next_event_id);
        frontier.next_lineage_id = lineage_end;
        frontier.next_event_id = event_end;
        Ok(starts)
    }

    pub(crate) fn frontiers(&self) -> (u64, u64) {
        let frontier = self
            .frontier
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        (frontier.next_lineage_id, frontier.next_event_id)
    }

    pub(crate) fn set_frontiers(&self, next_lineage_id: u64, next_event_id: u64) {
        let mut frontier = self
            .frontier
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        frontier.next_lineage_id = next_lineage_id;
        frontier.next_event_id = next_event_id;
    }

    pub(crate) fn advance_to(&self, next_lineage_id: Option<u64>, next_event_id: Option<u64>) {
        let mut frontier = self
            .frontier
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(next_lineage_id) = next_lineage_id {
            frontier.next_lineage_id = frontier.next_lineage_id.max(next_lineage_id);
        }
        if let Some(next_event_id) = next_event_id {
            frontier.next_event_id = frontier.next_event_id.max(next_event_id);
        }
    }
}

fn checked_end(start: u64, width: u64, name: &str) -> Result<u64, String> {
    start
        .checked_add(width)
        .filter(|end| *end < u64::MAX)
        .ok_or_else(|| format!("{name} allocator exhausted"))
}
