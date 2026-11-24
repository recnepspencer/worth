//! Shared root description DATA; neither a read capability nor a source owner.
pub mod root;
pub mod roster;
pub(crate) use root::{PreparedCheckpointRoot, RelationalCheckpointRoot};
pub(crate) use roster::{CheckpointRoots, PreparedCheckpointRoots};
