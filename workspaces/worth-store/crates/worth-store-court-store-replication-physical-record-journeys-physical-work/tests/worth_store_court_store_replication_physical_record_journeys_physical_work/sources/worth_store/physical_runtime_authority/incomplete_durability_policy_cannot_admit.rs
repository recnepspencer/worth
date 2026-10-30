use std::num::{NonZeroU32, NonZeroU64};

use worth_store::physical_runtime::{
    GroupCommitDelay, GroupCommitLimit, PhysicalDurabilityDeclaration,
};
use worth_store_physical_backend::PhysicalDurabilityAdmissionBasis;

fn admit_incomplete(basis: PhysicalDurabilityAdmissionBasis) {
    let incomplete = PhysicalDurabilityDeclaration::builder().group_commit(
        GroupCommitLimit::new(NonZeroU32::new(32).unwrap()),
        GroupCommitDelay::new(NonZeroU64::new(1).unwrap()),
    );
    let _ = incomplete.admit(basis);
}

fn main() {}
