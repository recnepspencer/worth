//! The runtime composes `output_lineage`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::output_lineage::host::OutputLineageHost;

impl<Schema: ApplicationSchema> OutputLineageHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn output_lineage_host(&self) -> &dyn OutputLineageHost<Schema> {
        self
    }
}
