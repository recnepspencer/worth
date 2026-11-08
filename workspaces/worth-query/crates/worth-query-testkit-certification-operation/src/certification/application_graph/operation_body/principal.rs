//! Native actor capture is binding-affine, replay-safe, and independent of caller values.
pub use super::*;
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub use worth_query_host::facade::{
    application_installation::{
        WorthQueryCheckpointCapturePolicy, WorthQueryInterimCheckpointRestore,
    },
    declaration::{application_schema::*, portable_identity::WorthQueryPortableTypeIdentity},
};
