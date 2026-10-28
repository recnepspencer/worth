//! Explicit authenticated registration and cold construction on the Query receiver.
use super::*;
use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_execution::facade::primary_graph::WorthQueryManagedIndex;

impl<'application, 'principal, 'scope, Schema, Intent>
    WorthQueryApplicationQueryRequest<'application, 'principal, 'scope, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
{
    /// Opens the installed index's shared registration in Created state.
    /// This does not read Source or build a root. Repeated opens share status;
    /// closing any sibling disposes that registration for all siblings.
    /// The installed Parent must have Public authorization and Public disclosure
    /// without capability/rule governance; other postures refuse during binding.
    pub fn open_index<I: ApplicationManagedIndexBinding<Schema>>(
        self,
    ) -> Result<WorthQueryManagedIndex<Schema, I>, WorthQueryApplicationRequestQueryDenial>
    where
        <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
    {
        let application = self.application;
        let request = self.scope;
        application
            .with_application_advancement(request, |phase| {
                self.with_admitted_query(&phase, |application, binding, _, plan| {
                    application
                        .open_application_managed_index(&phase, binding, plan)
                        .map_err(WorthQueryApplicationRequestQueryDenial::ManagedIndex)
                })
            })
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }

    /// Reads the complete current Source and builds its index under this request.
    /// Created or Evicted queries return NotReady until this explicit call succeeds.
    /// Every publication on the same branch makes the image cold again.
    pub fn rebuild_index<I: ApplicationManagedIndexBinding<Schema>>(
        self,
        index: &WorthQueryManagedIndex<Schema, I>,
    ) -> Result<(), WorthQueryApplicationRequestQueryDenial>
    where
        <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
    {
        let application = self.application;
        let request = self.scope;
        application
            .with_application_advancement(request, |phase| {
                self.with_admitted_query(&phase, |application, binding, external, plan| {
                    application
                        .rebuild_application_managed_index(&phase, binding, external, plan, index)
                        .map_err(WorthQueryApplicationRequestQueryDenial::ManagedIndex)
                })
            })
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }
}
