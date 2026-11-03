//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputProgress {
    #[doc(hidden)]
    pub fn checkpoint_for_test(&self) -> &Option<WorthQueryOutputCheckpoint> {
        &self.checkpoint
    }
}

impl WorthQueryOutputProgress {
    #[doc(hidden)]
    pub fn new_for_test(checkpoint: WorthQueryOutputCheckpoint) -> Self {
        Self::new(checkpoint)
    }
}

impl WorthQueryOutputProgress {
    #[doc(hidden)]
    pub fn stop_for_test(&mut self, denial: WorthQueryOutputDemandDenial) {
        Self::stop(self, denial)
    }
}

impl WorthQueryOutputProgress {
    #[doc(hidden)]
    pub fn checkpoint_into_for_test(self) -> Option<WorthQueryOutputCheckpoint> {
        self.checkpoint
    }
}
