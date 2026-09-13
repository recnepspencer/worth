use std::collections::BTreeSet;

use crate::evidence::UiAllocationNeighborhoodScope;
use crate::graph::UiGraphNodeIdentity;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiAllocationCatalogDeltaCounters {
    changed_graph_node_lookups: usize,
    active_scope_lookups: usize,
    submitted_row_visits: usize,
    submitted_member_visits: usize,
    carried_row_visits: usize,
    derived_index_row_visits: usize,
    derived_index_membership_mutations: usize,
    derived_index_owner_mutations: usize,
    derived_index_key_probes: usize,
    derived_index_node_copies: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationCatalogRowDisposition {
    Replanned,
    Inserted,
    Removed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAllocationCatalogRowTransition {
    root: UiGraphNodeIdentity,
    disposition: UiAllocationCatalogRowDisposition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAllocationCatalogSuccessorReceipt {
    predecessor_rows: usize,
    successor_rows: usize,
    carried_rows: usize,
    transitions: Box<[UiAllocationCatalogRowTransition]>,
    counters: UiAllocationCatalogDeltaCounters,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiAllocationCatalogDeltaClosureDenial {
    CandidateGraphAuthorityMismatch,
    UnknownRemovedRoot(UiGraphNodeIdentity),
    UnjustifiedRemoval(UiGraphNodeIdentity),
    MissingAffectedRow(UiGraphNodeIdentity),
    MissingCandidateCoverage(UiGraphNodeIdentity),
    ChangedRowOverlapsCarriedTruth(UiGraphNodeIdentity),
    CounterExhausted,
    Neighborhood(crate::graph::UiAllocationNeighborhoodDenial),
}

pub(crate) struct UiAllocationCatalogDeltaClosure {
    pub(crate) delta: crate::graph::UiAdmittedAllocationCatalogDelta,
    pub(crate) affected_predecessor_scopes: Box<[UiAllocationNeighborhoodScope]>,
    pub(crate) counters: UiAllocationCatalogDeltaCounters,
    pub(crate) receipt: UiAllocationCatalogSuccessorReceipt,
}

impl UiAllocationCatalogDeltaCounters {
    pub fn changed_graph_node_lookups(self) -> usize {
        self.changed_graph_node_lookups
    }

    pub fn active_scope_lookups(self) -> usize {
        self.active_scope_lookups
    }

    pub fn submitted_row_visits(self) -> usize {
        self.submitted_row_visits
    }

    pub fn submitted_member_visits(self) -> usize {
        self.submitted_member_visits
    }

    pub fn carried_row_visits(self) -> usize {
        self.carried_row_visits
    }

    pub fn derived_index_row_visits(self) -> usize {
        self.derived_index_row_visits
    }

    pub fn derived_index_membership_mutations(self) -> usize {
        self.derived_index_membership_mutations
    }

    pub fn derived_index_owner_mutations(self) -> usize {
        self.derived_index_owner_mutations
    }

    pub fn derived_index_key_probes(self) -> usize {
        self.derived_index_key_probes
    }

    pub fn derived_index_node_copies(self) -> usize {
        self.derived_index_node_copies
    }

    pub(crate) fn bind_derived_index_work(
        &mut self,
        work: crate::runtime::invalidation_narrowing::UiDerivedIndexDeltaCounters,
    ) {
        self.derived_index_row_visits = work.row_visits();
        self.derived_index_membership_mutations = work.membership_mutations();
        self.derived_index_owner_mutations = work.owner_mutations();
        self.derived_index_key_probes = work.persistent_key_probes();
        self.derived_index_node_copies = work.persistent_node_copies();
    }
}

impl UiAllocationCatalogRowTransition {
    pub fn root(&self) -> UiGraphNodeIdentity {
        self.root
    }

    pub fn disposition(&self) -> UiAllocationCatalogRowDisposition {
        self.disposition
    }
}

impl UiAllocationCatalogSuccessorReceipt {
    pub fn predecessor_rows(&self) -> usize {
        self.predecessor_rows
    }

    pub fn successor_rows(&self) -> usize {
        self.successor_rows
    }

    pub fn carried_rows(&self) -> usize {
        self.carried_rows
    }

    pub fn transitions(&self) -> &[UiAllocationCatalogRowTransition] {
        &self.transitions
    }

    pub fn counters(&self) -> UiAllocationCatalogDeltaCounters {
        self.counters
    }

    pub(crate) fn bind_derived_index_work(
        &mut self,
        work: crate::runtime::invalidation_narrowing::UiDerivedIndexDeltaCounters,
    ) {
        self.counters.bind_derived_index_work(work);
    }
}

impl crate::runtime::WorthUiRuntime {
    pub(crate) fn admit_allocation_catalog_delta_closure(
        &self,
        pending: &crate::runtime::WorthUiPendingActivation,
        active_snapshot: &crate::graph::UiGraphSnapshot,
        delta: crate::graph::UiAdmittedAllocationCatalogDelta,
        graph_changed_nodes: &BTreeSet<UiGraphNodeIdentity>,
    ) -> Result<UiAllocationCatalogDeltaClosure, UiAllocationCatalogDeltaClosureDenial> {
        if pending
            .candidate_application_authority()
            .graph_authority_identity()
            != delta.graph_authority_identity()
        {
            return Err(UiAllocationCatalogDeltaClosureDenial::CandidateGraphAuthorityMismatch);
        }
        let mut counters = UiAllocationCatalogDeltaCounters::default();
        let (active_changed, candidate_changed) = affected_graph_nodes_for(
            pending,
            active_snapshot,
            &delta.snapshot,
            graph_changed_nodes,
            &mut counters,
        )?;
        let authority = self.allocation_invalidation_index.borrow();
        let catalog = &authority.catalog;
        let mut affected_scopes = BTreeSet::new();
        for node in &active_changed {
            bump(&mut counters.active_scope_lookups)?;
            affected_scopes.extend(catalog.scopes_for_node(*node).iter().cloned());
        }
        let mut removed = delta.removed_roots.iter().copied().collect::<BTreeSet<_>>();
        for root in &removed {
            let row = catalog.row_for_root(*root).ok_or(
                UiAllocationCatalogDeltaClosureDenial::UnknownRemovedRoot(*root),
            )?;
            let scope = row.scope();
            let graph_removes_planning = !delta.snapshot.participates_in_allocation_planning(*root);
            if !affected_scopes.contains(&scope) && !graph_removes_planning {
                return Err(UiAllocationCatalogDeltaClosureDenial::UnjustifiedRemoval(
                    *root,
                ));
            }
            affected_scopes.insert(scope);
        }
        let changed_roots = delta
            .changed
            .iter()
            .map(|(basis, _)| basis.graph_node_identity())
            .collect::<BTreeSet<_>>();
        let mut candidate_coverage = BTreeSet::new();
        for (basis, selected) in &delta.changed {
            bump(&mut counters.submitted_row_visits)?;
            if let Some(active) = catalog.row_for_root(basis.graph_node_identity()) {
                affected_scopes.insert(active.scope());
            }
            let neighborhood = basis
                .admit_allocation_neighborhood(&delta.snapshot, selected)
                .map_err(UiAllocationCatalogDeltaClosureDenial::Neighborhood)?;
            for member in neighborhood.members() {
                bump(&mut counters.submitted_member_visits)?;
                let node = member.graph_node_identity();
                candidate_coverage.insert(node);
                for active_scope in catalog.scopes_for_node(node) {
                    bump(&mut counters.active_scope_lookups)?;
                    if !affected_scopes.contains(active_scope)
                        && !changed_roots.contains(&active_scope.root_graph_node_identity())
                    {
                        return Err(
                            UiAllocationCatalogDeltaClosureDenial::ChangedRowOverlapsCarriedTruth(
                                node,
                            ),
                        );
                    }
                }
            }
        }
        for scope in &affected_scopes {
            let root = scope.root_graph_node_identity();
            if !removed.contains(&root)
                && !changed_roots.contains(&root)
                && !delta.snapshot.participates_in_allocation_planning(root)
            {
                removed.insert(root);
            }
            if !removed.contains(&root) && !changed_roots.contains(&root) {
                return Err(UiAllocationCatalogDeltaClosureDenial::MissingAffectedRow(
                    root,
                ));
            }
        }
        for node in candidate_changed
            .into_iter()
            .filter(|node| delta.snapshot.participates_in_allocation_planning(*node))
        {
            if !candidate_coverage.contains(&node) {
                return Err(UiAllocationCatalogDeltaClosureDenial::MissingCandidateCoverage(node));
            }
        }
        let predecessor_rows = catalog.len();
        let carried_rows = predecessor_rows
            .checked_sub(affected_scopes.len())
            .ok_or(UiAllocationCatalogDeltaClosureDenial::CounterExhausted)?;
        let successor_rows = carried_rows
            .checked_add(delta.changed.len())
            .ok_or(UiAllocationCatalogDeltaClosureDenial::CounterExhausted)?;
        let mut transitions = delta
            .changed
            .iter()
            .map(|(basis, _)| {
                let root = basis.graph_node_identity();
                UiAllocationCatalogRowTransition {
                    root,
                    disposition: if catalog.row_for_root(root).is_some() {
                        UiAllocationCatalogRowDisposition::Replanned
                    } else {
                        UiAllocationCatalogRowDisposition::Inserted
                    },
                }
            })
            .collect::<Vec<_>>();
        transitions.extend(removed.iter().map(|root| UiAllocationCatalogRowTransition {
            root: *root,
            disposition: UiAllocationCatalogRowDisposition::Removed,
        }));
        transitions.sort_by_key(|row| row.root);
        let receipt = UiAllocationCatalogSuccessorReceipt {
            predecessor_rows,
            successor_rows,
            carried_rows,
            transitions: transitions.into_boxed_slice(),
            counters,
        };
        drop(authority);
        Ok(UiAllocationCatalogDeltaClosure {
            delta,
            affected_predecessor_scopes: affected_scopes.into_iter().collect(),
            counters,
            receipt,
        })
    }
}

fn affected_graph_nodes_for(
    pending: &crate::runtime::WorthUiPendingActivation,
    active: &crate::graph::UiGraphSnapshot,
    candidate: &crate::graph::UiGraphSnapshot,
    mounted_changes: &BTreeSet<UiGraphNodeIdentity>,
    counters: &mut UiAllocationCatalogDeltaCounters,
) -> Result<
    (BTreeSet<UiGraphNodeIdentity>, BTreeSet<UiGraphNodeIdentity>),
    UiAllocationCatalogDeltaClosureDenial,
> {
    let actual_mounted_changes = mounted_changes.iter().copied().filter(|node| {
        active.participates_in_allocation_planning(*node)
            != candidate.participates_in_allocation_planning(*node)
    });
    let mut active_nodes = BTreeSet::new();
    let mut candidate_nodes = BTreeSet::new();
    for node in actual_mounted_changes {
        active_nodes.insert(node);
        candidate_nodes.insert(node);
    }
    for classification in pending
        .staged_replacement()
        .node_plan()
        .changed_classifications()
    {
        let Some(provenance) = classification.authored_provenance_digest() else {
            continue;
        };
        bump(&mut counters.changed_graph_node_lookups)?;
        active_nodes.extend(
            active
                .graph_node_ids_for_authored_provenance(provenance)
                .iter()
                .copied(),
        );
        bump(&mut counters.changed_graph_node_lookups)?;
        candidate_nodes.extend(
            candidate
                .graph_node_ids_for_authored_provenance(provenance)
                .iter()
                .copied(),
        );
    }
    Ok((active_nodes, candidate_nodes))
}

fn bump(counter: &mut usize) -> Result<(), UiAllocationCatalogDeltaClosureDenial> {
    *counter = counter
        .checked_add(1)
        .ok_or(UiAllocationCatalogDeltaClosureDenial::CounterExhausted)?;
    Ok(())
}
