//! Immutable content authority carried by Native transactions and selected roots.
mod attachments;
mod codec;
mod control;
mod denial;
mod directory;
mod frame;
mod publication;
mod reconstruction;
mod registry;
mod schema;
pub(crate) use reconstruction::require_content_free_readmission;
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
