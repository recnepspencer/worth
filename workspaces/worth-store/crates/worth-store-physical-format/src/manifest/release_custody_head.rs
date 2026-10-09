mod block;
mod entry;
mod roster;
mod semantic;
mod transition;

pub use block::{
    ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadBlockV1, ReleaseCustodyHeadBlockViewV1,
};
pub use entry::{ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1};
pub use roster::ReleaseCustodyHeadRosterDigestV1;
pub use semantic::{
    verify_release_custody_head_controls, verify_release_custody_head_controls_view,
    verify_release_custody_head_successor, verify_release_custody_head_successor_view,
    ReleaseCustodyHeadControlIdentityV1,
};
pub use transition::{
    ReleaseCustodyHeadMutationV1, ReleaseCustodyHeadNodeWriteV1, ReleaseCustodyHeadPathNodeV1,
    ReleaseCustodyHeadTransitionLimitsV1, ReleaseCustodyHeadTransitionV1,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseCustodyHeadDenial {
    Frame(crate::DurableFrameDenial),
    UnsupportedVersion,
    Malformed,
    Identity,
    Capacity,
    CanonicalOrder,
    Reference,
    Digest,
    Path,
    Mutation,
    Budget,
}

#[cfg(test)]
mod tests;
