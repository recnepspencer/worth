//! Receiver-owned observations; recording never changes the safety rule.
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::schema::Node;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::schema::WalkSchema;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantEntity;
use worth_query_execution::facade::application_invariants::WorthQueryInvariantProjectionWork;

pub type Entity = WorthQueryApplicationInvariantEntity<WalkSchema, Node>;

#[derive(Default)]
pub struct State {
    pub armed: AtomicBool,
    pub preparation: AtomicUsize,
    pub evaluation: AtomicUsize,
    pub deep: AtomicUsize,
    pub roots: Mutex<Vec<Entity>>,
    pub reads: Mutex<Vec<WorthQueryInvariantProjectionWork>>,
}

#[derive(Clone, Default)]
pub struct Observation(Arc<State>);

impl Observation {
    pub fn arm(&self) {
        self.0.preparation.store(0, Ordering::SeqCst);
        self.0.evaluation.store(0, Ordering::SeqCst);
        self.0.deep.store(0, Ordering::SeqCst);
        self.0.roots.lock().unwrap().clear();
        self.0.reads.lock().unwrap().clear();
        self.0.armed.store(true, Ordering::SeqCst);
    }

    pub fn prepared(&self, roots: impl Iterator<Item = Entity>) {
        if self.0.armed.load(Ordering::SeqCst) {
            self.0.preparation.fetch_add(1, Ordering::SeqCst);
            self.0.roots.lock().unwrap().extend(roots);
        }
    }

    pub fn evaluated(&self, reads: WorthQueryInvariantProjectionWork) {
        if self.0.armed.load(Ordering::SeqCst) {
            self.0.evaluation.fetch_add(1, Ordering::SeqCst);
            self.0.reads.lock().unwrap().push(reads);
        }
    }

    pub fn deep(&self) {
        if self.0.armed.load(Ordering::SeqCst) {
            self.0.deep.fetch_add(1, Ordering::SeqCst);
        }
    }

    pub fn contacts(&self) -> [usize; 3] {
        [
            self.0.preparation.load(Ordering::SeqCst),
            self.0.evaluation.load(Ordering::SeqCst),
            self.0.deep.load(Ordering::SeqCst),
        ]
    }
}
