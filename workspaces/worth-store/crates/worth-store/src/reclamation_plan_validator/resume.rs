//! Chained-crash resume of a durable extent release (`M9`).
//!
//! Path A barriers are `Intent -> PublishFreeSpaceRoot -> ReclaimComplete ->
//! AdmitToAllocator`. Recovery observes the newest durable barrier and may
//! itself be interrupted before completion. A published free-space root is
//! loaded by bootstrap, so the range is already in the allocator's durable
//! index; admitting it again would free a range a later writer may already
//! hold at a newer generation.

#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

verus_items! {
    /// The newest durable barrier recovery observes for one release.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum ReleaseBarrier {
        IntentDurable,
        FreeSpacePublished,
        /// Recovery seeds only unresolved releases, so production never
        /// resumes from here; the variant pins the settled plan.
        #[cfg_attr(not(test), allow(dead_code))]
        CompletionDurable,
    }

    /// The remaining effects one resume performs.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct ResumeRelease {
        pub(crate) publish_free_space: bool,
        pub(crate) record_completion: bool,
        /// This process itself returns the range to the allocator, after
        /// completion and `WritebackFenceDrain`.
        pub(crate) admit_to_allocator: bool,
    }
}

#[cfg(verus_keep_ghost)]
verus! {
    /// Allocator releases a resume from `barrier` performs.
    pub(crate) closed spec fn spec_releases(plan: ResumeRelease) -> int {
        if plan.admit_to_allocator { 1 } else { 0 }
    }

    /// The barrier a resume leaves durable once every planned effect lands.
    pub(crate) closed spec fn spec_after(barrier: ReleaseBarrier, plan: ResumeRelease) -> ReleaseBarrier {
        if plan.record_completion { ReleaseBarrier::CompletionDurable }
        else if plan.publish_free_space { ReleaseBarrier::FreeSpacePublished }
        else { barrier }
    }
}

verus_fn! {
    /// Plan the remaining release effects from the newest durable barrier.
    pub(crate) fn resume_release(barrier: ReleaseBarrier) -> (result: ResumeRelease);
    ensures {
        result.publish_free_space == (barrier == ReleaseBarrier::IntentDurable),
        result.record_completion == (barrier != ReleaseBarrier::CompletionDurable),
        // Only the process that published the free-space root releases it,
        // so a range is admitted at most once across any chain of crashes.
        result.admit_to_allocator == result.publish_free_space,
        result.admit_to_allocator ==> result.record_completion,
        spec_after(barrier, result) == ReleaseBarrier::CompletionDurable,
        spec_releases(result) <= 1,
        barrier != ReleaseBarrier::IntentDurable ==> spec_releases(result) == 0,
    }
    body {
        match barrier {
            ReleaseBarrier::IntentDurable => ResumeRelease {
                publish_free_space: true,
                record_completion: true,
                admit_to_allocator: true,
            },
            ReleaseBarrier::FreeSpacePublished => ResumeRelease {
                publish_free_space: false,
                record_completion: true,
                admit_to_allocator: false,
            },
            ReleaseBarrier::CompletionDurable => ResumeRelease {
                publish_free_space: false,
                record_completion: false,
                admit_to_allocator: false,
            },
        }
    }
}
