use std::collections::BTreeMap;

mod accepted_settlement;
mod anchor_access;
mod direct_succession;
pub(crate) use direct_succession::UiPreparedScrollDirectSuccession;
#[cfg(any(test, feature = "certification-support"))]
mod certification;
mod layout_succession;
mod ownership_catalog;
mod reconciliation;
mod route_candidate;
mod transition_binding;

use reconciliation::reconcile_owner_record;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct UiScrollOwnerRecord {
    incarnation: super::UiScrollOwnerIncarnation,
    axes: super::UiScrollAxes,
    bounds: super::UiScrollBounds,
    offset: super::UiScrollOffset,
    anchor: Option<super::UiScrollAnchor>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UiScrollOwnershipCatalogRecord {
    incarnation: super::UiScrollOwnerIncarnation,
    resolution:
        Result<super::UiResolvedScrollOwnershipChain, super::UiScrollOwnershipResolutionDenial>,
}

/// Sole owner of semantic scroll offsets for one active application session.
/// Query- or host-provided extents establish bounds; only this state changes offsets.
#[derive(Clone)]
pub(crate) struct UiScrollRuntimeState {
    #[cfg(test)]
    clone_observation: route_candidate::UiScrollStateCloneObservation,
    policy: crate::declaration::UiScrollPolicy,
    owners: BTreeMap<super::UiScrollOwnerIdentity, UiScrollOwnerRecord>,
    ownership_catalog:
        BTreeMap<worth_ui_host_contract::UiMountedInstanceIdentity, UiScrollOwnershipCatalogRecord>,
    ownership_references: BTreeMap<super::UiScrollOwnerIdentity, u64>,
    transition_targets: super::transition::UiScrollTransitionSuccession,
    counters: super::UiScrollCounters,
    ownership_resolutions: u64,
    ownership_graph_nodes_visited: u64,
    ownership_plan_nodes_visited: u64,
    revision: u64,
    last_owner: Option<super::UiScrollOwnerInspectionRecord>,
    pending_layouts: BTreeMap<
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        layout_succession::UiScrollLayoutSuccessor,
    >,
    pending_direct: BTreeMap<super::UiScrollOwnerIdentity, UiPreparedScrollDirectSuccession>,
}

impl UiScrollRuntimeState {
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) const fn new_session_restore_candidate() -> Self {
        Self::new_session_restore_candidate_with_policy(
            crate::declaration::UiScrollPolicy::nested_region(),
        )
    }

    pub(crate) const fn new_session_restore_candidate_with_policy(
        policy: crate::declaration::UiScrollPolicy,
    ) -> Self {
        Self {
            #[cfg(test)]
            clone_observation: route_candidate::UiScrollStateCloneObservation,
            policy,
            owners: BTreeMap::new(),
            ownership_catalog: BTreeMap::new(),
            ownership_references: BTreeMap::new(),
            transition_targets: super::transition::UiScrollTransitionSuccession::new(),
            counters: super::UiScrollCounters::new(),
            ownership_resolutions: 0,
            ownership_graph_nodes_visited: 0,
            ownership_plan_nodes_visited: 0,
            revision: 0,
            last_owner: None,
            pending_layouts: BTreeMap::new(),
            pending_direct: BTreeMap::new(),
        }
    }

    pub(crate) fn apply_policy(&mut self, policy: crate::declaration::UiScrollPolicy) {
        self.policy = policy;
    }

    pub(in crate::runtime) const fn reveal_alignment(
        &self,
    ) -> crate::declaration::UiScrollRevealAlignment {
        self.policy.reveal_alignment()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn register(
        &mut self,
        registration: super::UiScrollOwnerRegistration,
    ) -> Result<(), super::UiScrollRouteDenial> {
        if !registration
            .bounds()
            .contains(registration.initial_offset())
        {
            return Err(super::UiScrollRouteDenial::InitialOffsetOutOfBounds);
        }
        self.owners.insert(
            registration.identity(),
            UiScrollOwnerRecord {
                incarnation: registration.incarnation(),
                axes: registration.axes(),
                bounds: registration.bounds(),
                offset: registration.initial_offset(),
                anchor: None,
            },
        );
        Ok(())
    }

    pub(crate) fn reconcile_rebind(
        &mut self,
        request: super::UiScrollRebindRequest,
    ) -> Result<super::UiScrollAnchorReconciliationReceipt, super::UiScrollRouteDenial> {
        let request = if matches!(
            self.policy.anchor_behavior(),
            crate::declaration::UiScrollAnchorBehavior::ClampOffset
        ) {
            super::UiScrollRebindRequest::new(
                request.registration(),
                request.successor_anchor(),
                super::UiScrollAnchorPolicy::Clamp,
            )
        } else {
            request
        };
        let registration = request.registration();
        if !registration
            .bounds()
            .contains(registration.initial_offset())
        {
            return Err(super::UiScrollRouteDenial::InitialOffsetOutOfBounds);
        }
        let previous = self.owners.get(&registration.identity()).copied();
        let successor_anchor = request.successor_anchor();
        let policy = match (request.policy(), previous, successor_anchor) {
            (super::UiScrollAnchorPolicy::Rebase, Some(previous), Some(successor))
                if previous.incarnation == registration.incarnation()
                    && previous
                        .anchor
                        .is_some_and(|anchor| anchor.exact_basis(successor)) =>
            {
                super::UiScrollAnchorPolicy::Preserve
            }
            (policy, _, _) => policy,
        };
        let (outcome, offset, anchor) =
            reconcile_owner_record(previous, registration, successor_anchor, policy);
        self.owners.insert(
            registration.identity(),
            UiScrollOwnerRecord {
                incarnation: registration.incarnation(),
                axes: registration.axes(),
                bounds: registration.bounds(),
                offset,
                anchor,
            },
        );
        self.reconcile_transition_bounds(
            registration.identity(),
            registration.incarnation(),
            registration.bounds(),
        );
        Ok(super::UiScrollAnchorReconciliationReceipt::new(
            outcome, offset,
        ))
    }

    pub(crate) fn offset(
        &self,
        owner: super::UiScrollOwnerIdentity,
        incarnation: super::UiScrollOwnerIncarnation,
    ) -> Result<super::UiScrollOffset, super::UiScrollRouteDenial> {
        Ok(self.exact_owner(owner, incarnation)?.offset)
    }

    pub(super) fn owner_geometry(
        &self,
        owner: super::UiScrollOwnerIdentity,
        incarnation: super::UiScrollOwnerIncarnation,
    ) -> Result<
        (
            super::UiScrollOffset,
            super::UiScrollBounds,
            super::UiScrollAxes,
        ),
        super::UiScrollRouteDenial,
    > {
        let record = self.exact_owner(owner, incarnation)?;
        Ok((record.offset, record.bounds, record.axes))
    }

    pub(crate) fn route(
        &mut self,
        request: super::UiScrollDeltaRequest,
    ) -> Result<super::UiScrollRouteReceipt, super::UiScrollRouteDenial> {
        let result = self.route_prevalidated(request, None);
        if result.is_err() {
            self.counters.reject();
        }
        result
    }

    /// Applies allocation-derived bounds and the matching host delta as one
    /// owner transition. Bounds are ordered by the already-resolved chain, so
    /// no partially reconciled owner can escape a denied route.
    pub(crate) fn route_with_reconciled_bounds(
        &mut self,
        request: super::UiScrollDeltaRequest,
        bounds: &[super::UiScrollBounds],
    ) -> Result<super::UiScrollRouteReceipt, super::UiScrollRouteDenial> {
        assert_eq!(
            request.chain().len(),
            bounds.len(),
            "current Scroll bounds must cover the exact resolved owner chain"
        );
        let result = self.route_prevalidated(request, Some(bounds));
        if result.is_err() {
            self.counters.reject();
        }
        result
    }

    fn route_prevalidated(
        &mut self,
        request: super::UiScrollDeltaRequest,
        reconciled_bounds: Option<&[super::UiScrollBounds]>,
    ) -> Result<super::UiScrollRouteReceipt, super::UiScrollRouteDenial> {
        let surface = request.chain()[0].owner().semantic_surface();
        for entry in request.chain() {
            if entry.owner().semantic_surface() != surface {
                return Err(super::UiScrollRouteDenial::CrossSurfaceChain);
            }
            self.exact_owner(entry.owner(), entry.incarnation())?;
        }
        let next_revision = self
            .revision
            .checked_add(1)
            .ok_or(super::UiScrollRouteDenial::RevisionExhausted)?;
        let mut remainder = request.delta();
        let mut transitions = Vec::with_capacity(request.chain().len());
        for (index, entry) in request.chain().iter().enumerate() {
            let record = self.exact_owner(entry.owner(), entry.incarnation())?;
            let previous = record.offset;
            let bounds = reconciled_bounds.map_or(record.bounds, |bounds| bounds[index]);
            let reconciled = bounds.clamp(previous);
            let (current, consumed) =
                super::routing::consume_delta(reconciled, bounds, record.axes, remainder);
            remainder = remainder.subtract(consumed);
            transitions.push(super::UiScrollChainTransition::new(
                entry.owner(),
                previous,
                current,
                consumed,
            ));
            if remainder.is_zero() || !self.policy.bubbles_remainder() {
                break;
            }
        }
        let owners_visited = u16::try_from(transitions.len())
            .expect("scroll chain depth limit is representable by u16");
        let changed = transitions
            .iter()
            .filter(|transition| transition.previous() != transition.current())
            .count();
        let next_counters = self.counters.after_admission(owners_visited, changed)?;
        // Reconciled bounds are live geometry for every owner the chain names,
        // so they land on all of them. Only the owners the route visited have
        // a new offset to take: a route stops where the travel runs out, and
        // an owner it never reached is exactly where it was. Writing bounds
        // only that far would leave an ancestor measuring itself against
        // geometry from whenever a delta last had something left for it --
        // which, for the zero delta a smooth wheel routes, is never.
        if let Some(bounds) = reconciled_bounds {
            for (index, entry) in request.chain().iter().enumerate() {
                self.exact_owner_mut(entry.owner(), entry.incarnation())?
                    .bounds = bounds[index];
            }
        }
        for (entry, transition) in request.chain().iter().zip(&transitions) {
            self.exact_owner_mut(entry.owner(), entry.incarnation())?
                .offset = transition.current();
        }
        if let Some(bounds) = reconciled_bounds {
            for (index, entry) in request.chain().iter().enumerate() {
                self.reconcile_transition_bounds(entry.owner(), entry.incarnation(), bounds[index]);
            }
        }
        self.counters = next_counters;
        self.revision = next_revision;
        let receipt = super::UiScrollRouteReceipt::new(
            request.cause(),
            transitions,
            remainder,
            owners_visited,
            self.revision,
        );
        self.last_owner = super::UiScrollOwnerInspectionRecord::from_receipt(&receipt);
        Ok(receipt)
    }

    #[cfg(test)]
    pub(in crate::runtime) const fn counters(&self) -> super::UiScrollCounters {
        self.counters
    }

    pub(crate) fn shutdown(&mut self) -> usize {
        let released = self.owners.len();
        self.owners.clear();
        self.ownership_catalog.clear();
        self.ownership_references.clear();
        self.release_transitions();
        self.last_owner = None;
        self.pending_layouts.clear();
        self.pending_direct.clear();
        released
    }

    pub(crate) const fn last_owner(&self) -> Option<super::UiScrollOwnerInspectionRecord> {
        self.last_owner
    }

    pub(crate) fn owner_count(&self) -> usize {
        self.owners.len()
    }

    fn exact_owner(
        &self,
        owner: super::UiScrollOwnerIdentity,
        incarnation: super::UiScrollOwnerIncarnation,
    ) -> Result<&UiScrollOwnerRecord, super::UiScrollRouteDenial> {
        let record = self
            .owners
            .get(&owner)
            .ok_or(super::UiScrollRouteDenial::UnknownOwner)?;
        if record.incarnation != incarnation {
            return Err(super::UiScrollRouteDenial::StaleOwnerIncarnation);
        }
        Ok(record)
    }

    fn exact_owner_mut(
        &mut self,
        owner: super::UiScrollOwnerIdentity,
        incarnation: super::UiScrollOwnerIncarnation,
    ) -> Result<&mut UiScrollOwnerRecord, super::UiScrollRouteDenial> {
        let record = self
            .owners
            .get_mut(&owner)
            .ok_or(super::UiScrollRouteDenial::UnknownOwner)?;
        if record.incarnation != incarnation {
            return Err(super::UiScrollRouteDenial::StaleOwnerIncarnation);
        }
        Ok(record)
    }
}
