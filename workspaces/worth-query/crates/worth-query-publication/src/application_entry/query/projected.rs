//! The named bound set for generic helpers that execute an installed query.
//!
//! It restates exactly the requirements of
//! [`super::WorthQueryApplicationQueryRequest::execute`] and is implemented for
//! every qualifying type. It grants no read permission; each execution is
//! admitted and authorized by its native owner.
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationProjection;

/// A query binding whose scope resolves for its principal and whose result
/// value projects from the installed query's rows.
pub trait WorthQueryProjectedQueryBinding<S: ApplicationSchema>:
    ApplicationQueryBinding<
    S,
    ScopeBinding: ApplicationQueryScopeResolution<
        S,
        <Self as ApplicationQueryBinding<S>>::PrincipalIdentity,
    >,
    ResultBinding: ApplicationStructuredValueBinding<
        Value: WorthQueryApplicationProjection<S, <Self as ApplicationQueryBinding<S>>::Query>,
    >,
>
{
}
impl<S, B> WorthQueryProjectedQueryBinding<S> for B
where
    S: ApplicationSchema,
    B: ApplicationQueryBinding<
        S,
        ScopeBinding: ApplicationQueryScopeResolution<
            S,
            <B as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
        ResultBinding: ApplicationStructuredValueBinding<
            Value: WorthQueryApplicationProjection<S, <B as ApplicationQueryBinding<S>>::Query>,
        >,
    >,
{
}

/// A query intent a generic helper can execute: `Q: WorthQueryProjectedQuery<S>`
/// replaces the scope-resolution and projection bounds on its binding.
pub trait WorthQueryProjectedQuery<S: ApplicationSchema>:
    ApplicationQueryIntent<S, Binding: WorthQueryProjectedQueryBinding<S>>
{
}
impl<S, Q> WorthQueryProjectedQuery<S> for Q
where
    S: ApplicationSchema,
    Q: ApplicationQueryIntent<S, Binding: WorthQueryProjectedQueryBinding<S>>,
{
}

/// The projected row value an executed query intent `Q` returns.
pub type WorthQueryProjectedValue<S, Q> =
    <<<Q as ApplicationQueryIntent<S>>::Binding as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
