//! Fresh authentication and native execution have distinct terminal contracts.
use super::*;
use worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation;

pub(super) enum NativeCommandAction<'a> {
    Execute,
    Resolve,
    Recover(&'a WorthQueryProductUnpublishedRecovery),
}
pub(super) enum NativeCommandResult {
    Commit(WorthQueryApplicationCommitOutcome),
    Resolved(WorthQueryAdmittedIdempotencyRead),
    Recovered(WorthQueryManagedApplicationRecoveryOutcome),
}

pub(super) trait NativeCommandBackend<S: ApplicationSchema, D> {
    fn validate(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), D>;
    fn run(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        action: NativeCommandAction<'_>,
    ) -> Result<NativeCommandResult, D>;
}

struct AuthenticatedCommandBackend<A, F, V> {
    authenticate: A,
    execute: F,
    marker: std::marker::PhantomData<fn() -> V>,
}

pub(super) fn authenticated_backend<S: ApplicationSchema, D, A, F, V>(
    authenticate: A,
    execute: F,
) -> impl NativeCommandBackend<S, D>
where
    A: for<'p, 'e, 'q, 'a, 'b, 'c> FnMut(
        &'p WorthQueryAdvancementPhase<'e>,
        &'q WorthQueryApplicationRequest<'a, 'b, 'c, S>,
    ) -> Result<
        (WorthQuerySelectedProductOperation<'a, S>, V),
        D,
    >,
    F: for<'p, 'e, 'q, 'a, 'b, 'c, 'x> FnMut(
        &'p WorthQueryAdvancementPhase<'e>,
        &'q WorthQueryApplicationRequest<'a, 'b, 'c, S>,
        WorthQuerySelectedProductOperation<'a, S>,
        V,
        NativeCommandAction<'x>,
    ) -> Result<NativeCommandResult, D>,
{
    AuthenticatedCommandBackend {
        authenticate,
        execute,
        marker: std::marker::PhantomData,
    }
}

impl<S: ApplicationSchema, D, A, F, V> NativeCommandBackend<S, D>
    for AuthenticatedCommandBackend<A, F, V>
where
    A: for<'p, 'e, 'q, 'a, 'b, 'c> FnMut(
        &'p WorthQueryAdvancementPhase<'e>,
        &'q WorthQueryApplicationRequest<'a, 'b, 'c, S>,
    ) -> Result<
        (WorthQuerySelectedProductOperation<'a, S>, V),
        D,
    >,
    F: for<'p, 'e, 'q, 'a, 'b, 'c, 'x> FnMut(
        &'p WorthQueryAdvancementPhase<'e>,
        &'q WorthQueryApplicationRequest<'a, 'b, 'c, S>,
        WorthQuerySelectedProductOperation<'a, S>,
        V,
        NativeCommandAction<'x>,
    ) -> Result<NativeCommandResult, D>,
{
    fn validate(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), D> {
        (self.authenticate)(phase, request).map(|_| ())
    }

    fn run(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        action: NativeCommandAction<'_>,
    ) -> Result<NativeCommandResult, D> {
        let (selected, values) = (self.authenticate)(phase, request)?;
        (self.execute)(phase, request, selected, values, action)
    }
}
