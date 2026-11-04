//! One borrowed installed query, including its exact optional binding observation.
use super::*;

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct InstalledQueryView<'a> {
    compiled: &'a Arc<WorthQueryCompiledApplicationQuery>,
    binding: Option<&'a super::super::WorthQueryInstalledApplicationQueryBindingObservation>,
}

impl<'a> InstalledQueryView<'a> {
    fn new(
        compiled: &'a Arc<WorthQueryCompiledApplicationQuery>,
        binding: Option<&'a super::super::WorthQueryInstalledApplicationQueryBindingObservation>,
    ) -> Self {
        Self { compiled, binding }
    }

    pub fn binding_identity(&self) -> &'a ApplicationSchemaBindingIdentity {
        self.compiled.binding_identity()
    }
    pub fn identity(&self) -> &'a WorthQueryInstalledApplicationQueryIdentity {
        self.compiled.identity()
    }
    pub fn name(&self) -> &'a str {
        self.compiled.name()
    }
    pub fn authority_identity(&self) -> &'a str {
        self.compiled.authority_identity()
    }
    pub fn read_graph(&self) -> &'a WorthQueryInstalledGraphReadContract {
        self.compiled.read_graph()
    }
    pub fn binding_observation(
        &self,
    ) -> Option<&'a super::super::WorthQueryInstalledApplicationQueryBindingObservation> {
        self.binding
    }
    pub fn graph_obligations(&self) -> WorthQueryInstalledGraphObligationInspection<'a> {
        self.compiled.graph_obligations()
    }
}

impl<S, Q, P, R, Sc> WorthQueryInstalledApplicationQuery<S, Q, P, R, Sc> {
    #[doc(hidden)]
    pub fn installed_view(&self) -> InstalledQueryView<'_> {
        InstalledQueryView::new(&self.compiled, self.binding.as_ref())
    }
}
