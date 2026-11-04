use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct FixtureDomainPortProbe {
    pub comparator_entries: Arc<AtomicUsize>,
    pub progress_entries: Arc<AtomicUsize>,
    pub repeated_state_entries: Arc<AtomicUsize>,
}

impl FixtureDomainPortProbe {
    pub fn entered_comparator(&self) {
        self.comparator_entries.fetch_add(1, Ordering::SeqCst);
    }

    pub fn entered_progress(&self) {
        self.progress_entries.fetch_add(1, Ordering::SeqCst);
    }

    pub fn entered_repeated_state(&self) {
        self.repeated_state_entries.fetch_add(1, Ordering::SeqCst);
    }

    pub fn entries(&self) -> [usize; 3] {
        [
            self.comparator_entries.load(Ordering::SeqCst),
            self.progress_entries.load(Ordering::SeqCst),
            self.repeated_state_entries.load(Ordering::SeqCst),
        ]
    }
}
