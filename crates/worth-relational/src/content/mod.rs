//! Immutable content authority carried by Native transactions and selected roots.
mod admission_fault;
mod attachments;
pub(crate) mod codec;
pub(crate) mod codec_control;
pub(crate) mod control;
pub(crate) mod decoding;
pub(crate) mod denial;
mod directory;
mod field_references;
mod frame;
mod frame_admission;
pub(crate) mod publication;
pub(crate) mod read_closure;
mod reconstruction;
pub(crate) mod registry;
mod resolution;
pub(crate) mod schema;
mod schema_transition;
pub use schema::{
    RelationalContentFieldBinding, RelationalContentFormatDeclaration,
    RelationalContentSchemaSnapshot,
};
mod historical;
mod preimage;
mod projection;
mod restoration;
pub(crate) mod source;
mod transaction;

pub use attachments::RelationalContentAttachments;
pub use codec::{RelationalContentCodec, RelationalContentSink};
pub use codec_control::RelationalContentControl;
pub use denial::{RelationalContentDenial, RelationalContentRejection};
pub(crate) use directory::ContentDirectory;
pub use frame::{RelationalContentFrame, RelationalContentRecord};
pub use registry::{RelationalContentFormat, RelationalContentFormatRegistry};

pub(crate) mod limits;
pub use limits::{MAXIMUM_CONTENT_FRAME_BYTES, MAXIMUM_STAGED_CONTENT_BYTES};

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

mod basis_read;
