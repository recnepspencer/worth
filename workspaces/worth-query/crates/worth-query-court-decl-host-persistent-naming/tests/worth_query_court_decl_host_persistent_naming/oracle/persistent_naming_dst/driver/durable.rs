//! Store crash and reopen at a commit boundary (Phase 7B): the Directory
//! Home is dropped while still owned and its directory reopened by a fresh
//! Home, runtime and installation. A checkpoint captured first makes the
//! reopen restore it plus the Journal tail after it; otherwise the reopen
//! replays every Journal frame since the last checkpoint. The driver keeps
//! only what an application keeps across a restart: the retained
//! `WorthQueryName`s and its record keys.
use super::Driver;

impl Driver {
    /// Crashes and reopens the world, capturing a checkpoint first when
    /// `checkpoint`; returns `checkpoint`.
    pub fn reopen(&mut self, checkpoint: bool) -> bool {
        if checkpoint {
            self.world.checkpoint();
        }
        self.world.crash_and_reopen();
        // Every later operation runs under fresh idempotency keys, so no
        // reopened request is answered by an earlier recorded receipt.
        self.generation += 1;
        checkpoint
    }
}
