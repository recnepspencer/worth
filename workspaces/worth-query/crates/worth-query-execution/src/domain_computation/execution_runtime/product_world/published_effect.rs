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

/// One advancement's marker. World publications run on the advancement's
/// opening thread: lent member computes do not publish (876 publications across
/// the synchronous-edit courts, none in lent work; `d132-thread-probe.log`).
#[derive(Clone)]
pub(crate) struct PublishedEffect(Arc<AtomicBool>);

impl PublishedEffect {
    /// A fresh, unset marker. Each advancement creates its own, so no earlier
    /// body's publication can leak into it.
    pub(crate) fn fresh() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
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
/// nothing to record. A publication inside lent work (a member compute or its
/// descendant workers) would have no marker and reopen #132 silently, so
/// courts refuse it loudly.
pub(crate) fn record() {
    SLOT.with(|slot| {
        debug_assert!(
            slot.borrow().is_some() || !worth_execution::ExecutionOwnerContext::is_active(),
            "a World publication inside lent work has no advancement marker (public #132)"
        );
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
