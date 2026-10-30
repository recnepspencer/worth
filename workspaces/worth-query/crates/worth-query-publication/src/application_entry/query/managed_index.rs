//! Explicit authenticated registration and cold construction on the Query receiver.
use super::*;
use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_execution::facade::primary_graph::WorthQueryManagedIndex;

mod completion;
mod eviction;
mod governed;

impl<'application, 'principal, 'scope, Schema, Intent>
    WorthQueryApplicationQueryRequest<'application, 'principal, 'scope, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
{
    /// Opens the installed index's shared registration without building it.
    /// Maintained material attaches Ready; standalone registration starts Created.
    /// This does not read Source. Repeated opens share status;
    /// closing any sibling disposes that registration for all siblings.
    /// This entrance uses ordinary Parent admission. Use `open_index_governed`
    /// or `open_index_approved` when disclosure requires a capability.
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
    /// Standalone Created/Evicted queries return NotReady until construction.
    /// Maintained rebuild refills the genuine selected Native cache; ordinary
    /// mutations continue to maintain that canonical resource.
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
        let mut completed = None;
        let advancement = application
            .with_application_advancement(request, |phase| {
                completed = Some(self.with_admitted_query(
                    &phase,
                    |application, binding, external, plan| {
                        application
                            .rebuild_application_managed_index(
                                &phase, binding, external, plan, index,
                            )
                            .map_err(WorthQueryApplicationRequestQueryDenial::ManagedIndex)
                    },
                ));
            })
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest);
        completion::completed_effect(advancement, completed)
    }
}
