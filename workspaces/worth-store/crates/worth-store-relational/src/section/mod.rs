//! Relational's native section and journals; physical lifecycle belongs to Application.
mod attachment;
pub(crate) mod checkpoint;
mod close;
mod codec;
mod complete_codec;
#[cfg(feature = "certification-test-authority")]
mod component_controls_for_test;
mod readmission;
mod source_preparation;
mod unshared;
pub use source_preparation::{
    PreparedFreshRelationalHomeSource, PreparedRelationalHomeSource,
    RelationalHomeSourcePreparation,
};
pub use unshared::{UnrecoveredRelationalSection, UnsharedReadmittedRelationalSection};
pub(crate) mod selection;
pub use selection::RelationalCheckpointSelectionSource;

pub use attachment::RelationalStoreSection;
pub use checkpoint::{
    PreparedCompleteRelationalSection, PreparedRelationalSection,
    RelationalCheckpointSelectionProtection, RelationalSectionCheckpointDenial,
    RelationalSectionCompleteCheckpointPreparation,
};
pub use close::{
    ClosingRelationalSection, PreparedRelationalSectionClose, RelationalSectionCloseDenial,
    RetiredRelationalSectionAllocation,
};
pub use readmission::RelationalSectionReadmissionDenial;
