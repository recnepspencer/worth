//! Stable receiver observations used by the ordinary provider courts.
use super::*;
impl Calls {
    pub fn totals(&self) -> (usize, usize, usize) {
        (
            self.capacity.load(Ordering::SeqCst),
            self.builds.load(Ordering::SeqCst),
            self.probe_validations.load(Ordering::SeqCst),
        )
    }
}
