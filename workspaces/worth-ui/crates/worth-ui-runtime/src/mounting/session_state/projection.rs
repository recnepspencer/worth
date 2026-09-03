use super::WorthUiMountedSessionState;

pub(crate) struct UiMountedPaintAttribution {
    pub(crate) surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    pub(crate) mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    pub(crate) node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    pub(crate) authored_provenance_digest: u64,
    pub(crate) authored_semantic_identity_digest: u64,
}

impl WorthUiMountedSessionState {
    pub(crate) fn seal_appearance_receipt_basis(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        incarnation: worth_ui_host_contract::UiMountIncarnation,
        successor: &crate::mounting::UiMountedNodeReceiptBasis,
    ) -> Result<
        crate::mounting::UiMountedAppearanceReceiptBasis,
        crate::mounting::UiMountedAppearanceReceiptBasisDenial,
    > {
        self.identity
            .seal_appearance_receipt_basis(instance, incarnation, successor)
    }

    pub(crate) fn current_mounted_identity_basis(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<crate::mounting::UiMountedIdentityBasis> {
        self.identity
            .projection_instance(instance)
            .map(|view| view.basis().clone())
    }

    pub(crate) fn current_surface_for_binding(
        &self,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> Option<worth_ui_host_contract::UiSemanticSurfaceIdentity> {
        self.identity
            .current_projection()?
            .view_for(binding)
            .ok()
            .map(|view| view.surface())
    }

    pub(crate) fn current_surfaces(
        &self,
    ) -> impl Iterator<Item = worth_ui_host_contract::UiSemanticSurfaceIdentity> + '_ {
        self.identity
            .view()
            .surface_bindings()
            .iter()
            .map(|binding| binding.semantic_surface_identity())
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub(crate) fn current_portal_owner_for_child(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<(
        crate::graph::UiGraphNodeIdentity,
        worth_ui_host_contract::UiMountedInstanceIdentity,
    )> {
        self.identity
            .current_projection()?
            .portal_owner_for_child(instance)
    }

    pub(crate) fn native_paint_attribution(
        &self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> Option<UiMountedPaintAttribution> {
        let view = self.identity.current_projection()?.view_for(binding).ok()?;
        if view.frame() != frame {
            return None;
        }
        for order in view.authored_paint_order().iter().rev() {
            let command = order.command();
            let identity = view
                .filled_rects()
                .rows()
                .iter()
                .find(|mechanic| {
                    worth_ui_host_contract::UiMountedPaintCommandIdentity::filled_rect(mechanic)
                        == command
                })
                .map(|mechanic| (mechanic.mounted_instance(), mechanic.node_receipt()))
                .or_else(|| {
                    view.semantic_text()
                        .rows()
                        .iter()
                        .find(|mechanic| {
                            worth_ui_host_contract::UiMountedPaintCommandIdentity::semantic_text(
                                mechanic,
                            ) == command
                        })
                        .map(|mechanic| (mechanic.mounted_instance(), mechanic.node_receipt()))
                });
            let Some((mounted_instance, node_receipt)) = identity else {
                continue;
            };
            let Some(authored) = self.identity.current_authored_attribution(mounted_instance)
            else {
                continue;
            };
            return Some(UiMountedPaintAttribution {
                surface: view.surface(),
                mounted_instance,
                node_receipt,
                authored_provenance_digest: authored.source_provenance_digest,
                authored_semantic_identity_digest: authored.semantic_identity_digest,
            });
        }
        None
    }

    pub(crate) fn classify_frame_reuse(
        &self,
        contract: crate::mounting::UiMountedFrameReuseContract,
    ) -> crate::mounting::UiMountedFrameReuse {
        self.identity.classify_reuse(contract)
    }

    pub(crate) fn current_allocation_truth_revision(&self) -> Option<u64> {
        self.identity.current_allocation_truth_revision()
    }

    pub(crate) fn seal_frame_reuse_contract(
        &self,
        basis: crate::mounting::UiMountedFrameReuseExternalBasis,
    ) -> crate::mounting::UiMountedFrameReuseContract {
        self.identity.seal_reuse_contract(basis)
    }

    pub(crate) fn begin_frame_assembly(
        &self,
        input: crate::mounting::UiMountedFrameAssemblyInput<'_, '_>,
    ) -> Result<
        crate::mounting::UiMountedFrameAssembler<'_>,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        crate::mounting::UiMountedFrameAssembler::begin(&self.identity, input)
    }

    pub(crate) fn begin_superseding_frame_assembly<'state>(
        &'state self,
        predecessor: &'state crate::mounting::UiPreparedMountedFrame,
        input: crate::mounting::UiMountedFrameAssemblyInput<'_, '_>,
    ) -> Result<
        crate::mounting::UiMountedFrameAssembler<'state>,
        crate::mounting::UiMountedFramePreparationDenial,
    > {
        crate::mounting::UiMountedFrameAssembler::begin_graph_replacement(
            &self.identity,
            Some(predecessor.semantic_projection()),
            Some(predecessor.canonical_core().frame()),
            input,
        )
    }

    pub(crate) fn current_projection_input(
        &self,
        slot: worth_ui_query_binding::UiProjectionInputSlot,
    ) -> Option<worth_ui_query_binding::UiProjectionInputFactReference> {
        self.retention.current_projection_input(slot)
    }
}
