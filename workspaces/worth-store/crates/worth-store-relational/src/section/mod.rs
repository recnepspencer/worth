//! Relational's native section and journals; physical lifecycle belongs to Application.
mod attachment;
pub(crate) mod checkpoint;
mod close;
mod codec;
#[cfg(feature = "certification-test-authority")]
mod component_controls_for_test;
mod readmission;

pub use attachment::RelationalStoreSection;
pub use checkpoint::{PreparedRelationalSection, RelationalSectionCheckpointDenial};
pub use close::ClosingRelationalSection;
pub use readmission::RelationalSectionReadmissionDenial;
