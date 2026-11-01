//! Prepaid actual head custody and the bounded retirement queue.
use super::super::head_custody::HistoryHeadCustody;
use super::*;

impl<'r, S> HistoryJournal<'r, S> {
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn ensure_policy_ready(
        &self,
    ) -> Result<(), Denial> {
        if self.pending_window.is_some() {
            Err(Denial::RetentionMaintenancePending)
        } else {
            Ok(())
        }
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn check_release_capacity(
        &self,
        needed: usize,
    ) -> Result<(), Denial> {
        let reserved_for_navigation = if self.navigation_sources.has_unfinished() {
            2
        } else {
            0
        };
        self.check_release_room(needed.saturating_add(reserved_for_navigation))
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn check_navigation_release_capacity(
        &self,
    ) -> Result<(), Denial> {
        // Competing attempts share the same expected physical selection. Only
        // its actual guard winner may consume these two retirement destinations.
        self.check_release_room(2)
    }

    fn check_release_room(&self, needed: usize) -> Result<(), Denial> {
        if self
            .retention_resources
            .maximum_pending_releases
            .get()
            .saturating_sub(self.releases.len())
            < needed
        {
            Err(Denial::RetentionReleaseCapacityExhausted)
        } else {
            Ok(())
        }
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn reserve_head(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<u64, Denial> {
        let slot = self.next_transition;
        let next = slot
            .checked_add(1)
            .ok_or(Denial::RecordingIdentityExhausted)?;
        let allocation = self
            .allocate::<(u64, PastHead<'r, S>)>(execution, self.past_heads.len(), 0)
            .map_err(Denial::Recording)?;
        self.access(Access::Custody);
        let custody = super::super::head_custody::HistoryHeadSlot::reserve(execution)?;
        self.past_heads.insert(
            slot,
            PastHead {
                custody,
                queued: false,
                _allocation: allocation,
            },
        );
        self.next_transition = next;
        Ok(slot)
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn cancel_head(
        &mut self,
        slot: u64,
    ) {
        self.access(Access::Custody);
        assert!(
            matches!(
                *self
                    .past_heads
                    .get(&slot)
                    .expect("original reserved head slot")
                    .custody
                    .lock(),
                HistoryHeadCustody::Reserved
            ),
            "a real adopted head cannot be cancelled"
        );
        self.past_heads.remove(&slot);
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn store_previous(
        &mut self,
        slot: u64,
        previous: WorthQueryOwnedProductHead<'r, S>,
    ) {
        let canonical = self.current_step().endpoint().product_branch() == previous.branch();
        if canonical {
            let node = self
                .nodes
                .get_mut(&self.current.ordinal)
                .expect("current completed node");
            debug_assert!(node.head_slot.is_none());
            node.head_slot = Some(slot);
        }
        let entry = self
            .past_heads
            .get_mut(&slot)
            .expect("prepaid actual head destination");
        debug_assert!(matches!(
            *entry.custody.lock(),
            HistoryHeadCustody::Reserved
        ));
        *entry.custody.lock() = HistoryHeadCustody::Live(previous);
        if !canonical {
            self.queue_head(slot);
        }
    }

    pub(super) fn queue_head(&mut self, slot: u64) {
        let head = self
            .past_heads
            .get_mut(&slot)
            .expect("attributed head remains owned");
        if !head.queued {
            assert!(
                self.releases.len() < self.retention_resources.maximum_pending_releases.get(),
                "retirement capacity was admitted before effects"
            );
            head.queued = true;
            self.releases.push_back(slot);
        }
    }

    pub(super) fn protect_head(&mut self, node: u64) {
        if let Some(slot) = self.nodes.get(&node).and_then(|node| node.head_slot) {
            if let Some(head) = self.past_heads.get_mut(&slot) {
                debug_assert!(
                    !matches!(*head.custody.lock(), HistoryHeadCustody::Retired { .. }),
                    "an unfinished original target keeps H live until selection"
                );
                head.queued = false;
                self.releases.retain(|queued| *queued != slot);
            }
        }
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn next_release(
        &mut self,
    ) -> Option<(u64, Arc<super::super::head_custody::HistoryHeadSlot<'r, S>>)> {
        let slot = *self.releases.front()?;
        self.releases.rotate_left(1);
        let head = self
            .past_heads
            .get(&slot)
            .expect("queued custody is registered");
        if head
            .custody
            .lock()
            .branch()
            .is_some_and(|branch| self.navigation_sources.protects(branch))
        {
            return None;
        }
        Some((slot, Arc::clone(&head.custody)))
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn finish_release(
        &mut self,
        slot: u64,
        original: &Arc<super::super::head_custody::HistoryHeadSlot<'r, S>>,
    ) -> bool {
        let Some(registered) = self.past_heads.get(&slot) else {
            return false;
        };
        if !Arc::ptr_eq(&registered.custody, original)
            || !matches!(*original.lock(), HistoryHeadCustody::Closed(_))
        {
            return false;
        }
        self.past_heads.remove(&slot).expect("exact completed slot");
        self.releases.retain(|queued| *queued != slot);
        self.closed_heads += 1;
        true
    }
}
