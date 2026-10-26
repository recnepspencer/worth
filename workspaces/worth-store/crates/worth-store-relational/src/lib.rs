//! A Relational home on Worth Store.
//!
//! [`RelationalStoreHome`] owns the durable lifecycle of one Relational
//! runtime: it creates or recovers the Store, readmits the home's checkpoint,
//! canonical journal and branch transition journal through Relational, and
//! attaches the journals as the runtime's only durable medium. A commit or
//! branch transition is durable before it is visible; a crash at any point
//! recovers the exact prefix of durable commits and transitions.
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
mod recovered_transitions;

pub use checkpoint_record::RelationalStoreCheckpointRecordRefusal;
pub use home::{
    RelationalStoreCheckpoint, RelationalStoreCheckpointFailure, RelationalStoreCommitFate,
    RelationalStoreHome, RelationalStoreHomeClosed, RelationalStoreHomeOpenRefusal,
    RelationalStoreHomeRequest, RelationalStoreTransitionFate,
};
pub use home_descriptor::RelationalStoreHomeDescriptorRefusal;
pub use physical_store::{RelationalStorePhysicalRefusal, RelationalStorePhysicalStage};
pub use recovered_history::RelationalStoreJournalRefusal;
pub use recovered_transitions::RelationalStoreTransitionJournalRefusal;

mod cold_source;
pub use cold_source::RelationalStoreColdReadCounters;
