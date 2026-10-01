//! Store-owned selected chunk relocation through C.10 bounded SourceCopy.
//! Physical tier classification is a separate durable-format obligation.
mod movement;

pub use movement::{
    BlobMovementFailure, BlobMovementReadHold, BlobMovementReceipt, BlobMovementSession,
};
