//! A funded edge is owned by its batch and linked at its current upstream.
use super::super::cleanup::closed_notes::ClosedNoteBacking;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Weak;

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
}
impl ClaimBatch {
    pub(super) fn prepare(
        owner: &Arc<ClosedNoteBacking>,
        targets: impl ExactSizeIterator<Item = Arc<ClosedNoteBacking>>,
    ) -> Self {
        let published = Arc::new(AtomicBool::new(false));
        let mut edges = Vec::with_capacity(targets.len());
        for target in targets {
            edges.push(Arc::new(ClaimLink {
                owner: Arc::downgrade(owner),
                published: Arc::clone(&published),
                target: Mutex::new(Arc::downgrade(&target)),
                links: Default::default(),
            }));
        }
        Self { edges, published }
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
        match previous.upgrade() {
            Some(previous) => {
                previous.links.lock().unwrap().next = next.clone();
            }
            _ => {
                incoming.first = next.clone();
            }
        }
        if let Some(next) = next {
            next.links.lock().unwrap().previous = previous;
        }
        links.linked = false;
        incoming.len -= 1;
    }
}
impl IncomingClaims {
    pub(in super::super) fn closed_published(&self) -> usize {
        let mut count = 0;
        let mut edge = self.first.clone();
        while let Some(current) = edge {
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
        count
    }
}
