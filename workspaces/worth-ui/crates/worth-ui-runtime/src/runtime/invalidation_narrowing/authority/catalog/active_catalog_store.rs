use crate::evidence::UiAllocationNeighborhoodScope;
use crate::graph::UiGraphNodeIdentity;
use crate::runtime::persistent_index::UiPersistentOrdMap;

type ActivationRow = crate::runtime::allocation_receipt::UiCommittedAllocationCatalogActivationRow;

/// Complete active allocation truth. The ordered roots and node reverse index
/// are persistent, so a candidate successor shares every untouched branch.
#[derive(Clone, Debug, Default)]
pub(crate) struct UiActiveAllocationCatalog {
    rows: UiPersistentOrdMap<UiAllocationNeighborhoodScope, ActivationRow>,
    scope_by_root: UiPersistentOrdMap<UiGraphNodeIdentity, UiAllocationNeighborhoodScope>,
    scopes_by_node: UiPersistentOrdMap<UiGraphNodeIdentity, Box<[UiAllocationNeighborhoodScope]>>,
}

impl UiActiveAllocationCatalog {
    pub(crate) fn len(&self) -> usize {
        self.rows.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub(crate) fn row(&self, scope: &UiAllocationNeighborhoodScope) -> Option<&ActivationRow> {
        self.rows.get(scope)
    }

    pub(crate) fn row_for_root(&self, root: UiGraphNodeIdentity) -> Option<&ActivationRow> {
        self.scope_by_root
            .get(&root)
            .and_then(|scope| self.rows.get(scope))
    }

    pub(crate) fn scopes_for_node(
        &self,
        node: UiGraphNodeIdentity,
    ) -> &[UiAllocationNeighborhoodScope] {
        self.scopes_by_node.get(&node).map_or(&[], Box::as_ref)
    }

    pub(crate) fn iter(
        &self,
    ) -> impl Iterator<Item = (&UiAllocationNeighborhoodScope, &ActivationRow)> {
        self.rows.iter()
    }

    #[cfg(test)]
    pub(crate) fn shared_row_nodes_with(&self, predecessor: &Self) -> usize {
        self.rows.shared_node_count_with(&predecessor.rows)
    }

    pub(crate) fn insert(&mut self, row: ActivationRow) {
        let scope = row.scope();
        if let Some(prior) = self.rows.get_shared(&scope) {
            self.remove_indexes(&scope, &prior);
        }
        self.insert_indexes(&scope, &row);
        self.rows.insert(scope, row);
    }

    pub(crate) fn remove_root(
        &mut self,
        root: UiGraphNodeIdentity,
    ) -> Option<UiAllocationNeighborhoodScope> {
        let scope = self.scope_by_root.get(&root)?.clone();
        let row = self.rows.get_shared(&scope)?;
        self.remove_indexes(&scope, &row);
        self.rows.remove(&scope);
        Some(scope)
    }

    fn insert_indexes(&mut self, scope: &UiAllocationNeighborhoodScope, row: &ActivationRow) {
        self.scope_by_root
            .insert(scope.root_graph_node_identity(), scope.clone());
        for member in row.neighborhood().members() {
            let node = member.graph_node_identity();
            let mut scopes = self
                .scopes_by_node
                .get(&node)
                .map_or_else(Vec::new, |rows| rows.to_vec());
            match scopes.binary_search(scope) {
                Ok(_) => {}
                Err(ordinal) => scopes.insert(ordinal, scope.clone()),
            }
            self.scopes_by_node.insert(node, scopes.into_boxed_slice());
        }
    }

    fn remove_indexes(&mut self, scope: &UiAllocationNeighborhoodScope, row: &ActivationRow) {
        self.scope_by_root.remove(&scope.root_graph_node_identity());
        for member in row.neighborhood().members() {
            let node = member.graph_node_identity();
            let Some(existing) = self.scopes_by_node.get(&node) else {
                continue;
            };
            let mut scopes = existing.to_vec();
            if let Ok(ordinal) = scopes.binary_search(scope) {
                scopes.remove(ordinal);
            }
            if scopes.is_empty() {
                self.scopes_by_node.remove(&node);
            } else {
                self.scopes_by_node.insert(node, scopes.into_boxed_slice());
            }
        }
    }
}
