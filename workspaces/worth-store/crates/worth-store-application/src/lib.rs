//! One Store-owned application lifecycle, composed by the Query host.
//!
//! Opening progresses through concrete recovery custody. A partial native
//! readmission is not application service and grants no absent owner proof.

mod checkpoint;
mod close;
mod deadline;
mod descriptor;
mod discard;
mod frame_encoding;
mod home_context;
#[cfg(feature = "certification-test-authority")]
mod home_controls_for_test;
mod home_request;
mod open;
mod opening_control;
mod query_obligations;
mod registry;
mod relational_preparation;
mod request_stage;
mod resolve;
#[cfg(test)]
mod tests;
mod world_section;

pub mod facade;
pub use facade::*;
