use super::ServingPhysicalRuntime;

impl ServingPhysicalRuntime {
    /// Holds the retirement owner after the claim and before the intent append.
    pub fn certification_pause_before_retirement_intent(&self) {
        self.parts.publication.arm_retirement_intent_gate();
    }

    pub fn certification_retirement_intent_arrived(&self) -> bool {
        self.parts.publication.retirement_intent_arrived()
    }

    pub fn certification_release_retirement_intent(&self) {
        self.parts.publication.release_retirement_intent_gate();
    }

    /// Parks the retirement that is in flight at `seam` until the process is killed.
    ///
    /// Seam 1 is after the durable intent and checkpoint, before unlink.
    /// Seam 2 is after unlink, before the removal directory sync.
    pub fn certification_arm_retirement_kill(
        &self,
        seam: u8,
    ) -> std::sync::Arc<std::sync::atomic::AtomicBool> {
        self.parts.publication.arm_retirement_kill(seam)
    }

    /// Stops retirement after the intent and checkpoint, before the segment unlink.
    pub fn certification_stop_before_retirement_delete(&self) {
        self.parts
            .publication
            .certification_stop_before_retirement_delete();
    }

    /// Fails the next segment-removal directory sync after the unlink has returned.
    pub fn certification_fail_next_removal_directory_sync(&self) {
        self.parts
            .work_runtime
            .executor
            .record_serving_media()
            .certification_fail_next_removal_directory_sync();
    }

    /// Stops retirement after the segment file is gone and before completion.
    pub fn certification_stop_after_retirement_delete(&self) {
        self.parts
            .publication
            .certification_stop_after_retirement_delete();
    }

    /// Ordinary publication work cannot remove a segment. The public command
    /// constructor refuses before any media effect.
    pub fn certification_public_segment_removal_rejected(&self) -> bool {
        self.parts
            .publication
            .certification_public_segment_removal_rejected()
    }
}
