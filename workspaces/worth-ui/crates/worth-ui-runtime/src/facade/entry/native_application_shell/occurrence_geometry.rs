use super::WorthUiNativeApplicationShell;

#[derive(Clone, Debug)]
pub struct UiNativeMountedComponentLayoutInput {
    authored_semantic_identity: Box<str>,
    instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    allocation: Option<crate::capability::ComponentAllocationMeasurementContract>,
    portal_parent: Option<worth_ui_host_contract::UiMountedInstanceIdentity>,
}

#[derive(Clone, Debug)]
pub struct UiNativeMountedRegionLayoutInput {
    owner: worth_ui_host_contract::UiMountedInstanceIdentity,
    region_kind: Box<str>,
    declaration: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
    executed_region: Box<str>,
}

impl UiNativeMountedComponentLayoutInput {
    pub fn authored_semantic_identity(&self) -> &str {
        &self.authored_semantic_identity
    }

    pub fn instance(&self) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.instance
    }

    pub fn allocation(&self) -> Option<crate::capability::ComponentAllocationMeasurementContract> {
        self.allocation
    }

    pub fn portal_parent(&self) -> Option<worth_ui_host_contract::UiMountedInstanceIdentity> {
        self.portal_parent
    }
}

impl UiNativeMountedRegionLayoutInput {
    pub fn owner(&self) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.owner
    }

    pub fn region_kind(&self) -> &str {
        &self.region_kind
    }

    pub fn geometry(
        &self,
        bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    ) -> crate::mounting::UiMountedMosaicRegionGeometry {
        crate::mounting::UiMountedMosaicRegionGeometry::new(
            self.owner,
            self.declaration,
            self.executed_region.clone(),
            bounds,
        )
    }
}

impl WorthUiNativeApplicationShell {
    pub(crate) fn native_program_layout_required(&self) -> bool {
        let viewport_changed = self.native_layout_viewport().is_some_and(|viewport| {
            self.session
                .mounted
                .current_surface_viewport(self.surface)
                .is_none_or(|(_, current)| current != viewport)
        });
        viewport_changed
            || self
                .mounted_rows
                .iter()
                .filter_map(|row| row.mounted)
                .any(|instance| {
                    !self
                        .session
                        .mounted
                        .has_current_occurrence_geometry(instance)
                })
    }

    /// Return the admitted component contracts paired with their exact mounted
    /// occurrences. These are immutable inputs; the application remains the
    /// owner of the resulting layout.
    pub fn native_component_layout_inputs(&self) -> Box<[UiNativeMountedComponentLayoutInput]> {
        let capabilities = self.session.application.capabilities().components();
        self.mounted_rows
            .iter()
            .filter_map(|row| {
                let instance = row.mounted?;
                let component_id = row
                    .authored_semantic_identity
                    .strip_prefix("component:")
                    .and_then(|identity| crate::capability::ComponentId::new(identity).ok());
                let descriptor = component_id
                    .as_ref()
                    .and_then(|identity| capabilities.get(identity));
                let portal_parent = descriptor
                    .and_then(|descriptor| descriptor.portal_child_contract())
                    .and_then(|contract| {
                        self.mounted_component_instance(&format!(
                            "component:{}",
                            contract.owner().as_str()
                        ))
                    });
                Some(UiNativeMountedComponentLayoutInput {
                    authored_semantic_identity: row.authored_semantic_identity.clone(),
                    instance,
                    allocation: descriptor
                        .and_then(|descriptor| descriptor.allocation_measurement_contract()),
                    portal_parent,
                })
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    /// Return the exact executed Mosaic regions whose rectangles must be
    /// supplied with the native layout batch.
    pub fn native_region_layout_inputs(&self) -> Box<[UiNativeMountedRegionLayoutInput]> {
        let Some(surface_declaration) = self
            .session
            .authored_overlay_bindings
            .bound_owners()
            .find_map(|(declaration, runtime, _)| (runtime == self.surface).then_some(declaration))
        else {
            return Box::default();
        };
        self.mounted_rows
            .iter()
            .filter_map(|row| {
                let owner = row.mounted?;
                let graph_node = self
                    .session
                    .mounted
                    .current_mounted_identity_basis(owner)?
                    .graph_node_identity();
                Some((owner, graph_node))
            })
            .flat_map(|(owner, graph_node)| {
                self.session
                    .application
                    .mounted_region_declarations(surface_declaration, graph_node)
                    .0
                    .into_iter()
                    .map(move |binding| UiNativeMountedRegionLayoutInput {
                        owner,
                        region_kind: binding.region_kind().into(),
                        declaration: binding.declaration(),
                        executed_region: binding.executed_region().into(),
                    })
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    /// Return the exact mounted instance currently owned by one authored
    /// component. The application layout owner uses this identity when it
    /// publishes native occurrence geometry.
    pub fn mounted_component_instance(
        &self,
        authored_semantic_identity: &str,
    ) -> Option<worth_ui_host_contract::UiMountedInstanceIdentity> {
        let index = *self.mounted_row_indices.get(authored_semantic_identity)?;
        self.mounted_rows.get(index)?.mounted
    }

    /// Mint the current basis for one exact native layout publication.
    pub fn native_layout_basis(
        &mut self,
    ) -> Result<
        crate::mounting::UiMountedLayoutBasis,
        crate::mounting::UiMountedOccurrenceGeometryDenial,
    > {
        self.session.begin_mounted_layout().basis(self.surface)
    }

    /// Mint the next monotonic revision for this native surface.
    pub fn next_native_layout_revision(
        &self,
    ) -> Result<
        crate::mounting::UiMountedLayoutRevision,
        crate::mounting::UiMountedOccurrenceGeometryDenial,
    > {
        self.session
            .mounted
            .next_occurrence_geometry_revision(self.surface)
    }

    /// Commit layout-owner supplied, exact instance-keyed native geometry.
    pub fn complete_native_layout(
        &mut self,
        batch: crate::mounting::UiMountedSurfaceGeometryBatch,
    ) -> Result<
        crate::mounting::UiMountedLayoutCompletionReceipt,
        crate::mounting::UiMountedOccurrenceGeometryDenial,
    > {
        if batch.surface() != self.surface {
            return Err(crate::mounting::UiMountedOccurrenceGeometryDenial::ForeignSurface);
        }
        self.session
            .begin_mounted_layout()
            .complete_surface_geometry(batch)
    }
}
