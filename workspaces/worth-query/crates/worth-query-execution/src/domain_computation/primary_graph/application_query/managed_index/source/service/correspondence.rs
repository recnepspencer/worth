//! Bind Native correspondence backing to the actual Query installation.
use super::*;
impl<S, I, P> TypedSourceService<S, I, P>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    pub(super) fn corresponding_root(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: &ManagedIndexSourceScope<'_>,
    ) -> Result<Option<Payload>, Denial> {
        let Some(payload) = scope.corresponding_root()? else {
            return Ok(None);
        };
        let root = payload
            .downcast_ref::<ManagedSourceRoot<S, I>>()
            .ok_or(Denial::ForeignSource)?;
        if !root.installed.same_installation(&self.provider.installed) {
            return Err(Denial::ForeignSource);
        }
        execution
            .consult()
            .map_err(|cause| source_storage(cause.into()))?;
        Ok(Some(payload))
    }
}
