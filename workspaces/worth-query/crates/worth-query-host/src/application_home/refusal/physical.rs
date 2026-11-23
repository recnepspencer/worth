//! Read-only physical open refusal diagnostics grant no runtime or recovery authority.
use super::WorthQueryApplicationHomeOpenRefused;
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_store_application::{ApplicationHomeOpenRefusal, ApplicationHomePhysicalRefusal};

impl<Schema: ApplicationSchemaComposition, Program>
    WorthQueryApplicationHomeOpenRefused<Schema, Program>
{
    /// Store's physical refusal when the Home location itself could not be
    /// opened: contended by a live owner or tombstoned by a pending discard.
    pub fn physical_refusal(&self) -> Option<&ApplicationHomePhysicalRefusal> {
        match self {
            Self::Physical {
                cause: ApplicationHomeOpenRefusal::Physical(cause),
                ..
            } => Some(cause),
            _ => None,
        }
    }
}
