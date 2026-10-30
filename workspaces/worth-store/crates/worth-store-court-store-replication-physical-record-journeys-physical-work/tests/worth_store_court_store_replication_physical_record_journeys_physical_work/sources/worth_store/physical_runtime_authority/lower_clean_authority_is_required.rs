use worth_store_buffer_pool::{
    CandidateFrameCleanAuthority, DirtyPhysicalFrame, FrameWritebackCleanAuthority,
    PhysicalWritebackClaim,
};

fn zero_argument_cleaning_is_absent(
    claim: PhysicalWritebackClaim,
    dirty: DirtyPhysicalFrame,
) {
    let _ = claim.complete_writeback();
    let _ = dirty.complete_candidate_publication();
}

fn cleaning_authorities_are_not_interchangeable(
    claim: PhysicalWritebackClaim,
    dirty: DirtyPhysicalFrame,
    candidate: CandidateFrameCleanAuthority,
    writeback: FrameWritebackCleanAuthority,
) {
    let _ = claim.complete_writeback(&candidate);
    let _ = dirty.complete_candidate_publication(&writeback);
}

fn main() {}
