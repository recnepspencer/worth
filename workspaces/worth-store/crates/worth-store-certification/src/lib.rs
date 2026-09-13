#![doc = include_str!("courtroom/cross_cutting/certification_compile_fail_proofs.md")]
#![forbid(unsafe_code)]

//! Store certification: focused owner checks and direct boundary evidence.

pub mod courtroom;
pub mod evidence;
#[cfg(test)]
mod physical_fixture_encoding;

mod public_api;

pub use courtroom::blobs::capsule_readiness_provenance::{
    certify_blob_capsule_readiness, BlobCapsuleReadinessCertificationReport,
};
pub use public_api::*;
