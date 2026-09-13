//! Named harness topology for Store certification and replay scenarios.
//!
//! `production_facade` contains helpers that exercise production-owned
//! capabilities through their public lifecycle. `test_authority` contains the
//! synthetic courtroom-only witnesses and shortcut attempts that exist only to
//! falsify production topology.

#[cfg(feature = "certification-world")]
mod blob;
#[cfg(feature = "boundary-fixtures")]
pub mod fixtures;
#[cfg(feature = "layout-fixtures")]
pub mod layout;
#[cfg(feature = "layout-fixtures")]
pub mod layout_evolution;
#[cfg(feature = "layout-fixtures")]
mod lsm_execution_fixture;
#[cfg(feature = "physical-isolation-fixtures")]
pub mod physical_isolation;
#[cfg(feature = "physical-isolation-fixtures")]
pub mod physical_reference;
#[cfg(feature = "physical-residency-fixtures")]
pub mod physical_residency;
#[cfg(feature = "certification-world")]
pub mod physical_simulation;
#[cfg(feature = "certification-world")]
mod pressure;
#[cfg(feature = "boundary-fixtures")]
pub mod production_facade;
#[cfg(feature = "physical-isolation-fixtures")]
pub mod recovery;
#[cfg(feature = "scheduling-fixtures")]
pub mod scheduling;
#[cfg(feature = "physical-isolation-fixtures")]
pub mod test_authority;

#[cfg(feature = "layout-fixtures")]
pub use lsm_execution_fixture::{
    observe_lsm_maintenance_owner_cases, observe_lsm_owner_cases, LsmOwnerCaseObservations,
};
#[cfg(feature = "boundary-fixtures")]
pub use production_facade::*;
