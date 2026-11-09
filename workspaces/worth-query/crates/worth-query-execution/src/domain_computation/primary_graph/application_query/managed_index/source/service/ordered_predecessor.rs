//! Only a Native-declared predecessor in the selected branch image can seed birth.
use super::*;

impl<S, I, P> TypedSourceService<S, I, P>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    pub(super) fn prepare_ordered(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: ManagedIndexSourceScope<'_>,
        predecessor: Option<&Payload>,
    ) -> Result<PreparedManagedIndexSourceRoot, Denial> {
        let parent = scope.ordered_predecessor()?;
        let proposed = match (parent, predecessor) {
            (Some(parent), Some(payload)) => {
                let root = payload
                    .downcast_ref::<ManagedSourceRoot<S, I>>()
                    .ok_or(Denial::ForeignSource)?;
                if !root.installed.same_installation(&self.provider.installed)
                    || root.rows.program() != scope.program()
                    || root.rows.scope() != parent
                {
                    return Err(Denial::ForeignSource);
                }
                let changed = scope.prepare_ordered_step()?;
                self.update_changed(execution, &scope, payload, &changed, true)?
            }
            (Some(predecessor), None) if self.cold == Cold::Refuse => {
                return Err(Denial::PredecessorUnavailable { predecessor })
            }
            // The chain root builds its own membership. A missing parent costs
            // one successor build, never a recursive preparation of the parent.
            _ => self.cold(execution, &scope, &scope.proposed())?,
        };
        execution
            .consult()
            .map_err(|cause| source_storage(cause.into()))?;
        Ok(scope.seal(None, Some(proposed)))
    }
}
