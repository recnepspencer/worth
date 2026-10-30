//! Native actor capture is binding-affine, replay-safe, and independent of caller values.
pub use super::*;
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub use worth_query_host::facade::{
    application_installation::{
        in_memory_program_from_checkpoint, WorthQueryCheckpointCapturePolicy,
    },
    declaration::{application_schema::*, portable_identity::WorthQueryPortableTypeIdentity},
};
