//! Shared root description DATA; neither a read capability nor a source owner.
mod root;
mod roster;
pub(crate) use root::{PreparedCheckpointRoot, RelationalCheckpointRoot};
pub(crate) use roster::{CheckpointRoots, PreparedCheckpointRoots};
