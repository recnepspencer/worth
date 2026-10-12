//! A funded edge is owned by its batch and linked at its current upstream.
use super::super::cleanup::closed_notes::ClosedNoteBacking;
use super::super::record_capacity::RecordCapacity;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};

#[derive(Default)]
pub(in super::super) struct IncomingClaims {
    first: Option<Arc<ClaimLink>>,
    len: usize,
}
struct ClaimLink {
    owner: Weak<ClosedNoteBacking>,
    published: Arc<AtomicBool>,
    target: Mutex<Weak<ClosedNoteBacking>>,
    links: Mutex<Links>,
    _capacity: Arc<RecordCapacity>,
}
#[derive(Default)]
struct Links {
    previous: Weak<ClaimLink>,
    next: Option<Arc<ClaimLink>>,
    linked: bool,
}
pub(in super::super) struct ClaimBatch {
    edges: Vec<Arc<ClaimLink>>,
    published: Arc<AtomicBool>,
    _capacity: Arc<RecordCapacity>,
}
fn arc_bytes<T>() -> Option<usize> {
    let (layout, _) = std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .ok()?;
    Some(layout.pad_to_align().size())
}
pub(super) fn batch_bytes(count: usize) -> Option<usize> {
    arc_bytes::<RecordCapacity>()?
        .checked_add(arc_bytes::<AtomicBool>()?)?
        .checked_add(count.checked_mul(
            arc_bytes::<ClaimLink>()?.checked_add(std::mem::size_of::<Arc<ClaimLink>>())?,
        )?)
}
impl ClaimBatch {
    pub(super) fn prepare(
        owner: &Arc<ClosedNoteBacking>,
        targets: impl ExactSizeIterator<Item = Arc<ClosedNoteBacking>>,
        capacity: Arc<RecordCapacity>,
    ) -> Self {
        let published = Arc::new(AtomicBool::new(false));
        let mut edges = Vec::with_capacity(targets.len());
        for target in targets {
            edges.push(Arc::new(ClaimLink {
                owner: Arc::downgrade(owner),
                published: Arc::clone(&published),
                target: Mutex::new(Arc::downgrade(&target)),
                links: Default::default(),
                _capacity: Arc::clone(&capacity),
            }));
        }
        Self {
            edges,
            published,
            _capacity: capacity,
        }
    }
    pub(in super::super) fn install(&self) {
        for edge in &self.edges {
            edge.link();
        }
    }
    pub(in super::super) fn deactivate(&self) {
        self.published.store(false, Ordering::Release);
    }
    pub(in super::super) fn publish(&self) {
        self.published.store(true, Ordering::Release);
    }
    pub(in super::super) fn retarget(
        &mut self,
        position: usize,
        target: Option<&Arc<ClosedNoteBacking>>,
    ) {
        let edge = &self.edges[position];
        edge.unlink();
        if let Some(target) = target {
            *edge.target.lock().unwrap() = Arc::downgrade(target);
            edge.link();
        } else {
            self.edges.remove(position);
        }
    }
}
impl Drop for ClaimBatch {
    fn drop(&mut self) {
        self.published.store(false, Ordering::Release);
        for edge in &self.edges {
            edge.unlink();
        }
    }
}
impl ClaimLink {
    fn link(self: &Arc<Self>) {
        let target = self
            .target
            .lock()
            .unwrap()
            .upgrade()
            .expect("claimed row is retained");
        let mut incoming = target.claimants.lock().unwrap();
        let mut links = self.links.lock().unwrap();
        assert!(!links.linked);
        links.next = incoming.first.take();
        if let Some(next) = &links.next {
            next.links.lock().unwrap().previous = Arc::downgrade(self);
        }
        links.linked = true;
        incoming.first = Some(Arc::clone(self));
        incoming.len += 1;
    }
    fn unlink(&self) {
        let Some(target) = self.target.lock().unwrap().upgrade() else {
            return;
        };
        let mut incoming = target.claimants.lock().unwrap();
        let mut links = self.links.lock().unwrap();
        if !links.linked {
            return;
        }
        let previous = std::mem::take(&mut links.previous);
        let next = links.next.take();
        if let Some(previous) = previous.upgrade() {
            previous.links.lock().unwrap().next = next.clone();
        } else {
            incoming.first = next.clone();
        }
        if let Some(next) = next {
            next.links.lock().unwrap().previous = previous;
        }
        links.linked = false;
        incoming.len -= 1;
    }
}
impl IncomingClaims {
    /// O(1) per incoming claim; no claimant discovery or registry pass.
    pub(in super::super) fn len(&self) -> usize {
        self.len
    }
    pub(in super::super) fn closed_published<
        C: super::super::refreshed_rejoin::OccurrenceSearchCharge,
    >(
        &self,
        charge: &mut C,
    ) -> Result<usize, C::Refusal> {
        let mut count = 0;
        let mut edge = self.first.clone();
        while let Some(current) = edge {
            charge.inspect_row()?;
            if current.published.load(Ordering::Acquire)
                && current.owner.upgrade().is_some_and(|owner| {
                    let state = *owner.claim_owner.lock().unwrap();
                    state.live && state.closed_cached
                })
            {
                count += 1;
            }
            edge = current.links.lock().unwrap().next.clone();
        }
        Ok(count)
    }
}
