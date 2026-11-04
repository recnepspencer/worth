//! Atomically install exact reserved history and its first head protection.
use super::*;
impl ReservedCompositeCommitCapacity {
    pub(crate) fn try_install_product_head(
        &mut self,
        commit: Arc<CompositeRuntimeWorldCommit>,
        pins: crate::retention::HistoryRetentionObligation,
    ) -> Result<crate::history::ProductHeadHistoryProtectionObligation, CompositeHistoryCatalogDenial>
    {
        use crate::history::retention::{
            CompositeHistoryProtectionObligation, HistoryProtectionClass,
            ProductHeadHistoryProtectionObligation,
        };
        let mut state = lock_state(&self.state);
        self.validate_installation(&state, &commit, &pins)?;
        let identity = self.identity.clone();
        let reachability = Arc::clone(&state.reachability);
        promote_reserved_commit(
            &mut state,
            &self.identity,
            self.slots.as_ref().expect("live reserved slots"),
            commit,
            self.publication
                .clone()
                .map(crate::history::PublicationEvidence::Live),
            pins,
        );
        super::super::lock_index(&reachability).protect_reserved_slot(
            &self
                .slots
                .as_ref()
                .expect("live reserved slots")
                .reachability,
        );
        self.armed = false;
        self.slots = None;
        Ok(ProductHeadHistoryProtectionObligation::issued(
            CompositeHistoryProtectionObligation::new(
                reachability,
                identity,
                HistoryProtectionClass::ProductHead,
            ),
        ))
    }

    pub(crate) fn try_install_readmitted_product_head(
        &mut self,
        commit: Arc<CompositeRuntimeWorldCommit>,
        pins: crate::retention::HistoryRetentionObligation,
        record: Arc<crate::history::ReadmittedPublicationRecord>,
    ) -> Result<crate::history::ProductHeadHistoryProtectionObligation, CompositeHistoryCatalogDenial>
    {
        use crate::history::retention::{
            CompositeHistoryProtectionObligation, HistoryProtectionClass,
            ProductHeadHistoryProtectionObligation,
        };
        let mut state = lock_state(&self.state);
        self.validate_installation(&state, &commit, &pins)?;
        let identity = self.identity.clone();
        let reachability = Arc::clone(&state.reachability);
        promote_reserved_commit(
            &mut state,
            &self.identity,
            self.slots.as_ref().expect("live reserved slots"),
            commit,
            Some(crate::history::PublicationEvidence::Readmitted(record)),
            pins,
        );
        super::super::lock_index(&reachability).protect_reserved_slot(
            &self
                .slots
                .as_ref()
                .expect("live reserved slots")
                .reachability,
        );
        self.armed = false;
        self.slots = None;
        Ok(ProductHeadHistoryProtectionObligation::issued(
            CompositeHistoryProtectionObligation::new(
                reachability,
                identity,
                HistoryProtectionClass::ProductHead,
            ),
        ))
    }
}
