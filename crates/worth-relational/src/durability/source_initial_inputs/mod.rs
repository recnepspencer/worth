//! Native-owned original source initialization and artifact attribution.
mod chronology;
mod commitment_observation;
mod complete_source;
mod frame;
mod input_commitments;
pub use complete_source::RelationalCompleteSourceArtifact;
pub(crate) mod normalized;
pub(crate) mod normalized_admission;
pub(crate) mod recorded;
pub(crate) mod schema_catalog;
pub(crate) mod source_admission;
pub use frame::{
    RelationalInitialInputsPublicationDetail, RelationalInitialInputsRefusal,
    RelationalSourceInitialInputsArtifact,
};
pub(crate) mod capture;
mod decoding;
mod fresh_initialization;
pub(crate) mod image;
pub use fresh_initialization::RelationalFreshInitialization;
