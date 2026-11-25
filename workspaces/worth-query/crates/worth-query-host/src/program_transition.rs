//! Ergonomic Host open-with-transition facade compiling declarative
//! `worth_query_program_transition!` specs onto WORTH's real Directory Home
//! program transition authority.

mod backfill;
mod backfill_memory;
mod config;
mod open;
mod outcome;
mod planner;

pub use config::{WorthQueryProgramTransitionConfigDenial, WorthQueryProgramTransitionOpenConfig};
pub use open::{open_directory_with_transition, open_directory_with_transition_and_step};
pub use outcome::{
    WorthQueryProgramTransitionOpenDenial, WorthQueryTransitionOpenOutcome,
    WorthQueryTransitionOpenedHome,
};
