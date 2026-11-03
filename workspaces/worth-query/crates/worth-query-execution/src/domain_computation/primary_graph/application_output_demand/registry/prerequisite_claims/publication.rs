//! Prepared claims transfer into the exact published downstream row.

use super::*;

impl PreparedPrerequisiteClaims {
    /// The lineage owner has already reserved and minted this exact address.
    /// Insert its invisible registry vacancy before any World owner effect.
    pub(in crate::domain_computation::primary_graph) fn reserve_identity(
        &mut self,
        identity: &Arc<RecordedSettlementIdentity>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        assert!(self.reserved_identity.is_none());
        let owner = WorthQueryOutputDemandRegistry::clone(self.context.registry());
        let (posting, cleanup) = owner.reserve_settlement_vacancy(identity)?;
        self.reserved_identity = Some(Arc::clone(identity));
        self.reserved_posting = Some(posting);
        self.reserved_cleanup = Some(cleanup);
        Ok(())
    }

    /// Prepare the successor's invisible posting while the prior posting is
    /// still held. A denial leaves every old token with this ticket; after a
    /// successful reserve, exchanging and queueing the old cue cannot fail.
    pub(in crate::domain_computation::primary_graph) fn replace_reserved_identity_for_recovery(
        &mut self,
        identity: &Arc<RecordedSettlementIdentity>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let owner = WorthQueryOutputDemandRegistry::clone(self.context.registry());
        let (posting, cleanup) = owner.reserve_settlement_vacancy(identity)?;
        let mut state = owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let previous = self.reserved_cleanup.replace(cleanup);
        self.reserved_posting = Some(posting);
        self.reserved_identity = Some(Arc::clone(identity));
        if let Some(previous) = previous {
            state.defer_cancelled_settlement_vacancy(previous);
        }
        Ok(())
    }

    /// This is called only after the product publication has committed. All
    /// allocations and index capacity were admitted by `prepare_prerequisites`.
    pub(in crate::domain_computation::primary_graph) fn publish(
        mut self,
        identity: Arc<RecordedSettlementIdentity>,
    ) -> super::super::SupersededSettlements {
        let owner = WorthQueryOutputDemandRegistry::clone(self.context.registry());
        let mut state = owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let downstream = self.context.key();
        let expected = self
            .reserved_identity
            .take()
            .expect("managed output reserved its exact settlement before effects");
        super::super::settlement_index::SettlementIndex::fill_prepared(
            self.reserved_posting
                .as_ref()
                .expect("prepared settlement posting remains retained"),
            expected.as_ref(),
            Arc::clone(&identity),
            Arc::clone(self.context.key_arc()),
        );
        let mut record = state
            .records
            .get_mut(downstream)
            .expect("prepared demand stays pinned");
        assert_eq!(record.prepared_prerequisite_claims, 1);
        record.prepared_prerequisite_claims = 0;
        assert!(record.settlements.len() < record.settlements.capacity());
        record.settlements.push(Arc::clone(&identity));
        crate::domain_computation::primary_graph::fact_verification_observation::registry_custody(
            crate::domain_computation::primary_graph::WorthQueryRegistryCustodyTransitionForTest::SettlementPublished,
            record.settlements.len(), self.predecessors.len(),
        );
        if let Some(batch) = &self.claim_batch {
            batch.publish();
        }
        let old_batch = std::mem::replace(&mut record.claim_batch, self.claim_batch.take());
        if let Some(batch) = &old_batch {
            batch.deactivate();
        }
        let old = std::mem::replace(
            &mut record.prerequisites,
            std::mem::take(&mut self.predecessors),
        );
        let old_checkpoint = std::mem::replace(
            &mut record.checkpoint_prerequisites,
            self.checkpoint_predecessors.take(),
        );
        record.finish();

        for upstream in &old {
            let mut prior = state
                .records
                .get_mut(upstream.as_ref())
                .expect("settled prerequisite retained");
            prior.framework_required_count -= 1;
            prior.finish();
            if !state.records[upstream].pending_cleanup_queued
                && !state.defer_terminal_cleanup(upstream)
            {
                super::super::post_commit::release_published_member(&mut state, upstream);
            }
        }
        for upstream in &old {
            state.defer_terminal_cleanup(upstream);
        }
        state.defer_terminal_cleanup(self.context.key_arc());

        // The context's owned key is now held by the exact-settlement index.

        self.reserved_cleanup.take();
        self.published = true;
        drop(state);
        drop(old);
        drop(old_batch);
        drop(old_checkpoint);
        super::super::SupersededSettlements::new(owner, Arc::clone(self.context.key_arc()))
    }
}
