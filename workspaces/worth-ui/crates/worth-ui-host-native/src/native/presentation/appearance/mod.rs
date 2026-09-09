#![allow(
    dead_code,
    reason = "Gate 1 retains native appearance qualification until the later host cutover"
)]

//! Gate 1 native appearance qualification lane.
//!
//! This module consumes only sealed Gate 0 host mechanics. It is deliberately
//! not emitted by the live presentation work or event loop. The retained draw
//! list carries staged coverage through its existing transaction lifecycle.
//! Gate 5 owns the future protocol handoff.

mod antialiasing;
mod backdrop_pipeline;
#[cfg(feature = "certification-support")]
mod certification_profile;
mod command;
mod damage;
mod damage_candidate;
mod geometry;
mod outline_pipeline;
mod replay;
#[cfg(test)]
mod replay_tests;
mod retained;
#[cfg(feature = "certification-support")]
mod surface_certification;
mod surface_pipeline;
pub(crate) mod text_foreground;

pub(crate) mod cursor;

#[cfg(feature = "certification-support")]
pub(crate) use certification_profile::staged_appearance_profile_contract;
#[cfg(all(test, feature = "certification-support"))]
pub(crate) use certification_profile::STAGED_APPEARANCE_MECHANICS;

#[cfg(test)]
mod backdrop_tests;
#[cfg(test)]
mod cursor_tests;
#[cfg(test)]
mod geometry_tests;
#[cfg(test)]
mod mounted_mechanic_fixtures;
#[cfg(test)]
mod outline_tests;
#[cfg(test)]
mod retained_tests;
#[cfg(test)]
mod surface_tests;

#[cfg(feature = "certification-support")]
pub(crate) mod text_retention_certification;

pub(crate) use geometry::UiNativeAppearanceScale;
pub(crate) use retained::{UiNativeAppearanceRetained, UiNativeTextCoverageUndo};

#[cfg(feature = "certification-support")]
pub use surface_certification::{
    certify_mounted_surface_sample, UiNativeSurfaceSampleCertification,
    UiNativeSurfaceSampleCertificationDenial,
};

pub(crate) use command::{UiNativeAppearanceCommand, UiNativeAppearanceCommandIdentity};
pub(crate) use replay::UiNativeAppearanceReplayRegion;
