//! Parent custody validation stays within the installed connection owner.
use super::*;
// Only this owner dispatches admission; neither variant can become the other.
pub(super) enum Parent<
    'a,
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
> {
    Settled(&'a WorthQuerySettledHistoryProgramOutput<S, P, D>),
    Verified(&'a WorthQueryVerifiedHistoryParent<S, P, D>),
}

impl<
        S: ApplicationSchema + 'static,
        P: ApplicationProgramDefinition<S>,
        D: WorthQueryApplicationOutputDemand<S>,
    > Parent<'_, S, P, D>
{
    pub(super) fn validate<C>(
        &self,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        basis: &WorthQueryApplicationReadObservation,
        minimum: &WorthQueryApplicationReadObservation,
    ) -> Result<Option<Artifact>, Denial>
    where
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
    {
        match self {
            Parent::Settled(parent) => {
                if !parent.profile.same_custody(authority.profile())
                    || !parent.profile.matches_program(application)
                    || !std::sync::Arc::ptr_eq(&parent.original, authority.original())
                {
                    return Err(foreign());
                }
                let artifact = application
                    .validate_dependent_output_parent::<D, C>(&parent.settled)
                    .map_err(Denial::Demand)?;
                application
                    .select_dependent_output_basis(execution, &parent.settled, basis, minimum)
                    .map_err(Denial::Demand)?;
                Ok(artifact)
            }
            Parent::Verified(parent) => {
                let artifact = application
                    .validate_verified_history_parent::<D, C>(parent, authority)
                    .map_err(Denial::Demand)?;
                let original = parent.custody().original();
                if basis.branch_incarnation() != original.branch_incarnation()
                    || minimum.branch_incarnation() != original.branch_incarnation()
                    || basis.branch_identity() != original.branch_identity()
                    || minimum.branch_identity() != original.branch_identity()
                    || basis.selected_commit().ordinal()
                        < parent.observation().selected_commit().ordinal()
                    || basis.selected_commit().ordinal() < minimum.selected_commit().ordinal()
                {
                    return Err(foreign());
                }
                Ok(artifact)
            }
        }
    }
}
