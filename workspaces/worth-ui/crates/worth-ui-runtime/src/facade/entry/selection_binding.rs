use super::WorthUiActiveApplicationSession;
pub use crate::mounting::UiMountedSelectionBindingDenial;

impl WorthUiActiveApplicationSession {
    /// Binds an independently mounted item to a declared collection owner.
    /// Both receipts and the option must be current. Rebinding an item to a
    /// different key requires retiring its previous mounted identity.
    pub fn bind_selection_item(
        &mut self,
        owner: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
        item: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
        option: worth_ui_query_binding::UiProjectionOptionReference,
    ) -> Result<(), UiMountedSelectionBindingDenial> {
        let mapping = self
            .mounted
            .admit_selection_item_binding(owner, item, &option)?;
        let prepared = self.application.prepared_authority();
        let projection = option.owner_revision().projection_identity();
        if prepared
            .query_binding_plan()
            .projection_input_slot(projection)
            != Some(option.owner_revision().slot())
        {
            return Err(UiMountedSelectionBindingDenial::OwnerNotDeclared);
        }
        let content_owner = prepared
            .consumed_fact_index()
            .consumes_projection(projection, mapping.owner.graph_node());
        let intent_owner = prepared.intent_catalog().lookup(mapping.owner.graph_node(), crate::capability::UiSemanticInteractionFamily::SelectionCommit)
            .is_some_and(|(route, _)| match route {
                crate::declaration::UiIntentCatalogResolvedRoute::Product { declaration, .. } => declaration.payload().iter().any(|binding| {
                    matches!(binding.source(), crate::declaration::UiResolvedIntentPayloadSource::ProjectionSelection(source)
                        if source.slot() == option.owner_revision().slot() && source.identity() == projection)
                }),
                _ => false,
            });
        if !content_owner && !intent_owner {
            return Err(UiMountedSelectionBindingDenial::OwnerNotDeclared);
        }
        let selection = self
            .selection
            .as_mut()
            .ok_or(UiMountedSelectionBindingDenial::SelectionUnavailable)?;
        let Some(worth_ui_query_binding::UiProjectionInputFactReference::Collection(collection)) =
            self.mounted
                .current_projection_input(option.owner_revision().slot())
        else {
            return Err(UiMountedSelectionBindingDenial::CollectionUnavailable);
        };
        if let Some(registration) = mapping
            .registration(&collection, selection)
            .map_err(|_| UiMountedSelectionBindingDenial::SelectionRejected)?
        {
            selection
                .synchronize(registration)
                .map_err(|_| UiMountedSelectionBindingDenial::SelectionRejected)?;
        }
        self.mounted
            .install_selection_item_binding(owner, item, option, mapping);
        Ok(())
    }
}
