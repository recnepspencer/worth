//! Sealed test doors over this module's private members.
use super::*;

impl<S: ApplicationSchema + 'static> ManagedIndexProviderCatalog<S> {
    #[doc(hidden)]
    pub fn register_for_test<I, P>(
        &mut self, schema: &WorthQueryInstalledApplicationSchema<S>, owner: &str, provider: P, refinement: Option<super::super::refinement::RefinementCallbacks<S, I, P>>) -> Result<(), Denial>
    where
        I: ApplicationManagedIndexBinding<S>,
        P: WorthQueryManagedIndexProvider<S, I>, {
        Self::register::<I, P>(self, schema, owner, provider, refinement)
    }
}
