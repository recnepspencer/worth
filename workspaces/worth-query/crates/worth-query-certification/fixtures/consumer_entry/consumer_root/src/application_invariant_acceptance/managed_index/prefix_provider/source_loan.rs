//! Each concrete index lends its declared String field through the actual Source.
use super::*;
pub(in crate::application_invariant_acceptance) trait PrefixSourceIndex<S: ApplicationSchema>:
    ApplicationManagedIndexBinding<
        S,
        ProbeBinding: worth_query_decl::facade::application_schema::ApplicationScalarValueBinding<
            Value = Probe,
        >,
    > + Sized
{
    fn with_name<R>(
        row: &WorthQueryManagedIndexSourceRow<'_, S, Self>,
        visitor: impl FnOnce(&str) -> R,
    ) -> Result<R, WorthQueryManagedIndexTextReadDenial>;
    #[cfg(test)]
    fn inspect_text(
        calls: &Calls,
        row: &WorthQueryManagedIndexSourceRow<'_, S, Self>,
    ) -> Result<(), Stop>;
    #[cfg(test)]
    fn observe_update(
        _calls: &Calls,
        _before: &WorthQueryManagedIndexSourceRows<'_, S, Self>,
        _after: &WorthQueryManagedIndexSourceRows<'_, S, Self>,
        _changes: &[WorthQueryManagedIndexSourceChange],
    ) -> Result<(), Stop> {
        Ok(())
    }
}
