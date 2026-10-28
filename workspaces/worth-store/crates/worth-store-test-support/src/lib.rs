#![doc = include_str!("test_support_compile_fail_proofs.md")]
#![forbid(unsafe_code)]

//! Store test support — production-facade harness helpers plus named synthetic
//! courtroom authority for certification replay.
//!
//! Production admission authority lives in production crates. This crate falsifies
//! production topology through named harness modules rather than bypassing it:
//! `harness::production_facade` assembles legal production flows, while
//! `harness::test_authority` exposes courtroom-only synthetic evidence.

#[cfg(feature = "boundary-fixtures")]
mod allocation_sentinels;
#[cfg(feature = "boundary-fixtures")]
mod hostile_readmission_json_fixtures;
#[cfg(feature = "boundary-fixtures")]
mod json_fixture_boundary;
#[cfg(feature = "boundary-fixtures")]
mod large_record_streams;
#[cfg(feature = "boundary-fixtures")]
mod memory_pressure;
#[cfg(feature = "boundary-fixtures")]
mod native_aspect_fixture_authoring;
#[cfg(feature = "boundary-fixtures")]
mod native_aspect_fixtures;
#[cfg(feature = "boundary-fixtures")]
mod resident_pressure_fixtures;
#[cfg(feature = "boundary-fixtures")]
mod terminal_projection_json_fixtures;

pub mod compiler_boundary;
pub mod harness;
mod independent_executable;
mod test_directory;

pub use independent_executable::{
    IndependentExecutable, FRESH_PROCESS_RECOVERY, OFFLINE_RECOVERY_OBSERVER, OFFLINE_ROOT_OBSERVER,
};
pub use test_directory::TemporaryDirectory;

#[cfg(feature = "boundary-fixtures")]
pub use harness::production_facade;
#[cfg(feature = "boundary-fixtures")]
pub use harness::production_facade::*;
#[cfg(feature = "physical-isolation-fixtures")]
pub use harness::test_authority;
