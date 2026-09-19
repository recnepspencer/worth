//! Canonical truth envelope shapes for bridge-owned committed patch input.

use std::sync::Arc;

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectLocator, AspectMask, CanonicalFieldPath, MutationMask,
    ProjectionMask,
};

use crate::clone_budget::CheapClone;
use crate::identity::{
    BridgeIdentity, CommittedPatchDigestTag, TruthBranchTag, TruthCommitTag, TruthPatchTag,
};
use crate::snapshot::TruthSnapshotIdentity;

mod canonical;
mod construction;
mod core;
mod semantic;
#[cfg(test)]
#[path = "envelope/semantic_tests.rs"]
mod semantic_tests;
mod source_authority;
mod target;

pub use canonical::*;
pub use core::*;
pub use semantic::*;
pub use source_authority::*;
pub use target::*;
