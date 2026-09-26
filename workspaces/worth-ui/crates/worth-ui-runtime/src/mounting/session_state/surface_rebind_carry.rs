//! What a rebind carries from the binding it ends to its successor.
//!
//! A rebind keeps a surface's selection bindings and its occurrence geometry,
//! with the direct input staged in it, for the binding that succeeds it. A
//! surface that ends with no successor, by a plain deregistration or by a
//! rebind whose registration fails, keeps none of it: a page staged for a
//! successor that never came must not land on a later registration, so the
//! facade retires Scroll's record of that page wherever this retires the
//! geometry it was staged in.

use crate::mounting::{UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration};

impl super::WorthUiMountedSessionState {
    /// Forget what `surface` would have carried to a successor binding.
    pub(super) fn retire_rebind_carry(&mut self, surface: UiSemanticSurfaceIdentity) {
        self.selection_bindings.retire_surface(surface);
        self.retire_occurrence_geometry_surface(surface);
    }

    /// Give up a rebind of `surface` from `prior` whose registration failed,
    /// leaving the surface unbound with no successor to carry anything to.
    pub(super) fn abandon_surface_rebind(
        &mut self,
        prior: UiSurfaceBindingGeneration,
        surface: UiSemanticSurfaceIdentity,
    ) {
        self.presentation.abandon_surface_rebind(prior);
        self.retire_rebind_carry(surface);
    }
}
