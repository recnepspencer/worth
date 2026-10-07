//! Independent request-history rules and public serial request observations.
//!
//! Part two supplies placement through `serial_observation::observe`'s
//! `Posture` argument, advancement reports through `serial_observation::AdvancementReport`, and
//! ordered commit observation when 7.7 exposes advancement commit positions. The serial adapter
//! observes completed member requests; it makes no aggregate advancement claim.

pub(super) mod expected_history;
pub(super) mod reuse_cases;
mod seeded_world;
mod serial_observation;
mod structural_cost;

mod application;
mod computation;
mod installation;
mod operation;
mod read;
