//! Immutable content authority carried by Native transactions and selected roots.
pub use worth_relational_contracts::internal_port::content::admission_fault;
pub mod attachments;
pub use worth_relational_contracts::internal_port::content::codec;
pub use worth_relational_contracts::internal_port::content::codec_control;
pub mod control;
pub use worth_relational_contracts::internal_port::content::denial;
pub mod directory;
pub mod field_references;
pub mod frame;
pub mod frame_admission;
pub mod reconstruction;
pub use worth_relational_contracts::internal_port::content::registry;
pub mod resolution;
pub use schema::{
    RelationalContentFieldBinding, RelationalContentFormatDeclaration,
    RelationalContentSchemaSnapshot,
};
pub use worth_relational_contracts::internal_port::content::schema;

pub use attachments::RelationalContentAttachments;
pub use codec::{RelationalContentCodec, RelationalContentSink};
pub use codec_control::RelationalContentControl;
pub use denial::{RelationalContentDenial, RelationalContentRejection};
pub use directory::ContentDirectory;
pub use frame::{RelationalContentFrame, RelationalContentRecord};
pub use registry::{RelationalContentFormat, RelationalContentFormatRegistry};

pub use limits::{MAXIMUM_CONTENT_FRAME_BYTES, MAXIMUM_STAGED_CONTENT_BYTES};
pub use worth_relational_contracts::internal_port::content::limits;

pub fn charge_bytes(
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
