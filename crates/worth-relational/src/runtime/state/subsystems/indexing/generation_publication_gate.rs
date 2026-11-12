//! Coordinate catalog growth from actual admission through its fixed install.
use std::convert::Infallible;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[derive(Debug, Default)]
pub(super) struct GenerationPublicationGate {
    occupied: Mutex<bool>,
    released: Condvar,
}

#[derive(Debug)]
pub(super) struct GenerationPublicationPermit {
    owner: Arc<GenerationPublicationGate>,
}

impl GenerationPublicationGate {
    pub(super) fn enter<Stop>(
        self: &Arc<Self>,
        mut poll: impl FnMut() -> Result<(), Stop>,
    ) -> Result<GenerationPublicationPermit, Stop> {
        loop {
            // Neither the original request nor candidate expiry may disappear
            // behind an owner wait. The callback never runs under this mutex.
            poll()?;
            let mut occupied = self.occupied.lock().unwrap_or_else(|e| e.into_inner());
            if !*occupied {
                *occupied = true;
                return Ok(GenerationPublicationPermit {
                    owner: Arc::clone(self),
                });
            }
            let (occupied, _) = self
                .released
                .wait_timeout(occupied, Duration::from_millis(10))
                .unwrap_or_else(|e| e.into_inner());
            drop(occupied);
        }
    }

    pub(super) fn enter_uncontrolled(self: &Arc<Self>) -> GenerationPublicationPermit {
        match self.enter(|| Ok::<_, Infallible>(())) {
            Ok(permit) => permit,
            Err(never) => match never {},
        }
    }
}

impl Drop for GenerationPublicationPermit {
    fn drop(&mut self) {
        *self
            .owner
            .occupied
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = false;
        self.owner.released.notify_one();
    }
}

#[cfg(test)]
mod tests;
