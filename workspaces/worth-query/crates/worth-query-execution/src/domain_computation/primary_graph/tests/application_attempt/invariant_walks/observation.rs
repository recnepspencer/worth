//! Receiver-owned observations; recording never changes the safety rule.
use super::schema::{Node, WalkSchema};
use crate::facade::application_invariants::*;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};

pub(super) type Entity = WorthQueryApplicationInvariantEntity<WalkSchema, Node>;

#[derive(Default)]
struct State {
    armed: AtomicBool,
    preparation: AtomicUsize,
    evaluation: AtomicUsize,
    deep: AtomicUsize,
    roots: Mutex<Vec<Entity>>,
    reads: Mutex<Vec<WorthQueryInvariantProjectionWork>>,
}

#[derive(Clone, Default)]
pub(super) struct Observation(Arc<State>);

impl Observation {
    pub(super) fn arm(&self) {
        self.0.preparation.store(0, Ordering::SeqCst);
        self.0.evaluation.store(0, Ordering::SeqCst);
        self.0.deep.store(0, Ordering::SeqCst);
        self.0.roots.lock().unwrap().clear();
        self.0.reads.lock().unwrap().clear();
        self.0.armed.store(true, Ordering::SeqCst);
    }

    pub(super) fn prepared(&self, roots: impl Iterator<Item = Entity>) {
        if self.0.armed.load(Ordering::SeqCst) {
            self.0.preparation.fetch_add(1, Ordering::SeqCst);
            self.0.roots.lock().unwrap().extend(roots);
        }
    }

    pub(super) fn evaluated(&self, reads: WorthQueryInvariantProjectionWork) {
        if self.0.armed.load(Ordering::SeqCst) {
            self.0.evaluation.fetch_add(1, Ordering::SeqCst);
            self.0.reads.lock().unwrap().push(reads);
        }
    }

    pub(super) fn deep(&self) {
        if self.0.armed.load(Ordering::SeqCst) {
            self.0.deep.fetch_add(1, Ordering::SeqCst);
        }
    }

    pub(super) fn contacts(&self) -> [usize; 3] {
        [
            self.0.preparation.load(Ordering::SeqCst),
            self.0.evaluation.load(Ordering::SeqCst),
            self.0.deep.load(Ordering::SeqCst),
        ]
    }

    pub(super) fn roots(&self) -> Vec<Entity> {
        self.0.roots.lock().unwrap().clone()
    }

    pub(super) fn reads(&self) -> Vec<WorthQueryInvariantProjectionWork> {
        self.0.reads.lock().unwrap().clone()
    }
}
