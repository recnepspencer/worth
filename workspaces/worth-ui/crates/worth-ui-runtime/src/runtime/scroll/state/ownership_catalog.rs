impl super::UiScrollRuntimeState {
    pub(crate) fn has_mounted_ownership(&self) -> bool {
        !self.ownership_references.is_empty()
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn resolve_and_install_ownership(
        &mut self,
        mounted: worth_ui_host_contract::UiMountedInstanceIdentity,
        incarnation: crate::runtime::scroll::UiScrollOwnerIncarnation,
        graph: crate::graph::UiGraphAuthority<'_>,
        plan: crate::mounting::UiMountedPlanProjectionSource<'_>,
        graph_node: crate::graph::UiGraphNodeIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        repeated_instance_digest: u64,
    ) {
        self.ownership_resolutions = self
            .ownership_resolutions
            .checked_add(1)
            .expect("mounted identity capacity bounds Scroll ownership resolutions");
        let resolution = crate::runtime::scroll::ownership_chain::resolve(
            graph,
            plan,
            graph_node,
            surface,
            repeated_instance_digest,
        );
        if let Ok(chain) = resolution.as_ref() {
            self.ownership_graph_nodes_visited = self
                .ownership_graph_nodes_visited
                .checked_add(u64::from(chain.graph_nodes_visited()))
                .expect("mounted identity capacity bounds Scroll ownership discovery work");
            self.ownership_plan_nodes_visited = self
                .ownership_plan_nodes_visited
                .checked_add(u64::from(chain.plan_nodes_visited()))
                .expect("mounted identity capacity bounds Scroll plan discovery work");
        }
        self.install_ownership_resolution(mounted, incarnation, resolution);
    }

    fn install_ownership_resolution(
        &mut self,
        mounted: worth_ui_host_contract::UiMountedInstanceIdentity,
        incarnation: crate::runtime::scroll::UiScrollOwnerIncarnation,
        resolution: Result<
            crate::runtime::scroll::UiResolvedScrollOwnershipChain,
            crate::runtime::scroll::UiScrollOwnershipResolutionDenial,
        >,
    ) {
        let successor_owners = resolution
            .as_ref()
            .map(|chain| chain.owners())
            .unwrap_or_default();
        if let Some(predecessor) = self.ownership_catalog.remove(&mounted) {
            let predecessor_owners = predecessor
                .resolution
                .as_ref()
                .map(|chain| chain.owners())
                .unwrap_or_default();
            for owner in predecessor_owners
                .iter()
                .copied()
                .filter(|owner| !successor_owners.contains(owner))
            {
                self.release_ownership_reference(owner);
            }
            for owner in successor_owners
                .iter()
                .copied()
                .filter(|owner| !predecessor_owners.contains(owner))
            {
                self.retain_ownership_reference(owner);
            }
        } else {
            for owner in successor_owners.iter().copied() {
                self.retain_ownership_reference(owner);
            }
        }
        self.ownership_catalog.insert(
            mounted,
            super::UiScrollOwnershipCatalogRecord {
                incarnation,
                resolution,
            },
        );
    }

    pub(crate) fn ownership_chain(
        &self,
        mounted: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Result<
        &crate::runtime::scroll::UiResolvedScrollOwnershipChain,
        crate::runtime::scroll::UiScrollOwnershipResolutionDenial,
    > {
        self.ownership_catalog
            .get(&mounted)
            .ok_or(crate::runtime::scroll::UiScrollOwnershipResolutionDenial::OwnershipNotIndexed)?
            .resolution
            .as_ref()
            .map_err(|denial| *denial)
    }

    /// Every mounted instance whose ownership chain this catalog holds.
    ///
    /// The instances are borrowed from the catalog rather than gathered into a
    /// list: a caller that only reads them pays nothing for the walk, and a
    /// caller that has to change the catalog while it walks them gathers at its
    /// own call site, where the reason for the copy is visible.
    pub(crate) fn ownership_instances(
        &self,
    ) -> impl Iterator<Item = worth_ui_host_contract::UiMountedInstanceIdentity> + '_ {
        self.ownership_catalog.keys().copied()
    }

    pub(crate) fn retire_mounted_instance(
        &mut self,
        mounted: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> usize {
        let Some(record) = self.ownership_catalog.remove(&mounted) else {
            return 0;
        };
        let owners = record
            .resolution
            .as_ref()
            .map(|chain| chain.owners().to_vec())
            .unwrap_or_default();
        owners
            .into_iter()
            .map(|owner| usize::from(self.release_ownership_reference(owner)))
            .sum()
    }

    pub(crate) fn suspend_mounted_instance(
        &mut self,
        mounted: worth_ui_host_contract::UiMountedInstanceIdentity,
        incarnation: crate::runtime::scroll::UiScrollOwnerIncarnation,
    ) {
        self.retire_mounted_instance(mounted);
        self.ownership_catalog.insert(
            mounted,
            super::UiScrollOwnershipCatalogRecord {
                incarnation,
                resolution: Err(
                    crate::runtime::scroll::UiScrollOwnershipResolutionDenial::OwnershipNotIndexed,
                ),
            },
        );
    }

    fn retain_ownership_reference(&mut self, owner: crate::runtime::scroll::UiScrollOwnerIdentity) {
        let references = self.ownership_references.entry(owner).or_insert(0);
        *references = references
            .checked_add(1)
            .expect("scroll chain depth and mounted capacity bound owner references");
    }

    fn release_ownership_reference(
        &mut self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
    ) -> bool {
        let Some(references) = self.ownership_references.get_mut(&owner) else {
            return false;
        };
        *references -= 1;
        if *references != 0 {
            return false;
        }
        self.ownership_references.remove(&owner);
        self.owners.remove(&owner).is_some()
    }
}
