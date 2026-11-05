//! The runtime composes `application_entry`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::application_entry::host::ApplicationEntryHost;

impl<Schema: ApplicationSchema> ApplicationEntryHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn application_entry_host(&self) -> &dyn ApplicationEntryHost<Schema> {
        self
    }
}
