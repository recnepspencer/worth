//! Immutable content authority carried by Native transactions and selected roots.
mod attachments;
mod codec;
mod control;
pub(crate) mod decoding;
mod denial;
mod directory;
mod field_references;
mod frame;
mod frame_admission;
mod publication;
mod reconstruction;
mod registry;
mod resolution;
mod schema;
pub use schema::{
    RelationalContentFieldBinding, RelationalContentFormatDeclaration,
    RelationalContentSchemaSnapshot,
};
mod transaction;
pub(crate) use publication::{commit_denial, prepare_publication, ContentPublication};

pub use attachments::RelationalContentAttachments;
pub use codec::{RelationalContentCodec, RelationalContentSink};
pub use control::RelationalContentControl;
pub use denial::RelationalContentDenial;
pub(crate) use directory::ContentDirectory;
pub use frame::{RelationalContentFrame, RelationalContentRecord};
pub use registry::{RelationalContentFormat, RelationalContentFormatRegistry};

pub const MAXIMUM_CONTENT_FRAME_BYTES: usize = 16 * 1024 * 1024;
pub const MAXIMUM_STAGED_CONTENT_BYTES: usize = 64 * 1024 * 1024;

pub(crate) fn charge_bytes(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    bytes: usize,
) -> Result<(), RelationalContentDenial> {
    control::run(execution, |control| {
        let mut remaining = bytes;
        while remaining > 0 {
            let chunk = remaining.min(4096);
            control.checkpoint(chunk as u64)?;
            remaining -= chunk;
        }
        Ok(())
    })
}
