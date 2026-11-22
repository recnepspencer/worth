//! Receiver-owned corruption controls for the exact public refresh boundary.
use super::*;

impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> WorthQueryManagedIndexWatch<S, I> {
    /// Exchange only retained association evidence between two genuine watches.
    #[doc(hidden)]
    pub fn exchange_selection_association_for_test(&mut self, other: &mut Self) {
        let original = &mut self.state.as_mut().expect("open court watch").selection;
        let other = &mut other.state.as_mut().expect("open court watch").selection;
        std::mem::swap(&mut original.association, &mut other.association);
    }

    /// Exchange only retained scope evidence; audience and queue remain intact.
    #[doc(hidden)]
    pub fn exchange_selection_scope_for_test(&mut self, other: &mut Self) {
        let original = &mut self.state.as_mut().expect("open court watch").selection;
        let other = &mut other.state.as_mut().expect("open court watch").selection;
        std::mem::swap(&mut original.scope, &mut other.scope);
    }

    /// Observe custody without retaining any of the four immutable backings.
    #[doc(hidden)]
    pub fn selection_custody_observer_for_test(&self) -> impl Fn() -> [bool; 4] + 'static {
        self.state
            .as_ref()
            .expect("open court watch")
            .selection
            .custody_observer()
    }
}
