//! External compilation surface for the complete public Bank estate facade.

#![doc = include_str!(
    "../../../../worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md"
)]
#![forbid(unsafe_code)]

mod estate_commands;
mod estate_queries;

pub use estate_commands::{
    exercise_estate_commands, exercise_estate_lifecycle, EstateCommandCertificationDenial,
    EstateCommandInputs, EstateCommandStoppedOutcome, EstateLifecycleInputs,
    EstateLifecyclePrincipals, EstateLifecycleProgressionOutcome,
};
pub use estate_queries::{
    exercise_emergency_estate_queries, exercise_ordinary_estate_queries, EmergencyEstateQueryInputs,
};
