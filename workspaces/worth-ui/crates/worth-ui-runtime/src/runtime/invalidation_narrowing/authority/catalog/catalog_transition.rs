#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiAllocationNeighborhoodActivationDenial {
    StalePredecessor,
    DerivedIndexDiverged,
    ScrollBinding(super::UiScrollOwnerCatalogDenialReport),
    PortalBinding(crate::runtime::UiPortalActivationBindingDenial),
}

pub(crate) struct UiPreparedInvalidationCatalogTransition {
    predecessor_identity_digest: u64,
    successor: super::UiAllocationInvalidationAuthority,
    scroll_evidence: crate::runtime::UiScrollCatalogSwapEvidence,
    derived_index_counters: crate::runtime::invalidation_narrowing::UiDerivedIndexDeltaCounters,
}

impl UiPreparedInvalidationCatalogTransition {
    pub(crate) fn predecessor_identity_digest(&self) -> u64 {
        self.predecessor_identity_digest
    }

    pub(crate) fn scroll_catalog_evidence(&self) -> crate::runtime::UiScrollCatalogSwapEvidence {
        self.scroll_evidence.clone()
    }

    pub(crate) fn derived_index_counters(
        &self,
    ) -> crate::runtime::invalidation_narrowing::UiDerivedIndexDeltaCounters {
        self.derived_index_counters
    }
}

impl super::UiAllocationInvalidationAuthority {
    pub(crate) fn prepare_catalog_transition(
        &self,
        delta: &super::UiAllocationNeighborhoodCatalogTransition,
    ) -> Result<UiPreparedInvalidationCatalogTransition, UiAllocationNeighborhoodActivationDenial>
    {
        if !delta.certifies_successor() {
            return Err(UiAllocationNeighborhoodActivationDenial::StalePredecessor);
        }
        if delta.affected_predecessor_scopes().is_some() {
            return self.prepare_delta_catalog_transition(delta);
        }
        let mut successor = self.clone();
        let mut successor_catalog = super::UiActiveAllocationCatalog::default();
        for row in delta.changed_rows() {
            successor_catalog.insert(row.clone());
        }
        let committed_bindings = delta.committed_bindings();
        let graph_transition = successor.graph_replan.seal_activation_transition(
            committed_bindings
                .rows()
                .iter()
                .map(|binding| {
                    (
                        binding.scope(),
                        binding.neighborhood().identity().clone(),
                        binding.neighborhood().graph_snapshot_authority_digest(),
                        binding.planning_identity_digest(),
                        binding.graph_replan_admission(),
                    )
                })
                .collect(),
        );
        if !successor
            .graph_replan
            .apply_activation_transition(&graph_transition)
        {
            return Err(UiAllocationNeighborhoodActivationDenial::StalePredecessor);
        }
        let predecessor_identity = graph_transition.predecessor_identity_digest();
        let successor_identity = graph_transition.successor_identity_digest();
        let scroll_bindings = super::UiScrollInvalidationBindingIndex::seal(
            committed_bindings,
            &successor.graph_replan,
            predecessor_identity,
            successor_identity,
        )
        .map_err(UiAllocationNeighborhoodActivationDenial::ScrollBinding)?;
        let scroll_evidence = crate::runtime::UiScrollCatalogSwapEvidence::Prepared(
            scroll_bindings
                .catalog_receipt
                .clone()
                .expect("sealed scroll bindings carry their receipt"),
        );
        let portal_bindings = super::UiPortalInvalidationBindingIndex::seal(
            committed_bindings,
            &successor.graph_replan,
        )
        .map_err(map_portal_denial)?;
        successor.catalog = successor_catalog;
        successor.scroll_bindings = scroll_bindings;
        successor.portal_bindings = portal_bindings;
        successor.rebuild_indexes();
        Ok(UiPreparedInvalidationCatalogTransition {
            predecessor_identity_digest: self.active_catalog_identity_digest(),
            successor,
            scroll_evidence,
            derived_index_counters: Default::default(),
        })
    }

    fn prepare_delta_catalog_transition(
        &self,
        delta: &super::UiAllocationNeighborhoodCatalogTransition,
    ) -> Result<UiPreparedInvalidationCatalogTransition, UiAllocationNeighborhoodActivationDenial>
    {
        let affected = delta
            .affected_predecessor_scopes()
            .expect("delta posture was proven");
        let mut removed = Vec::with_capacity(affected.len());
        for scope in affected {
            removed.push(
                self.catalog
                    .row(scope)
                    .cloned()
                    .ok_or(UiAllocationNeighborhoodActivationDenial::StalePredecessor)?,
            );
        }
        let mut successor = self.clone();
        for scope in affected {
            if successor
                .catalog
                .remove_root(scope.root_graph_node_identity())
                .as_ref()
                != Some(scope)
            {
                return Err(UiAllocationNeighborhoodActivationDenial::StalePredecessor);
            }
        }
        for row in delta.changed_rows() {
            successor.catalog.insert(row.clone());
        }
        let (predecessor_identity, successor_identity) = successor
            .graph_replan
            .apply_activation_delta(affected, delta.changed_rows())
            .ok_or(UiAllocationNeighborhoodActivationDenial::StalePredecessor)?;
        let derived_index_counters = successor
            .apply_index_delta(&removed, delta.changed_rows())
            .map_err(|()| UiAllocationNeighborhoodActivationDenial::DerivedIndexDiverged)?;
        successor
            .portal_bindings
            .apply_catalog_delta(&removed, delta.changed_rows(), &successor.graph_replan)
            .map_err(map_portal_denial)?;
        successor
            .scroll_bindings
            .apply_catalog_delta(
                &removed,
                delta.changed_rows(),
                delta.committed_bindings().identity_digest(),
                &successor.graph_replan,
                predecessor_identity,
                successor_identity,
            )
            .map_err(UiAllocationNeighborhoodActivationDenial::ScrollBinding)?;
        let scroll_evidence = crate::runtime::UiScrollCatalogSwapEvidence::Prepared(
            successor
                .scroll_bindings
                .catalog_receipt
                .clone()
                .expect("delta scroll bindings carry their receipt"),
        );
        Ok(UiPreparedInvalidationCatalogTransition {
            predecessor_identity_digest: self.active_catalog_identity_digest(),
            successor,
            scroll_evidence,
            derived_index_counters,
        })
    }

    pub(crate) fn commit_catalog_transition(
        &mut self,
        prepared: UiPreparedInvalidationCatalogTransition,
    ) {
        *self = prepared.successor;
    }
}

fn map_portal_denial(
    denial: super::portal_binding_index::UiPortalBindingDenial,
) -> UiAllocationNeighborhoodActivationDenial {
    let denial = match denial {
        super::portal_binding_index::UiPortalBindingDenial::MissingPredecessorBinding => {
            return UiAllocationNeighborhoodActivationDenial::DerivedIndexDiverged;
        }
        super::portal_binding_index::UiPortalBindingDenial::DuplicateRequestIdentity => {
            crate::runtime::UiPortalActivationBindingDenial::CardinalityExceeded
        }
        super::portal_binding_index::UiPortalBindingDenial::MissingGraphTarget => {
            crate::runtime::UiPortalActivationBindingDenial::NeighborhoodMismatch { ordinal: 0 }
        }
        super::portal_binding_index::UiPortalBindingDenial::ReceiptContextMismatch => {
            crate::runtime::UiPortalActivationBindingDenial::AnchorIdentityMismatch { ordinal: 0 }
        }
    };
    UiAllocationNeighborhoodActivationDenial::PortalBinding(denial)
}
