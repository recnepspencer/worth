//! The ordinary Host request forwards to the sole Execution resource owner.
use super::*;

impl<'application, 'principal, 'scope, Schema: ApplicationSchema>
    WorthQueryApplicationRequest<'application, 'principal, 'scope, Schema>
{
    pub fn resources(
        &self,
    ) -> worth_query_execution::facade::primary_graph::WorthQueryApplicationResourcesRequest<
        'application,
        'principal,
        'scope,
        Schema,
    > {
        self.application
            .application_resources(self.principal, self.scope, self.branch)
    }
}
