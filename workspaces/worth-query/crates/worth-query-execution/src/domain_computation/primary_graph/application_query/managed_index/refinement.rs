//! Optional domain exactness over one genuine candidate row and the same probe.
use {crate::domain_computation::vocabulary::primary_graph::control::WorthQueryManagedIndexControl as Control, crate::domain_computation::primary_graph::vocabulary::application_query::provider::WorthQueryManagedIndexProbeValue as Probe, super::WorthQueryManagedIndexProvider as Provider, super::WorthQueryManagedIndexProviderStop as Stop, super::WorthQueryManagedIndexSourceRow as Row};
use worth_query_declaration::facade::{
    application_query::{ApplicationManagedIndexBinding, ApplicationManagedIndexRefinement},
    application_schema::ApplicationSchema,
};

/// Exact row-local meaning, separately versioned from conservative candidate mechanics.
/// Every callback uses only the supplied Source projection, probe and controlled work.
/// The provider must qualify exact acceptance implies its candidate membership.
pub trait WorthQueryManagedIndexRefinementProvider<S, I>: Provider<S, I>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    const REFINEMENT_IDENTITY: &'static str;
    const REFINEMENT_VERSION: u32;
    /// Required even when the complete candidate set is empty.
    fn validate_refinement_probe(
        &self,
        probe: &Probe<S, I>,
        control: &Control<'_>,
    ) -> Result<(), Stop<Self::Denial>>;
    fn refine_source(
        &self,
        row: Row<'_, S, I>,
        probe: &Probe<S, I>,
        control: &Control<'_>,
    ) -> Result<bool, Stop<Self::Denial>>;
}

pub struct RefinementCallbacks<S, I, P>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: Provider<S, I>,
{
    pub(super) meaning: ApplicationManagedIndexRefinement,
    pub(super) validate: fn(&P, &Probe<S, I>, &Control<'_>) -> Result<(), Stop<P::Denial>>,
    pub(super) evaluate:
        fn(&P, Row<'_, S, I>, &Probe<S, I>, &Control<'_>) -> Result<bool, Stop<P::Denial>>,
}
impl<S, I, P> Copy for RefinementCallbacks<S, I, P>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: Provider<S, I>,
{
}
impl<S, I, P> Clone for RefinementCallbacks<S, I, P>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: Provider<S, I>,
{
    fn clone(&self) -> Self {
        *self
    }
}
impl<S, I, P> RefinementCallbacks<S, I, P>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexRefinementProvider<S, I>,
{
    pub(in crate::domain_computation::primary_graph) fn new() -> Self {
        Self {
            meaning: ApplicationManagedIndexRefinement::row_local(
                P::REFINEMENT_IDENTITY,
                P::REFINEMENT_VERSION,
            ),
            validate: P::validate_refinement_probe,
            evaluate: P::refine_source,
        }
    }
}
