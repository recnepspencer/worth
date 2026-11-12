//! Descriptions cannot substitute for their exact installed roles or target slots.
pub use super::*;
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub use worth_query_host::facade::{
    application_installation::{
        WorthQueryCheckpointCapturePolicy, WorthQueryInterimCheckpointRestore,
    },
    declaration::application_schema::*,
};
