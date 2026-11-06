//! The subjects authorization admits: an authenticated principal and a resolved scope entity.
//!
//! Authorization owns these contracts and the primary graph implements them beside the values
//! it mints, so each value, its producer and its seal stay with the graph while authorization
//! reads only what admission compares and asks the graph for the reads only it can perform.
mod principal;
mod scope;

pub(in crate::domain_computation) use principal::WorthQueryAdmissionPrincipal;
pub(in crate::domain_computation) use scope::WorthQueryAdmissionScope;
