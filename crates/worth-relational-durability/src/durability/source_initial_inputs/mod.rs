//! Native-owned original source initialization and artifact attribution.
pub mod chronology;
pub mod commitment_observation;
pub mod complete_source;
pub mod frame;
pub mod input_commitments;
pub use complete_source::RelationalCompleteSourceArtifact;
pub mod normalized;
pub mod normalized_admission;
pub mod recorded;
pub mod schema_catalog;
pub mod source_admission;
pub use frame::{
    RelationalInitialInputsPublicationDetail, RelationalInitialInputsRefusal,
    RelationalSourceInitialInputsArtifact,
};
pub mod capture;
pub mod decoding;

pub mod image;

pub mod producer_roster;
