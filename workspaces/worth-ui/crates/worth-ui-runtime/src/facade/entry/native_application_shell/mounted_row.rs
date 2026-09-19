pub(super) struct NativeMountedRow {
    pub(super) authored_semantic_identity: Box<str>,
    pub(super) latest_mounted: worth_ui_host_contract::UiMountedInstanceIdentity,
}

impl super::WorthUiNativeApplicationShell {
    pub(super) fn reconcile_native_mounted_rows(
        &mut self,
        receipt: &crate::runtime::rebind::UiRebindReceipt,
    ) {
        let decisions = receipt
            .plan()
            .identity_decisions()
            .iter()
            .filter(|entry| {
                entry.key().kind() == crate::graph::UiGraphFactConsumerKind::GraphNode
                    && entry.key().authored_identity().starts_with("component:")
            })
            .map(|entry| {
                let candidate = match entry.candidate() {
                    Some(crate::graph::UiGraphFactConsumerIdentity::GraphNode(node)) => Some(node),
                    _ => None,
                };
                (Box::<str>::from(entry.key().authored_identity()), candidate)
            })
            .collect::<Vec<_>>();
        for (authored, candidate) in decisions {
            match candidate {
                Some(graph_node) => {
                    let mounted = self
                        .session
                        .mounted_graph_node(graph_node)
                        .ok()
                        .and_then(|handle| self.session.mounted_instances_for(handle).ok())
                        .and_then(|instances| {
                            instances.iter().copied().find(|instance| {
                                self.session
                                    .mounted
                                    .current_mounted_identity_basis(*instance)
                                    .is_some_and(|basis| {
                                        basis.semantic_surface_identity() == self.surface
                                    })
                            })
                        })
                        .expect("accepted native component candidate is mounted");
                    if let Some(index) = self.mounted_row_indices.get(&authored).copied() {
                        self.mounted_rows[index].latest_mounted = mounted;
                    } else {
                        let index = self.mounted_rows.len();
                        self.mounted_row_indices.insert(authored.clone(), index);
                        self.mounted_rows.push(NativeMountedRow {
                            authored_semantic_identity: authored,
                            latest_mounted: mounted,
                        });
                    }
                }
                None => self.remove_native_mounted_row(&authored),
            }
        }
    }

    fn remove_native_mounted_row(&mut self, authored: &str) {
        let Some(index) = self.mounted_row_indices.remove(authored) else {
            return;
        };
        self.mounted_rows.swap_remove(index);
        if let Some(swapped) = self.mounted_rows.get(index) {
            self.mounted_row_indices
                .insert(swapped.authored_semantic_identity.clone(), index);
        }
    }

    pub(super) fn current_native_graph_node(
        &self,
        row: &NativeMountedRow,
    ) -> Option<crate::graph::UiGraphNodeIdentity> {
        self.session
            .application
            .prepared_authority()
            .consumed_fact_index()
            .unique_authored_graph_node(&row.authored_semantic_identity)
    }

    pub(super) fn current_native_mounted_instance(
        &self,
        row: &NativeMountedRow,
    ) -> Option<worth_ui_host_contract::UiMountedInstanceIdentity> {
        let graph_node = self.current_native_graph_node(row)?;
        self.session
            .mounted
            .current_mounted_identity_basis(row.latest_mounted)
            .filter(|basis| {
                basis.graph_node_identity() == graph_node
                    && basis.semantic_surface_identity() == self.surface
            })
            .map(|_| row.latest_mounted)
    }
}
