//! Field-bound completed content values and their exact door refusals.
pub mod admitted;
pub mod resolution;
pub mod selection;
pub use resolution::{resolve_frame, typed};
pub use selection::{ContentFieldSelection, DecodeReference};
pub mod denial;
pub use admitted::AdmittedContent;
pub use denial::{ContentReadDenial, ContentWriteDenial};
pub use worth_relational::facade::content::{RelationalContentDenial, RelationalContentRejection};

pub mod staging;
pub use staging::{require_contract, ContentStaging};

pub mod retention_work;
pub use retention_work::checkpoint as retention_checkpoint;
