//! Independent histories read owner advancement reports at a chosen posture.
//! The same observation returns ordered commit/publication evidence and typed
//! application outcomes; there is no serial-only charge ledger.

pub(super) mod expected_history;
mod observation;
pub(super) mod reuse_cases;
mod seeded_world;
mod structural_cost;

mod application;
mod computation;
mod installation;
mod operation;
mod read;

mod matrix;
