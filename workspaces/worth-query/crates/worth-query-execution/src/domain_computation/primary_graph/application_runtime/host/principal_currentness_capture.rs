//! The runtime composes `principal_currentness_capture`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::principal_currentness_capture::host::PrincipalCurrentnessCaptureHost;

impl<Schema: ApplicationSchema> PrincipalCurrentnessCaptureHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn principal_currentness_capture_host(&self) -> &dyn PrincipalCurrentnessCaptureHost<Schema> {
        self
    }
}
