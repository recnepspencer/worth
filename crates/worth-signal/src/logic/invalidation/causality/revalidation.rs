mod cause_validation;
use crate::data::error::SignalError;
#[cfg(test)]
use crate::data::graph::storage::invalidation_causes::PendingCauseSetId;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::ResolvedDependencyCause;

use crate::data::graph::SignalGraph;

impl SignalGraph {
    pub(crate) fn node_invalidation_input_with_execution_work(
        &self,
        node: NodeId,
        work: Option<&mut worth_execution::MapKernelContext<'_, '_>>,
        preparation: Option<&mut crate::data::request_preparation::SignalPreparationBudget>,
    ) -> Result<crate::data::proof::invalidation::revalidation::NodeInvalidationInput, SignalError>
    {
        if let Some(work) = work {
            // Inspect borrowed owner storage before any cloned cause/scope payload.
            work.checkpoint(1)
                .map_err(|_| SignalError::invalid_input("invalidation input admission stopped"))?;
            let causes = self.pending_causes(node)?;
            let scopes = self.node_dirty_partition_scope_payload(node)?;
            let mut units = (causes.len() as u64)
                .saturating_mul(causes.len().checked_ilog2().unwrap_or(0) as u64 + 32);
            let scope_work = |scope: &crate::data::output::PartitionSubscription| {
                (scope.path().total_segment_bytes() as u64)
                    .saturating_add(scope.path().depth() as u64)
                    .saturating_add(1)
            };
            for (_, scope) in scopes {
                units = units.saturating_add(scope_work(scope).saturating_mul(3));
            }
            for cause in causes {
                for scope in [&cause.key.edge_scope, &cause.binding_axes.edge_scope]
                    .into_iter()
                    .flatten()
                {
                    units = units.saturating_add(scope_work(scope));
                }
                for scope in cause.changed_scopes.as_slice() {
                    units = units.saturating_add(scope_work(scope).saturating_mul(2));
                }
            }
            if let Some(pending) = self.node_pending_revalidation(node)? {
                units = units.saturating_add(pending.unresolved_producers().len() as u64);
            }
            work.checkpoint(units).map_err(|_| {
                SignalError::invalid_input("invalidation input stopped before copying authority")
            })?;
            if let Some(budget) = preparation {
                use crate::data::retained_storage::{
                    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
                    RetainedStoragePreparation,
                };
                let mut measurement = RetainedStoragePreparation::new(usize::MAX);
                let bytes = (|| {
                    let mut bytes = Charge::capacity::<ResolvedDependencyCause>(causes.len())?;
                    for cause in causes {
                        bytes = bytes.checked_add(cause.retained_heap_charge(&mut measurement)?)?;
                    }
                    bytes = bytes.checked_add(Charge::capacity::<(crate::data::aspect::Aspect,
                        crate::data::output::PartitionSubscription)>(scopes.len())?)?;
                    for scoped in scopes {
                        bytes = bytes.checked_add(scoped.retained_heap_charge(&mut measurement)?)?;
                    }
                    if let Some(pending) = self.node_pending_revalidation(node)
                        .map_err(|_| crate::data::retained_storage::RetainedStoragePreparationDenial::ChargeOverflow)? {
                        bytes = bytes.checked_add(Charge::capacity::<NodeId>(pending.unresolved_producers().len())?)?;
                    }
                    // Canonicalization retains both the causes and their derived
                    // scoped cache; direct-source reconstruction also copies it.
                    bytes.checked_mul(3)
                })().map_err(|_| SignalError::invalid_input("invalidation input memory overflow"))?;
                budget.claim(bytes.bytes())?;
            }
        }
        self.node_invalidation_input(node)
    }

