//! Optional phase measurements, kept separate from constitutional diagnostics.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

static ENABLED: AtomicBool = AtomicBool::new(false);
static PHASES: Mutex<BTreeMap<&'static str, (u64, Duration)>> = Mutex::new(BTreeMap::new());

pub(crate) struct Session(Option<PathBuf>);
impl Session {
    pub(crate) fn new(path: Option<PathBuf>) -> Self {
        ENABLED.store(path.is_some(), Ordering::Relaxed);
        Self(path)
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        ENABLED.store(false, Ordering::Relaxed);
        if let Some(path) = &self.0 {
            let phases = PHASES.lock().expect("profile lock");
            let rows: BTreeMap<_, _> = phases
                .iter()
                .map(|(name, (count, elapsed))| (*name, (*count, elapsed.as_secs_f64())))
                .collect();
            let bytes = serde_json::to_vec_pretty(&rows).expect("profile serialization");
            std::fs::write(path, bytes).expect("write profile sidecar");
        }
    }
}

pub(crate) fn measure<T>(name: &'static str, operation: impl FnOnce() -> T) -> T {
    if !ENABLED.load(Ordering::Relaxed) {
        return operation();
    }
    let start = Instant::now();
    let result = operation();
    let mut phases = PHASES.lock().expect("profile lock");
    let row = phases.entry(name).or_default();
    row.0 += 1;
    row.1 += start.elapsed();
    result
}

pub(crate) fn record_count(name: &'static str, count: usize) {
    if ENABLED.load(Ordering::Relaxed) {
        PHASES
            .lock()
            .expect("profile lock")
            .entry(name)
            .or_default()
            .0 += count as u64;
    }
}
