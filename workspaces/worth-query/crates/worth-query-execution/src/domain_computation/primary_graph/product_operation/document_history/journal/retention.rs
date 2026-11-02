//! Creation order controls eligible sources; annotations convey no retention right.
use super::*;
use worth_query_declaration::facade::application_history::ApplicationHistoryRecentSteps;

impl<S> HistoryJournal<'_, S> {
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn retained_source(
        &self,
        node: WorthQueryHistoryNodeId,
    ) -> Result<&Arc<WorthQueryApplicationReadObservation>, Denial> {
        if !self.owns_node(node) {
            return Err(Denial::ForeignHistoryNode);
        }
        if self.step(node).is_none() {
            return Err(Denial::HistoryNodeUnavailable);
        }
        self.retained
            .get(&node.ordinal)
            .and_then(|entry| entry.source.as_ref())
            .ok_or(Denial::HistoryNodeExcluded)
    }

    fn unprotected_count(&self) -> usize {
        let current_unbookmarked = usize::from(
            self.retained
                .get(&self.current.ordinal)
                .is_some_and(|entry| !entry.bookmarked),
        );
        self.retained_completed - self.bookmarked - current_unbookmarked
    }

    pub(super) fn trim_one(&mut self) -> Option<RetainedHistorySource> {
        let limit = self.pending_window.unwrap_or(self.window).get();
        if self.unprotected_count() <= limit {
            return None;
        }
        // Only current, bounded bookmarks and the one pre-effects reservation can
        // be skipped. Excluded annotations are absent from this index entirely.
        let oldest = self
            .retained
            .iter()
            .find_map(|(ordinal, entry)| {
                self.access(Access::RetentionFrontier);
                (*ordinal != self.current.ordinal && !entry.bookmarked && entry.source.is_some())
                    .then_some(*ordinal)
            })
            .expect("the count proves an eligible unprotected source");
        if let Some(slot) = self.nodes.get(&oldest).and_then(|node| node.head_slot) {
            self.queue_head(slot);
        }
        let mut removed = self
            .retained
            .remove(&oldest)
            .expect("oldest eligible source");
        if let Some(NodeBinding::Completed(node)) =
            self.nodes.get(&oldest).map(|node| &node.binding)
        {
            if let Some(parent) = node.step.parent() {
                removed.released_child = self.children.remove(&(parent.ordinal, oldest));
            }
        }
        self.retained_completed -= 1;
        Some(removed)
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn begin_window(
        &mut self,
        window: ApplicationHistoryRecentSteps,
    ) -> Result<(), Denial> {
        if self.pending_window == Some(window) {
            return Ok(());
        }
        self.ensure_policy_ready()?;
        if window == self.window {
            return Ok(());
        }
        if self.pending.is_some() {
            return Err(Denial::AttemptInFlight);
        }
        self.check_release_capacity(1)?;
        self.view_generation
            .checked_add(1)
            .ok_or(Denial::RecordingIdentityExhausted)?;
        self.pending_window = Some(window);
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn retention_status(
        &self,
    ) -> super::super::WorthQueryHistoryRetentionStatus {
        super::super::WorthQueryHistoryRetentionStatus {
            recent_steps: self.window,
            pending_window: self.pending_window,
            unprotected_steps: self.unprotected_count(),
            bookmarks: self.bookmarked,
            current: self.current,
            pending_releases: self.releases.len(),
        }
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn advance_window(
        &mut self,
    ) -> Result<Option<RetainedHistorySource>, Denial> {
        let Some(target) = self.pending_window else {
            return Ok(None);
        };
        let generation = self
            .view_generation
            .checked_add(1)
            .ok_or(Denial::RecordingIdentityExhausted)?;
        if self.unprotected_count() <= target.get() {
            self.window = target;
            self.pending_window = None;
            self.view_generation = generation;
            return Ok(None);
        }
        self.check_release_capacity(1)?;
        let removed = self.trim_one();
        self.view_generation = generation;
        if self.unprotected_count() <= target.get() {
            self.window = target;
            self.pending_window = None;
        }
        Ok(removed)
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn bookmark(
        &mut self,
        node: WorthQueryHistoryNodeId,
        pin: bool,
    ) -> Result<Option<RetainedHistorySource>, Denial> {
        self.ensure_policy_ready()?;
        if self.pending.is_some() {
            return Err(Denial::AttemptInFlight);
        }
        self.retained_source(node)?;
        let was_pinned = self
            .retained
            .get(&node.ordinal)
            .expect("eligible source")
            .bookmarked;
        if pin == was_pinned {
            return Ok(None);
        }
        if pin && self.bookmarked == self.retention_resources.maximum_bookmarks.get() {
            return Err(Denial::BookmarkCapacityExhausted);
        }
        if !pin {
            self.check_release_capacity(1)?;
        }
        let generation = self
            .view_generation
            .checked_add(1)
            .ok_or(Denial::RecordingIdentityExhausted)?;
        self.retained
            .get_mut(&node.ordinal)
            .expect("eligible source")
            .bookmarked = pin;
        if pin {
            self.bookmarked += 1;
        } else {
            self.bookmarked -= 1;
        }
        self.view_generation = generation;
        Ok(self.trim_one())
    }
}