    pub(crate) fn node_invalidation_input(
        &self,
        node: NodeId,
    ) -> Result<crate::data::proof::invalidation::revalidation::NodeInvalidationInput, SignalError>
    {
        use crate::data::proof::invalidation::revalidation::{
            CanonicalDependencyCauseSet, NodeInvalidationInput, ResolvedDependencyBasis,
        };
        self.ensure_cause_readmission_complete()?;
        self.validate_direct_invalidation_storage(node)?;
        let revision = self.dependency_revision(node)?;
        if let Some(pending) = self.pending_dependency_revalidation(node)? {
            if pending.dependency_revision() != revision {
                return Err(SignalError::invalid_input(
                    "pending dependency revalidation belongs to a stale dependency revision",
                ));
            }
            if !pending.is_resolved() {
                return Ok(NodeInvalidationInput::Pending(pending));
            }
            if pending.requires_structural_recompute() {
                let pending_causes = self.pending_causes(node)?.to_vec();
                let causes = if pending_causes.is_empty() {
                    CanonicalDependencyCauseSet::structural(revision)
                } else {
                    self.resolved_dependency_causes(node, pending_causes)?
                };
                debug_assert!(causes.is_bound_to_revision(revision));
                return Ok(NodeInvalidationInput::Resolved(causes));
            }
        }
        let causes = self.pending_causes(node)?.to_vec();
        let dirty_aspects = self.node_dirty_aspects(node)?;
        if let Some(direct) = self.node_direct_invalidation_basis(node)? {
            if !causes.is_empty() {
                return Err(SignalError::invalid_input(
                    "direct invalidation basis cannot coexist with dependency causes",
                ));
            }
            let cached_scoped_aspects = self.node_dirty_partition_scope_payload(node)?.to_vec();
            if dirty_aspects != direct.dirty_aspects()
                || cached_scoped_aspects.as_slice() != direct.scoped_aspects()
            {
                return Err(SignalError::invalid_input(
                    "dirty mask or scope cache drifted from direct invalidation basis",
                ));
            }
            let resolved = CanonicalDependencyCauseSet::from_source_recompute(
                revision,
                direct.generation(),
                direct.dirty_aspects(),
                direct.scoped_aspects().to_vec(),
            );
            return Ok(NodeInvalidationInput::Resolved(resolved));
        }
        if causes.is_empty() && dirty_aspects.is_empty() {
            let basis = ResolvedDependencyBasis::new(revision, revision.0);
            debug_assert!(basis.is_bound_to_revision(revision));
            return Ok(NodeInvalidationInput::ResolvedNoChange(basis));
        }
        if causes.is_empty() {
            return Err(SignalError::invalid_input(
                "dirty cache has no direct invalidation or dependency-cause basis",
            ));
        }
        let causes = self.resolved_dependency_causes(node, causes)?;
        debug_assert!(causes.is_bound_to_revision(revision));
        Ok(NodeInvalidationInput::Resolved(causes))
    }

    fn resolved_dependency_causes(
        &self,
        node: NodeId,
        causes: Vec<ResolvedDependencyCause>,
    ) -> Result<
        crate::data::proof::invalidation::revalidation::CanonicalDependencyCauseSet,
        SignalError,
    > {
        use crate::data::proof::invalidation::revalidation::CanonicalDependencyCauseSet;
        let resolved = CanonicalDependencyCauseSet::from_dependency_causes(causes);
        let cached_aspects = self.node_dirty_aspects(node)?;
        let cached_scoped_aspects = self.node_dirty_partition_scope_payload(node)?.to_vec();
        if cached_aspects != resolved.dirty_aspects()
            || cached_scoped_aspects.as_slice() != resolved.dirty_scoped_aspects()
        {
            return Err(SignalError::invalid_input(
                "dirty mask or scope cache drifted from canonical dependency causes",
            ));
        }
        Ok(resolved)
    }

    pub(crate) fn pending_dependency_revalidation(
        &self,
        node: NodeId,
    ) -> Result<
        Option<crate::data::proof::invalidation::binding::PendingDependencyRevalidation>,
        SignalError,
    > {
        Ok(self.node_pending_revalidation(node)?.cloned())
    }

    pub(crate) fn ensure_cause_readmission_complete(&self) -> Result<(), SignalError> {
        if self.cause_readmission_required || self.cause_sets.requires_readmission() {
            return Err(SignalError::invalid_input(
                "checkpoint dependency authority is quarantined until restore readmission",
            ));
        }
        Ok(())
    }

