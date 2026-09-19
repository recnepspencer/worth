use crate::facade::prepared_application_authority::{
    WorthUiPreparedApplicationGenerationIdentity, WorthUiPreparedApplicationGraphSuccessor,
};

use super::{UiPortalOverlayBindingLifecycle, UiPortalOverlayBindingLifecycleDenial};

/// A pending mutation of this live binding owner, admitted from a graph-only
/// successor. Source replacement cannot construct this state from identities.
pub(crate) struct UiPreparedPortalOverlayGraphSuccession {
    predecessor: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    successor: WorthUiPreparedApplicationGenerationIdentity,
}

impl UiPortalOverlayBindingLifecycle {
    /// Graph-only succession retains the prepared declaration bindings. Source
    /// replacement must use its separate declaration admission and lifecycle.
    pub(crate) fn prepare_graph_succession(
        &self,
        successor: &WorthUiPreparedApplicationGraphSuccessor,
    ) -> Result<UiPreparedPortalOverlayGraphSuccession, UiPortalOverlayBindingLifecycleDenial> {
        let succession = successor.generation_succession();
        if self.generation.prepared_generation() != succession.predecessor()
            || self
                .owners
                .values()
                .any(|owner| owner.generation() != succession.predecessor())
        {
            return Err(UiPortalOverlayBindingLifecycleDenial::ForeignGeneration);
        }
        Ok(UiPreparedPortalOverlayGraphSuccession {
            predecessor: self.generation.clone(),
            successor: succession.successor().clone(),
        })
    }

    pub(crate) fn commit_graph_succession(
        &mut self,
        succession: UiPreparedPortalOverlayGraphSuccession,
    ) {
        assert_eq!(self.generation, succession.predecessor);
        for owner in self.owners.values_mut() {
            owner.commit_graph_generation_succession(&succession);
        }
        self.generation = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            self.generation.session_identity(),
            &succession.successor,
        );
    }
}

impl UiPreparedPortalOverlayGraphSuccession {
    pub(in crate::runtime::portal) fn successor(
        &self,
    ) -> &WorthUiPreparedApplicationGenerationIdentity {
        &self.successor
    }
}
