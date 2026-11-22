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

impl WorthQueryOutputProgress {
    #[doc(hidden)]
    pub fn published_commit_for_test(&self) -> &Option<CompositeCommitIdentity> {
        &self.published_commit
    }
}

impl WorthQueryOutputProgress {
    #[doc(hidden)]
    pub fn published_commit_mut_for_test(&mut self) -> &mut Option<CompositeCommitIdentity> {
        &mut self.published_commit
    }
}

impl WorthQueryOutputProgress {
    #[doc(hidden)]
    pub fn advancement_for_test(&self) -> &WorthQueryOutputAdvancement {
        Self::advancement(self)
    }
}

impl WorthQueryOutputProgress {
    #[doc(hidden)]
    pub fn claim_for_test(&mut self, claim: WorthQueryOutputClaimIdentity) {
        Self::claim(self, claim)
    }
}

impl WorthQueryOutputProgress {
    #[doc(hidden)]
    pub fn claim_stopped_checkpoint_for_close_for_test(
        &mut self,
    ) -> Option<(WorthQueryOutputClaimIdentity, WorthQueryOutputCheckpoint)> {
        Self::claim_stopped_checkpoint_for_close(self)
    }
}

impl WorthQueryOutputProgress {
    #[doc(hidden)]
    pub fn finish_stopped_for_test(&mut self, denial: WorthQueryOutputDemandDenial) {
        Self::finish_stopped(self, denial)
    }
}

impl WorthQueryOutputProgress {
    #[doc(hidden)]
    pub fn owns_checkpoint_for_test(&self, checkpoint: &WorthQueryOutputCheckpoint) -> bool {
        Self::owns_checkpoint(self, checkpoint)
    }
}

impl WorthQueryOutputProgress {
    #[doc(hidden)]
    pub fn reopen_for_test(&mut self) {
        Self::reopen(self)
    }
}

impl WorthQueryOutputProgress {
    #[doc(hidden)]
    pub fn restored_for_test(completion: ReadyCompletion) -> Self {
        Self::restored(completion)
    }
}

impl WorthQueryOutputProgress {
    #[doc(hidden)]
    pub fn stopped_without_checkpoint_for_test(denial: WorthQueryOutputDemandDenial) -> Self {
        Self::stopped_without_checkpoint(denial)
    }
}