    pub(crate) fn readmit_checkpoint_causes(&mut self) -> Result<(), SignalError> {
        for node in self.live_node_ids() {
            self.validate_direct_invalidation_storage(node)?;
        }
        if !self.cause_readmission_required && !self.cause_sets.requires_readmission() {
            return Ok(());
        }
        self.cause_sets
            .readmit_graph_instance(self.runtime_instance_id());
        let nodes = self.live_node_ids();
        for &node in &nodes {
            let id = self.get_entry(node)?.pending_cause_set_id();
            let causes = self.cause_sets.get(id)?.to_vec();
            self.validate_pending_causes(node, &causes)
                .map_err(|error| {
                    SignalError::incompatible_snapshot(format!(
                        "checkpoint cause readmission failed for {node}: {error}"
                    ))
                })?;
        }
        self.cause_readmission_required = false;
        self.cause_sets.complete_readmission();
        for node in nodes {
            self.rebuild_dirty_caches_from_pending_causes(node)?;
        }
        Ok(())
    }

    fn validate_direct_invalidation_storage(&self, node: NodeId) -> Result<(), SignalError> {
        let causes = self.cause_sets.get(self.node_pending_cause_set_id(node)?)?;
        let direct = self.node_direct_invalidation_basis(node)?;
        let state = self.get_state(node)?;
        if direct.is_some() && !causes.is_empty() {
            return Err(SignalError::invalid_input(
                "direct invalidation basis cannot coexist with dependency causes",
            ));
        }
        if !causes.is_empty() && matches!(state, crate::data::node::NodeState::Clean) {
            return Err(SignalError::invalid_input(
                "dependency causes require an unsettled consumer",
            ));
        }
        let invalidation_storage = self.node_invalidation_consistency_view(node)?;
        let dirty_aspects = invalidation_storage.dirty_aspects();
        let dirty_scoped_aspects = invalidation_storage.dirty_partition_scopes();
        match direct {
            Some(basis) => {
                if matches!(state, crate::data::node::NodeState::Clean) {
                    return Err(SignalError::invalid_input(
                        "direct invalidation basis requires an unsettled node",
                    ));
                }
                if let crate::data::proof::invalidation::source_seed::DirectInvalidationBasis::SourceRecompute {
                    dirty_aspects: basis_aspects,
                    scoped_aspects,
                    ..
                } = basis
                {
                    if basis_aspects.is_empty()
                        || scoped_aspects.windows(2).any(|pair| pair[0] >= pair[1])
                        || scoped_aspects.iter().any(|(aspect, _)| {
                            !basis_aspects.contains(
                                crate::data::aspect::AspectMask::from_aspect(*aspect),
                            )
                        })
                    {
                        return Err(SignalError::invalid_input(
                            "source recompute basis is empty or non-canonical",
                        ));
                    }
                }
                if basis.generation() == 0
                    || basis.generation() != self.node_direct_invalidation_generation(node)?
                {
                    return Err(SignalError::invalid_input(
                        "direct invalidation generation drifted from node authority",
                    ));
                }
                if dirty_aspects != basis.dirty_aspects()
                    || dirty_scoped_aspects != basis.scoped_aspects()
                {
                    return Err(SignalError::invalid_input(
                        "direct invalidation cache drifted from its persisted basis",
                    ));
                }
            }
            None if causes.is_empty()
                && (!dirty_aspects.is_empty() || !dirty_scoped_aspects.is_empty()) =>
            {
                return Err(SignalError::invalid_input(
                    "dirty cache has no persisted direct or dependency-cause basis",
                ));
            }
            None => {}
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn inject_pending_causes_unchecked_for_test(
        &mut self,
        node: NodeId,
        causes: impl IntoIterator<Item = ResolvedDependencyCause>,
    ) -> Result<PendingCauseSetId, SignalError> {
        let current = self.get_entry(node)?.pending_cause_set_id();
        let id = self.cause_sets.replace_set(current, causes)?;
        self.get_entry_mut(node)?.set_pending_cause_set_id(id);
        self.rebuild_dirty_caches_from_pending_causes(node)?;
        Ok(id)
    }
}
