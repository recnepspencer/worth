//! Gate 1 native appearance qualification lane.
//!
//! This module consumes only sealed Gate 0 host mechanics. It is deliberately
//! not imported by the live presentation work, retained draw list, or event
//! loop. Gate 5 owns the future protocol handoff.

mod antialiasing;
mod backdrop_pipeline;
mod command;
mod damage;
mod damage_candidate;
mod geometry;
mod outline_pipeline;
mod retained;
mod surface_pipeline;

pub(crate) mod cursor;

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
