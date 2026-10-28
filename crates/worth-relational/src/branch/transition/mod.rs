//! Durable branch lifecycle transitions (Milestone 2 D8).
//!
//! Fork, archive, deletion-requested and delete each become durable before
//! they become live. A transition is recorded runtime-neutrally, carries its
//! own contiguous sequence (it never consumes a patch position) and the
//! performed commit position it follows. Publication runs inside the
//! patch-position reservation, after every fallible preparation step and
//! before any live effect; its fates are those of a commit (D2).

mod durable_step;
mod positioned;
mod record;
mod sequence;
mod stop;

pub(crate) use durable_step::{
    LiveBranchTransitionStep, RelationalBranchTransitionDenial,
    RelationalBranchTransitionDurableStep, ReplayedBranchTransitionDivergence,
    ReplayedBranchTransitionStep,
};
pub use positioned::PositionedBranchTransition;
pub use record::RelationalBranchTransition;
pub use sequence::RelationalBranchTransitionSequence;
pub use stop::RelationalBranchTransitionStop;
