//! Ordinary edit reservation and exact completed binding use prepaid slots.
use super::super::head_custody::HistoryHeadCustody;
use super::*;
use worth_query_execution_query::facade::ports::HistoryNodeIdInternalPort as _;

impl HistoryJournal {
    /// Refuse step metadata above the installed bound before anything is reserved.
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn admit_step_metadata(
        &self,
        metadata: &WorthQueryHistoryStepMetadata,
    ) -> Result<(), Denial> {
        let length = metadata.bytes().len();
        let maximum = self.retention_resources.maximum_step_metadata_bytes;
        if length > maximum {
            return Err(Denial::StepMetadataTooLarge { length, maximum });
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn reserve(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        meaning: WorthQueryApplicationIdempotencyBinding,
        label: &str,
        metadata: &WorthQueryHistoryStepMetadata,
    ) -> Result<ReservedHistoryRecord, Denial> {
        self.access(Access::Selection);
        if self.pending.is_some() {
            return Err(Denial::AttemptInFlight);
        }
        self.ensure_policy_ready()?;
        self.check_release_capacity(1)?;
        self.quote_frontier(execution)?;
        let node = self.incarnation.node(self.next);
        let next = self
            .next
            .checked_add(1)
            .ok_or(Denial::RecordingIdentityExhausted)?;
        let key = meaning.canonical_key();
        let view_after = self
            .view_generation
            .checked_add(1)
            .ok_or(Denial::RecordingIdentityExhausted)?;
        // Every real tree slot is inserted before any candidate creation or operation effect.
        // Failure while acquiring the grants drops only new grants, leaving the journal intact.
        let described = label
            .len()
            .checked_add(metadata.bytes().len())
            .ok_or(Denial::RecordingIdentityExhausted)?;
        let node_allocation = self
            .allocate::<(u64, HistoryNode)>(execution, self.nodes.len(), described)
            .map_err(Denial::Recording)?;
        let attempt_allocation = self
            .allocate::<([u8; 32], HistoryAttempt)>(execution, self.attempts.len(), 0)
            .map_err(Denial::Recording)?;
        let source_allocation = self
            .allocate::<(u64, RetainedHistorySource)>(execution, self.retained.len(), 0)
            .map_err(Denial::Recording)?;
        let link_allocation = self
            .allocate::<((u64, u64), ChildLink)>(execution, self.children.len(), 0)
            .map_err(Denial::Recording)?;
        let label = Arc::from(label);
        let transition = self.reserve_head(execution)?;
        self.access(Access::Attempt);
        assert!(
            !self.attempts.contains_key(&key),
            "only an unseen attempt reserves storage"
        );
        let line = self.child_position(self.current.ordinal());
        self.access(Access::Node);
        self.nodes.insert(
            node.ordinal(),
            HistoryNode {
                binding: NodeBinding::Reserved {
                    parent: self.current,
                    label,
                    metadata: metadata.clone(),
                },
                line,
                head_slot: None,
                _allocation: node_allocation,
            },
        );
        self.access(Access::Attempt);
        self.attempts.insert(
            key,
            HistoryAttempt {
                meaning,
                binding: AttemptBinding::Reserved(node),
                _allocation: attempt_allocation,
            },
        );
        self.retained.insert(
            node.ordinal(),
            RetainedHistorySource {
                source: None,
                discovery_recipes: None,
                bookmarked: false,
                _allocation: source_allocation,
                released_child: None,
            },
        );
        self.access(Access::Child);
        self.children.insert(
            (self.current.ordinal(), node.ordinal()),
            ChildLink {
                _allocation: link_allocation,
            },
        );
        self.next = next;
        self.pending = Some(PendingRecord::Ordinary(node));
        Ok(ReservedHistoryRecord {
            node,
            parent: self.current,
            transition,
            key,
            view_after,
        })
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn cancel(
        &mut self,
        reserved: ReservedHistoryRecord,
    ) {
        self.cancel_head(reserved.transition);
        self.release_record(&reserved);
    }

    /// Cancel a reservation whose private candidate did not close. The prepaid
    /// head slot keeps that exact live head, close owner or cleanup owner, so
    /// retention maintenance and final disposal retry it like any other past head.
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn cancel_parking(
        &mut self,
        reserved: ReservedHistoryRecord,
        custody: HistoryHeadCustody,
    ) {
        debug_assert!(matches!(
            custody,
            HistoryHeadCustody::Live(_)
                | HistoryHeadCustody::Refused { .. }
                | HistoryHeadCustody::Retired { .. }
        ));
        self.access(Access::Custody);
        let slot = self
            .past_heads
            .get(&reserved.transition)
            .expect("original reserved head slot");
        let previous = std::mem::replace(&mut *slot.custody.lock(), custody);
        assert!(
            matches!(previous, HistoryHeadCustody::Reserved),
            "only an unused reserved slot can park a failed candidate"
        );
        self.queue_head(reserved.transition);
        self.release_record(&reserved);
    }

    fn release_record(&mut self, reserved: &ReservedHistoryRecord) {
        self.release_pending_source(reserved.node.ordinal());
        self.access(Access::Node);
        self.nodes.remove(&reserved.node.ordinal());
        self.access(Access::Attempt);
        self.attempts.remove(&reserved.key);
        self.access(Access::Child);
        self.children
            .remove(&(reserved.parent.ordinal(), reserved.node.ordinal()));
        let source = self
            .retained
            .remove(&reserved.node.ordinal())
            .expect("prepaid source slot");
        assert!(
            source.source.is_none(),
            "a completed source cannot be cancelled"
        );
        self.access(Access::Selection);
        self.pending = None;
    }

    /// All lookup slots already exist. Invoke only while the same exclusive
    /// document owner performs its genuine head adoption; this cannot allocate.
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn complete(
        &mut self,
        reserved: ReservedHistoryRecord,
        completed: RecordedNode,
        source: Arc<WorthQueryApplicationReadObservation>,
        discovery_recipes: Option<Arc<super::super::discovery_recipes::HistoryDiscoveryRecipes>>,
        previous: DetachedOwnedProductHead,
    ) -> Option<RetainedHistorySource> {
        self.store_previous(reserved.transition, previous);
        self.access(Access::Node);
        self.nodes
            .get_mut(&reserved.node.ordinal())
            .expect("reserved node")
            .binding = NodeBinding::Completed(completed);
        self.access(Access::Attempt);
        self.attempts
            .get_mut(&reserved.key)
            .expect("reserved attempt")
            .binding = AttemptBinding::Completed(reserved.node);
        let retained = self
            .retained
            .get_mut(&reserved.node.ordinal())
            .expect("prepaid source destination");
        retained.source = Some(source);
        retained.discovery_recipes = discovery_recipes;
        self.retained_completed += 1;
        self.access(Access::Selection);
        self.current = reserved.node;
        self.completed_end = reserved.node.ordinal() + 1; // Successor was admitted by reserve.
        self.view_generation = reserved.view_after;
        self.pending = None;
        self.pending_source = None;
        self.pending_endpoint = None;
        self.trim_one()
    }
}
