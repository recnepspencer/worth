mod key;
mod lookup;
mod quarantine;
mod source;

pub(in crate::physical_runtime) use key::{DedupeIndexKey, DedupeIndexValue};
pub(super) use lookup::lookup_reusable_chunk;
pub use lookup::BlobDedupeFailure;
pub(in crate::physical_runtime) use quarantine::verify_selected_quarantine;
pub(in crate::physical_runtime) use source::VerifiedDedupeSource;
pub(in crate::physical_runtime) use source::{
    verify_selected_claim_source_with_selected_chunk, verify_source,
    verify_source_with_selected_chunk,
};
