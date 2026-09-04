#![allow(
    dead_code,
    reason = "Gate 1 retains native appearance qualification until the later host cutover"
)]

//! Gate 1 native appearance qualification lane.
//!
//! This module consumes only sealed Gate 0 host mechanics. It is deliberately
//! not imported by the live presentation work, retained draw list, or event
//! loop. Gate 5 owns the future protocol handoff.

mod antialiasing;
mod backdrop_pipeline;
#[cfg(feature = "certification-support")]
mod certification_profile;
mod command;
mod damage;
mod damage_candidate;
mod geometry;
mod outline_pipeline;
mod retained;
mod surface_pipeline;

pub(crate) mod cursor;

#[cfg(feature = "certification-support")]
pub(crate) use certification_profile::{
    staged_appearance_profile_contract, STAGED_APPEARANCE_MECHANICS,
};

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
