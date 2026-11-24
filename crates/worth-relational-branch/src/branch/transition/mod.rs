//! Durable branch lifecycle transitions (Milestone 2 D8).
//!
//! Fork, archive, deletion-requested and delete each become durable before
//! they become live. A transition is recorded runtime-neutrally, carries its
//! own contiguous sequence (it never consumes a patch position) and the
//! performed commit position it follows. Publication runs inside the
//! patch-position reservation, after every fallible preparation step and
//! before any live effect; its fates are those of a commit (D2).

pub mod committed_evidence;
pub mod durable_step;
pub mod original_inputs;
pub mod positioned;
pub mod record;
pub use worth_relational_contracts::internal_port::branch::transition::sequence;
pub mod stop;

pub use committed_evidence::RelationalCommittedTransitionEvidence;
pub use durable_step::{
    RelationalBranchTransitionDenial, RelationalBranchTransitionDurableStep,
    RelationalTransitionPublication, ReplayedBranchTransitionDivergence,
    ReplayedBranchTransitionStep,
};
pub use positioned::PositionedBranchTransition;
pub use record::RelationalBranchTransition;
pub use sequence::RelationalBranchTransitionSequence;
pub use stop::BranchTransitionRecordError;
pub use stop::RelationalBranchTransitionStop;

pub mod durable_publication;
pub use durable_publication::RelationalDurableTransitionPublication;
