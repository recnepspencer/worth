//! Installed audit-query meaning. A declaration grants no read or retention authority.
mod definition;
mod metadata;
mod target;

pub use definition::{
    ApplicationHistoryQueryDefinition, ApplicationHistoryQueryDefinitionBuilder,
    ApplicationHistoryQueryDefinitionDenial, ErasedApplicationHistoryQueryDefinition,
    WorthQueryPortableApplicationHistoryQueryParts,
};
pub use metadata::{
    ApplicationHistoryMetadataFacet, ApplicationHistoryMetadataMeaning,
    ApplicationHistoryMetadataRef,
};
pub use target::{ApplicationHistoryAspectSelection, ApplicationHistoryTarget};

use crate::application_schema::ApplicationSchema;
use crate::portable_identity::WorthQueryPortableType;

/// Names one declared history query and the ordinary binding that originally
/// disclosed its subjects. That association grants no historical disclosure.
pub trait ApplicationHistoryQueryBinding<Schema: ApplicationSchema>:
    WorthQueryPortableType + Sized + 'static
{
    type SubjectRead: super::ApplicationQueryBinding<Schema>;
    const IDENTITY: &'static str;
}
