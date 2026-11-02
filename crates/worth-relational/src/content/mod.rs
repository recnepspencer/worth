//! Immutable content authority carried by Native transactions and selected roots.
mod admission_fault;
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
mod read_closure;
mod reconstruction;
mod registry;
pub use read_closure::RelationalContentReadClosure;
mod resolution;
mod schema;
pub use schema::{
    RelationalContentFieldBinding, RelationalContentFormatDeclaration,
    RelationalContentSchemaSnapshot,
};
mod historical;
mod preimage;
mod projection;
mod source;
mod transaction;
pub(crate) use publication::{commit_denial, prepare_publication, ContentPublication};
pub(crate) use source::NativeContentSourceView;

pub use attachments::RelationalContentAttachments;
pub use codec::{RelationalContentCodec, RelationalContentSink};
pub(crate) use control::run_commit_stage;
pub use control::RelationalContentControl;
pub use denial::{RelationalContentDenial, RelationalContentRejection};
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
