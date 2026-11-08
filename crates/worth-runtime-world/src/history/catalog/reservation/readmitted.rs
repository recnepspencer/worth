//! Original recovered publication installation shares the reserved catalog owner.
use super::*;
use crate::history::{
    NativeReadmittedPublicationRecord, ReadmittedPublicationRecord, WorldReadmittedPublication,
};
use crate::retention::HistoryRetentionObligation;

impl ReservedCompositeCommitCapacity {
    fn validate_readmitted(
        &self,
        state: &CompositeHistoryCatalogState,
        commit: &Arc<CompositeRuntimeWorldCommit>,
        pins: &HistoryRetentionObligation,
        record: &Arc<ReadmittedPublicationRecord>,
    ) -> Result<(), CompositeHistoryCatalogDenial> {
        match self.publication {
            ReservedHistoryPublication::Readmitted => {}
            ReservedHistoryPublication::NoPublication | ReservedHistoryPublication::Live(_) => {
                return Err(CompositeHistoryCatalogDenial::ReservationCommitMismatch)
            }
        }
        self.validate_installation(state, commit, pins)?;
        if !Arc::ptr_eq(&record.after.shared_commit(), commit)
            || commit.provenance()
                != &crate::history::CompositeCommitProvenance::Publication(record.attempt.clone())
        {
            return Err(CompositeHistoryCatalogDenial::ReservationCommitMismatch);
        }
        match commit.parent() {
            crate::history::CompositeCommitParent::Ordinary(parent)
                if parent.commit() == record.before.selected_commit() =>
            {
                Ok(())
            }
            crate::history::CompositeCommitParent::Ordinary(_)
            | crate::history::CompositeCommitParent::Root => {
                Err(CompositeHistoryCatalogDenial::ReservationParentMismatch)
            }
        }
    }

    /// Cold construction installs before its not-yet-exposed branch cells.
    pub(crate) fn install_readmitted(
        &mut self,
        commit: Arc<CompositeRuntimeWorldCommit>,
        pins: HistoryRetentionObligation,
        record: Arc<ReadmittedPublicationRecord>,
    ) -> Result<(), CompositeHistoryCatalogDenial> {
        let mut state = lock_state(&self.state);
        self.validate_readmitted(&state, &commit, &pins, &record)?;
        promote_reserved_commit(
            &mut state,
            &self.identity,
            self.slots.as_ref().expect("live reserved slots"),
            commit,
            Some(crate::history::PublicationEvidence::Readmitted(record)),
            pins,
        );
        self.armed = false;
        self.slots = None;
        Ok(())
    }

    /// Install head and caller protection atomically. Every refusal leaves the
    /// original reservation, record and pin value in its receiver-owned slots.
    pub(crate) fn install_readmitted_product_head(
        &mut self,
        history: &crate::history::CompositeHistoryCatalog,
        commit: &Arc<CompositeRuntimeWorldCommit>,
        pins: &mut Option<HistoryRetentionObligation>,
        record: &NativeReadmittedPublicationRecord,
    ) -> Result<
        (
            crate::history::ProductHeadHistoryProtectionObligation,
            WorldReadmittedPublication,
        ),
        CompositeHistoryCatalogDenial,
    > {
        use crate::history::retention::{
            CompositeHistoryProtectionObligation, HistoryProtectionClass,
            ProductHeadHistoryProtectionObligation,
        };
        if !Arc::ptr_eq(&self.state, &history.state) {
            return Err(CompositeHistoryCatalogDenial::ReservationCommitMismatch);
        }
        let mut state = lock_state(&self.state);
        let original = pins
            .as_ref()
            .ok_or(CompositeHistoryCatalogDenial::HistoryPinBasisMismatch)?;
        self.validate_readmitted(&state, commit, original, record.record())?;
        let identity = self.identity.clone();
        let reachability = state.reachability.clone();
        let slot = self.slots.as_ref().expect("same live reserved slots");
        promote_reserved_commit(
            &mut state,
            &self.identity,
            slot,
            commit.clone(),
            Some(crate::history::PublicationEvidence::Readmitted(
                record.record().clone(),
            )),
            pins.take().expect("validated original pins"),
        );
        {
            let mut index = super::super::lock_index(&reachability);
            index.protect_reserved_slot(&slot.reachability);
            index
                .increment_reserved_protection(&slot.reachability, &identity)
                .expect("one new head has room for its prepaid original caller");
        }
        self.armed = false;
        self.slots = None;
        let head = ProductHeadHistoryProtectionObligation::issued(
            CompositeHistoryProtectionObligation::new(
                reachability.clone(),
                identity.clone(),
                HistoryProtectionClass::ProductHead,
            ),
        );
        let publication = WorldReadmittedPublication::from_native(
            record.retained(),
            CompositeHistoryProtectionObligation::new(
                reachability,
                identity,
                HistoryProtectionClass::ExplicitObligation,
            ),
            history.clone(),
        );
        Ok((head, publication))
    }
}
