//! The existing certification trace grows by one fixed page, never by history.
#[cfg(feature = "certification-test-authority")]
use super::PhysicalResidencyAllocationBoundaryEvent as Event;
#[cfg(feature = "certification-test-authority")]
use worth_execution::ExecutionMemoryReservation;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};

#[cfg(feature = "certification-test-authority")]
const PAGE_EVENTS: usize = 64;

#[cfg(all(test, feature = "certification-test-authority"))]
#[path = "trace_storage/tests.rs"]
mod tests;

#[derive(Debug, Default)]
#[cfg(feature = "certification-test-authority")]
pub(super) struct EventTrace {
    newest: Option<Box<EventPage>>,
    len: usize,
}

#[derive(Debug)]
#[cfg(feature = "certification-test-authority")]
struct EventPage {
    events: [Option<Event>; PAGE_EVENTS],
    len: usize,
    previous: Option<Box<EventPage>>,
    // Last: every event and previous-page allocation precedes its grant.
    _memory: Option<ExecutionMemoryReservation>,
}

/// Storage for one actual recorder release batch, admitted before owner gates.
pub(crate) struct PreparedTracePage {
    #[cfg(feature = "certification-test-authority")]
    page: Option<Box<EventPage>>,
}

impl PreparedTracePage {
    pub(crate) fn new(request: &ExecutionRequest<'_, '_>) -> Result<Self, WorkCeilingDenial> {
        #[cfg(feature = "certification-test-authority")]
        {
            let bytes = u64::try_from(std::alloc::Layout::new::<EventPage>().size())
                .expect("one concrete event page fits u64");
            let memory = request.reserve_retained_memory(bytes)?;
            Ok(Self {
                page: Some(EventPage::new(memory)),
            })
        }
        #[cfg(not(feature = "certification-test-authority"))]
        {
            let _ = request;
            Ok(Self {})
        }
    }
}

#[cfg(feature = "certification-test-authority")]
impl EventPage {
    fn new(memory: Option<ExecutionMemoryReservation>) -> Box<Self> {
        Box::new(Self {
            events: [None; PAGE_EVENTS],
            len: 0,
            previous: None,
            _memory: memory,
        })
    }
}

#[cfg(feature = "certification-test-authority")]
impl EventTrace {
    fn remaining(&self) -> usize {
        self.newest
            .as_ref()
            .map_or(0, |page| PAGE_EVENTS - page.len)
    }

    fn prepend(&mut self, mut page: Box<EventPage>) {
        page.previous = self.newest.take();
        self.newest = Some(page);
    }

    pub(super) fn can_extend(&self, additional: usize) -> bool {
        additional <= PAGE_EVENTS && self.len.checked_add(additional).is_some()
    }

    /// All arithmetic is checked by the release preparation before this cut.
    #[cfg(feature = "certification-test-authority")]
    pub(super) fn prepare_capacity(&mut self, additional: usize, prepared: &mut PreparedTracePage) {
        if self.remaining() < additional {
            self.prepend(
                prepared
                    .page
                    .take()
                    .expect("one original prepared event page"),
            );
        }
    }

    pub(super) fn push(&mut self, event: Event) {
        if self.remaining() == 0 {
            self.prepend(EventPage::new(None));
        }
        self.push_prepared(event);
    }

    pub(super) fn push_prepared(&mut self, event: Event) {
        let page = self.newest.as_mut().expect("trace capacity established");
        page.events[page.len] = Some(event);
        page.len += 1;
        self.len = self
            .len
            .checked_add(1)
            .expect("trace cardinality exhausted");
    }

    /// Explicit unbudgeted observation copies history; operational append does not.
    pub(super) fn snapshot(&self) -> Vec<Event> {
        let mut events = Vec::with_capacity(self.len);
        let mut next = self.newest.as_deref();
        while let Some(page) = next {
            for event in page.events[..page.len].iter().rev() {
                events.push(event.expect("initialized event prefix"));
            }
            next = page.previous.as_deref();
        }
        events.reverse();
        events
    }
}

#[cfg(feature = "certification-test-authority")]
impl Drop for EventTrace {
    fn drop(&mut self) {
        // Final reclamation is unbudgeted, but not recursively stack-sized.
        while let Some(mut page) = self.newest.take() {
            self.newest = page.previous.take();
        }
    }
}
