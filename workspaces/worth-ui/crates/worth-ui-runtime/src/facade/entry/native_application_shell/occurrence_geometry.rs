use super::WorthUiNativeApplicationShell;

#[derive(Clone, Debug)]
pub struct UiNativeMountedComponentLayoutInput {
    authored_semantic_identity: Box<str>,
    instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    allocation: Option<crate::capability::ComponentAllocationMeasurementContract>,
    portal_parent: Option<worth_ui_host_contract::UiMountedInstanceIdentity>,
    layout: Option<crate::capability::MosaicResponsiveLayout>,
    layout_member: Option<(
        worth_ui_host_contract::UiMountedInstanceIdentity,
        crate::capability::ComponentId,
    )>,
}

#[derive(Clone, Debug)]
pub struct UiNativeMountedRegionLayoutInput {
    owner: worth_ui_host_contract::UiMountedInstanceIdentity,
    region_kind: Box<str>,
    declaration: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
    executed_region: Box<str>,
}

impl UiNativeMountedComponentLayoutInput {
    /// Pairs one mounted occurrence with its admitted descriptor. `mounted`
    /// finds the occurrence of the Portal owner or layout container the
    /// descriptor names.
    pub(crate) fn from_descriptor(
        authored_semantic_identity: impl Into<Box<str>>,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        descriptor: Option<&crate::capability::ComponentDescriptor>,
        components: &crate::capability::FrozenComponentCapabilities,
        mounted: impl Fn(
            &crate::capability::ComponentId,
        ) -> Option<worth_ui_host_contract::UiMountedInstanceIdentity>,
    ) -> Self {
        let allocation =
            descriptor.and_then(|descriptor| descriptor.allocation_measurement_contract());
        let layout_member = descriptor
            .filter(|_| {
                matches!(
                    allocation,
                    Some(crate::capability::ComponentAllocationMeasurementContract::LayoutCell(_))
                )
            })
            .and_then(|descriptor| {
                let container = components.layout_container(descriptor.id())?;
                Some((mounted(container.id())?, descriptor.id().clone()))
            });
        Self {
            authored_semantic_identity: authored_semantic_identity.into(),
            instance,
            allocation,
            portal_parent: descriptor
                .and_then(|descriptor| descriptor.portal_child_contract())
                .and_then(|contract| mounted(contract.owner())),
            layout: descriptor.and_then(|descriptor| descriptor.layout().cloned()),
            layout_member,
        }
    }

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

    /// The tracks this component lays its members out in, when it is a
    /// layout container.
    pub fn layout(&self) -> Option<&crate::capability::MosaicResponsiveLayout> {
        self.layout.as_ref()
    }

    /// The mounted container that places this component in one of its cells.
    pub fn layout_container(&self) -> Option<worth_ui_host_contract::UiMountedInstanceIdentity> {
        self.layout_member.as_ref().map(|(container, _)| *container)
    }

    pub(super) fn layout_node(&self) -> crate::runtime::mosaic::layout::UiMosaicLayoutNode<'_> {
        use crate::runtime::mosaic::layout::UiMosaicLayoutParent;
        crate::runtime::mosaic::layout::UiMosaicLayoutNode {
            instance: self.instance,
            allocation: self.allocation,
            layout: self.layout.as_ref(),
            parent: match (&self.layout_member, self.portal_parent) {
                (Some((container, member)), _) => UiMosaicLayoutParent::Container {
                    container: *container,
                    member,
                },
                (None, Some(owner)) => UiMosaicLayoutParent::Portal(owner),
                (None, None) => UiMosaicLayoutParent::Viewport,
            },
        }
    }
}

impl UiNativeMountedRegionLayoutInput {
    pub(crate) fn from_mounted_region(
        owner: worth_ui_host_contract::UiMountedInstanceIdentity,
        region_kind: &str,
        declaration: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
        executed_region: &str,
    ) -> Self {
        Self {
            owner,
            region_kind: region_kind.into(),
            declaration,
            executed_region: executed_region.into(),
        }
    }

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
                .filter_map(|row| self.current_native_mounted_instance(row))
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
                let instance = self.current_native_mounted_instance(row)?;
                let component_id = row
                    .authored_semantic_identity
                    .strip_prefix("component:")
                    .and_then(|identity| crate::capability::ComponentId::new(identity).ok());
                let descriptor = component_id
                    .as_ref()
                    .and_then(|identity| capabilities.get(identity));
                Some(UiNativeMountedComponentLayoutInput::from_descriptor(
                    row.authored_semantic_identity.clone(),
                    instance,
                    descriptor,
                    capabilities,
                    |component| {
                        self.mounted_component_instance(&format!(
                            "component:{}",
                            component.as_str()
                        ))
                    },
                ))
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
                let owner = self.current_native_mounted_instance(row)?;
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
                    .map(move |binding| {
                        UiNativeMountedRegionLayoutInput::from_mounted_region(
                            owner,
                            binding.region_kind(),
                            binding.declaration(),
                            binding.executed_region(),
                        )
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
        self.current_native_mounted_instance(self.mounted_rows.get(index)?)
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
