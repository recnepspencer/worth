//! Versioned durable persistence of persistent-name lineage.
//!
//! The legacy checkpoint and journal lineage codecs carry neither event
//! subjects, births, nor kept names; this log carries exactly that naming
//! payload, one appendable frame per committed naming batch, under an
//! explicit format version. Reopen rebuilds the derived name index from the
//! frames alone, readmitting each persisted identity only by re-verifying
//! its birth against the one canonical persistent-name hash.

mod event_shape;
mod format;
mod frame_decoding;
mod frame_encoding;
mod frame_readmission;
mod log_rebuild;
mod refusal;
mod wire;

pub use format::FORMAT_VERSION;
pub use frame_encoding::{
    encode_commit_frame, encode_durable_persistent_name_log, encode_log_header,
};
pub use log_rebuild::{admit_persisted_name, rebuild_name_index_from_durable_log};
pub use refusal::{
    PersistentNameDurabilityError, PersistentNameReferenceRole, PersistentNameWireSection,
};
