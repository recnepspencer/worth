//! Whether World answered a publication inside the current advancement with an
//! effect (public #132).
//!
//! The only setter is Query's mapping of World's publication outcome: World
//! answered `Performed` or `ProductUnpublished`. `NoEffect` never sets it. The
//! only reader is the advancement runner. When this is set, the runner keeps a
//! completed body's own typed outcome past a late native stop, because the
//! publication can no longer be withheld. The marker is transient: it is never
//! stored and never consulted for anything else, so it is not a second record
//! of the commit.
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

thread_local! {
    static SLOT: RefCell<Option<PublishedEffect>> = const { RefCell::new(None) };
}

/// One advancement's marker, shared with the member computes it lends to
/// descendant workers.
#[derive(Clone)]
pub(crate) struct PublishedEffect(Arc<AtomicBool>);

impl PublishedEffect {
    /// A fresh, unset marker. Each advancement creates its own, so no earlier
    /// body's publication can leak into it.
    pub(crate) fn fresh() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    /// The marker installed on this thread, captured by lent work so that a
    /// publication on a descendant worker still reaches its advancement.
    pub(crate) fn current() -> Option<Self> {
        SLOT.with(|slot| slot.borrow().clone())
    }

    /// Install `marker` on this thread until the scope drops, also on unwind.
    pub(crate) fn install(marker: Option<Self>) -> PublishedEffectScope {
        PublishedEffectScope {
            previous: SLOT.with(|slot| slot.replace(marker)),
            _thread: PhantomData,
        }
    }

    /// World answered a publication of this advancement with an effect.
    pub(crate) fn observed(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

/// Called only where Query maps World's publication outcome, for `Performed`
/// and `ProductUnpublished`. Outside an advancement there is no marker and
/// nothing to record.
pub(crate) fn record() {
    SLOT.with(|slot| {
        if let Some(marker) = slot.borrow().as_ref() {
            marker.0.store(true, Ordering::Release);
        }
    });
}

/// Restores the thread's previous marker when dropped.
#[must_use = "the marker is installed only while its scope is held"]
pub(crate) struct PublishedEffectScope {
    previous: Option<PublishedEffect>,
    _thread: PhantomData<Rc<()>>,
}

impl Drop for PublishedEffectScope {
    fn drop(&mut self) {
        let previous = self.previous.take();
        SLOT.with(|slot| *slot.borrow_mut() = previous);
    }
}
