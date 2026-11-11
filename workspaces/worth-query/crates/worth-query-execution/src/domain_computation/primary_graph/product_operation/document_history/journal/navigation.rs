//! Navigation selects an existing occurrence without rewriting or appending its source.
use super::*;

impl<S> HistoryJournal<'_, S> {
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn resolve_navigation(
        &self,
        intent: super::super::navigation::WorthQueryHistoryNavigationIntent,
    ) -> Result<WorthQueryHistoryNodeId, super::super::navigation::WorthQueryHistoryNavigationDenial>
    {
        use super::super::navigation::{
            WorthQueryHistoryNavigationDenial as Failure,
            WorthQueryHistoryNavigationIntent as Intent,
        };
        match intent {
            Intent::Checkout(node) => Ok(node),
            Intent::UndoStep => self.current_step().parent().ok_or(Failure::NoParent),
            Intent::RedoTo(node) => {
                if !self.owns_node(node) {
                    return Err(Failure::Document(Denial::ForeignHistoryNode));
                }
                let step = self
                    .step(node)
                    .ok_or(Failure::Document(Denial::HistoryNodeUnavailable))?;
                if step.parent() != Some(self.current) {
                    return Err(Failure::NotChild);
                }
                Ok(node)
            }
        }
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn prepare_navigation(
        &self,
        node: WorthQueryHistoryNodeId,
    ) -> Result<u64, Denial> {
        self.ensure_policy_ready()?;
        if matches!(self.pending, Some(PendingRecord::Ordinary(_))) {
            return Err(Denial::AttemptInFlight);
        }
        if !self.owns_node(node) {
            return Err(Denial::ForeignHistoryNode);
        }
        self.retained_source(node)?;
        self.check_navigation_release_capacity()?;
        self.view_generation
            .checked_add(1)
            .ok_or(Denial::RecordingIdentityExhausted)
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn complete_navigation(
        &mut self,
        node: WorthQueryHistoryNodeId,
        view_after: u64,
    ) -> Option<RetainedHistorySource> {
        // The caller holds the actual selection guard and rechecked its original
        // physical selection. Target and generation were admitted before effects.
        debug_assert!(self.step(node).is_some());
        debug_assert_eq!(self.view_generation.checked_add(1), Some(view_after));
        self.current = node;
        self.view_generation = view_after;
        self.protect_head(node.ordinal);
        self.trim_one()
    }
}

pub(in crate::domain_computation::primary_graph::product_operation::document_history) struct NavigationSourceReservation
{
    source: bool,
    child: Option<(u64, u64)>,
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) view_after:
        u64,
}

impl<S> HistoryJournal<'_, S> {
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn reserve_navigation_source(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        node: WorthQueryHistoryNodeId,
    ) -> Result<NavigationSourceReservation, Denial> {
        if matches!(self.pending, Some(PendingRecord::Ordinary(_))) {
            return Err(Denial::AttemptInFlight);
        }
        self.check_navigation_release_capacity()?;
        let parent = self
            .step(node)
            .ok_or(Denial::HistoryNodeUnavailable)?
            .parent();
        self.quote_frontier(execution)?;
        let view_after = self
            .view_generation
            .checked_add(1)
            .ok_or(Denial::RecordingIdentityExhausted)?;
        let source = !self.retained.contains_key(&node.ordinal);
        let child = parent
            .map(|parent| (parent.ordinal, node.ordinal))
            .filter(|key| !self.children.contains_key(key));
        let source_memory = if source {
            Some(
                self.allocate::<(u64, RetainedHistorySource)>(execution, self.retained.len(), 0)
                    .map_err(Denial::Recording)?,
            )
        } else {
            None
        };
        let child_memory = if child.is_some() {
            Some(
                self.allocate::<((u64, u64), ChildLink)>(execution, self.children.len(), 0)
                    .map_err(Denial::Recording)?,
            )
        } else {
            None
        };
        if let Some(allocation) = source_memory {
            self.retained.insert(
                node.ordinal,
                RetainedHistorySource {
                    source: None,
                    discovery_recipes: None,
                    bookmarked: false,
                    _allocation: allocation,
                    released_child: None,
                },
            );
        }
        if let Some((key, allocation)) = child.zip(child_memory) {
            self.children.insert(
                key,
                ChildLink {
                    _allocation: allocation,
                },
            );
        }
        Ok(NavigationSourceReservation {
            source,
            child,
            view_after,
        })
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn cancel_navigation_source(
        &mut self,
        node: WorthQueryHistoryNodeId,
        reserved: NavigationSourceReservation,
    ) {
        if reserved.source {
            let entry = self
                .retained
                .remove(&node.ordinal)
                .expect("prepaid restoration");
            assert!(entry.source.is_none());
        }
        if let Some(child) = reserved.child {
            self.children.remove(&child);
        }
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn fill_navigation_source(
        &mut self,
        node: WorthQueryHistoryNodeId,
        source: Arc<WorthQueryApplicationReadObservation>,
        discovery_recipes: Option<Arc<super::super::discovery_recipes::HistoryDiscoveryRecipes>>,
    ) -> Option<Arc<WorthQueryApplicationReadObservation>> {
        let entry = self
            .retained
            .get_mut(&node.ordinal)
            .expect("prepaid target source");
        if entry.source.is_none() {
            entry.source = Some(source);
            entry.discovery_recipes = discovery_recipes;
            self.retained_completed += 1;
            None
        } else {
            Some(source)
        }
    }
}
