//! A Relational home on Worth Store.
//!
//! [`RelationalStoreHome`] owns the durable lifecycle of one Relational
//! runtime: it creates or recovers the Store, readmits the home's checkpoint
//! and canonical journal through Relational, and attaches the journal as the
//! runtime's only durable medium. A commit is durable before it is visible;
//! a crash at any point recovers the exact prefix of durable commits.
//!
//! Store persists Relational's artifacts as opaque bytes. Relational owns
//! their format and readmission; this crate owns ordering, placement and the
//! sequence in which a home recovers.

mod checkpoint_record;
mod home;
mod home_descriptor;
mod journal_medium;
mod physical_store;
mod recovered_history;

pub use checkpoint_record::RelationalStoreCheckpointRecordRefusal;
pub use home::{
    RelationalStoreCheckpoint, RelationalStoreCheckpointFailure, RelationalStoreCommitFate,
    RelationalStoreHome, RelationalStoreHomeClosed, RelationalStoreHomeOpenRefusal,
    RelationalStoreHomeRequest,
};
pub use home_descriptor::RelationalStoreHomeDescriptorRefusal;
pub use physical_store::{RelationalStorePhysicalRefusal, RelationalStorePhysicalStage};
pub use recovered_history::RelationalStoreJournalRefusal;
