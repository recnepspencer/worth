//! The runtime composes `operation_body`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::operation_body::host::OperationBodyHost;

impl<Schema: ApplicationSchema> OperationBodyHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn operation_body_host(&self) -> &dyn OperationBodyHost<Schema> {
        self
    }
}
