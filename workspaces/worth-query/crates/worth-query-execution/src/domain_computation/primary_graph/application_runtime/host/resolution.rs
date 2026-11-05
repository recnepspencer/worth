//! The runtime composes `resolution`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::resolution::host::ResolutionHost;

impl<Schema: ApplicationSchema> ResolutionHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn resolution_host(&self) -> &dyn ResolutionHost<Schema> {
        self
    }
}
