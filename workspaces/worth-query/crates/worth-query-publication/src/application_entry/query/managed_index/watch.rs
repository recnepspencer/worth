//! Bound requests authorize every whole-Parent opening and refresh independently.
use super::*;
mod governed_next;
mod governed_open;
use worth_query_execution::facade::primary_graph::{
    WorthQueryManagedIndexWatch, WorthQueryManagedIndexWatchOutcome,
};

type Row<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type Query<S, B> = <B as ApplicationQueryBinding<S>>::Query;

impl<'application, 'principal, 'scope, Schema, Intent>
    WorthQueryApplicationQueryRequest<'application, 'principal, 'scope, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
{
    /// Reads the full Parent and opens an inverse-routed current-refresh watch.
    /// The Parent declares Live eligibility and uses the configured maintained
    /// predicate provider. Each subsequent call supplies a fresh bound request.
    pub fn watch_index<I: ApplicationManagedIndexBinding<Schema>>(
        self,
    ) -> Result<
        (
            WorthQueryPublishedApplicationResult<
                Query<Schema, Intent::Binding>,
                Row<Schema, Intent::Binding>,
            >,
            WorthQueryManagedIndexWatch<Schema, I>,
        ),
        WorthQueryApplicationRequestQueryDenial,
    >
    where
        <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        Row<Schema, Intent::Binding>:
            WorthQueryApplicationProjection<Schema, Query<Schema, Intent::Binding>>,
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
                            .open_application_managed_index_watch::<Intent::Binding, I>(
                                &phase, binding, external, plan,
                            )
                            .map(|(read, watch)| {
                                (
                                    publish_application_result(read.into_admitted_disclosed()),
                                    watch,
                                )
                            })
                            .map_err(WorthQueryApplicationRequestQueryDenial::ManagedIndex)
                    },
                ));
            })
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest);
        completion::completed_effect(advancement, completed)
    }

    /// Refreshes the whole Parent for the next notification. Notification B and
    /// the independently completed current read C are identified separately.
    pub fn next_watch<I: ApplicationManagedIndexBinding<Schema>>(
        self,
        watch: &mut WorthQueryManagedIndexWatch<Schema, I>,
    ) -> Result<
        WorthQueryManagedIndexWatchOutcome<
            Query<Schema, Intent::Binding>,
            Row<Schema, Intent::Binding>,
        >,
        WorthQueryApplicationRequestQueryDenial,
    >
    where
        <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        Row<Schema, Intent::Binding>:
            WorthQueryApplicationProjection<Schema, Query<Schema, Intent::Binding>>,
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
                            .next_application_managed_index_watch::<Intent::Binding, I>(
                                &phase, binding, external, plan, watch,
                            )
                            .map_err(WorthQueryApplicationRequestQueryDenial::ManagedIndex)
                    },
                ));
            })
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest);
        finish_next(advancement, completed)
    }
}

fn finish_next<Q, R>(
    advancement: Result<(), WorthQueryApplicationRequestQueryDenial>,
    completed: Option<
        Result<WorthQueryManagedIndexWatchOutcome<Q, R>, WorthQueryApplicationRequestQueryDenial>,
    >,
) -> Result<WorthQueryManagedIndexWatchOutcome<Q, R>, WorthQueryApplicationRequestQueryDenial> {
    match (advancement, completed) {
        (_, Some(Ok(outcome)))
            if matches!(&outcome, WorthQueryManagedIndexWatchOutcome::Delivered(_)) =>
        {
            Ok(outcome)
        }
        (Err(cause), _) => Err(cause),
        (Ok(()), Some(outcome)) => outcome,
        (Ok(()), None) => unreachable!("completed advancement ran the managed watch body"),
    }
}
