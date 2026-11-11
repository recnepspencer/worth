//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<RelationalHomeTargetCoverage>(
            "crates/worth-relational/src/runtime/home_program_transition/coverage.rs",
            "RelationalHomeTargetCoverage",
            128 * 1024,
        );
        owner.field::<usize>("authored_batches");
        owner.field::<worth_foundational::facade::HomeProgramTransitionBodyDigest>(
            "rejudgment_body",
        );
        owner.field::<ExecutionArray<RecordRef>>("records");
        owner.field::<Option<ExecutionMemoryReservation>>("_batch_memory");
    }
}
