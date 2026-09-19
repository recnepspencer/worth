use super::WorthUiApplicationSessionState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiDeclaredSelectionMappingDenial {
    TargetUnavailable,
    GraphNodeChanged,
    SelectionInputUnavailable,
    SelectionInputChanged,
    Selection(crate::runtime::selection::UiSelectionRequestDenial),
}

impl WorthUiApplicationSessionState {
    /// Resolves the same admitted mounted owner/item relationship used by
    /// appearance, while Selection alone decides membership and activation.
    pub(crate) fn declared_selection_for_intent_target(
        &self,
        handoff: &crate::runtime::intent_execution::UiIntentConsequenceHandoff,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        selection: &crate::runtime::selection::UiSelectionRuntimeState,
    ) -> Result<
        Option<crate::runtime::selection::UiDeclaredSelectionBinding>,
        UiDeclaredSelectionMappingDenial,
    > {
        let target = handoff.target();
        let basis = mounted
            .current_mounted_identity_basis(target.mounted_instance())
            .ok_or(UiDeclaredSelectionMappingDenial::TargetUnavailable)?;
        if basis.graph_node_identity() != handoff.graph_node() {
            return Err(UiDeclaredSelectionMappingDenial::GraphNodeChanged);
        }
        if handoff.interaction_family()
            != crate::capability::UiSemanticInteractionFamily::SelectionCommit
        {
            return Ok(None);
        }
        let option = handoff
            .selection_option()
            .ok_or(UiDeclaredSelectionMappingDenial::SelectionInputUnavailable)?;
        let current = mounted
            .current_projection_input(option.owner_revision().slot())
            .ok_or(UiDeclaredSelectionMappingDenial::SelectionInputUnavailable)?;
        let worth_ui_query_binding::UiProjectionInputFactReference::Collection(collection) =
            current
        else {
            return Err(UiDeclaredSelectionMappingDenial::SelectionInputUnavailable);
        };
        if !mounted.selection_item_matches_option(target.mounted_instance(), option) {
            return Err(UiDeclaredSelectionMappingDenial::SelectionInputChanged);
        }
        let mapping = mounted
            .selection_mapping_for_item(target.mounted_instance())
            .map_err(|_| UiDeclaredSelectionMappingDenial::SelectionInputChanged)?;
        let owner = mapping.owner;
        let incarnation = mapping.incarnation;
        let key = mapping.key;
        let registration = mapping
            .registration(&collection, selection)
            .map_err(UiDeclaredSelectionMappingDenial::Selection)?;
        let request = selection
            .request_for_declared_activation(owner, incarnation, key, registration.as_ref())
            .map_err(UiDeclaredSelectionMappingDenial::Selection)?;
        let action = crate::runtime::session::service_proposal::UiDeclaredFocusSelectionAction::new(
            target.mounted_instance(),
            owner,
            incarnation,
            request,
            crate::runtime::session::service_proposal::UiSelectionInvocationCause::Intent,
        );
        Ok(Some(match registration {
            Some(registration) => {
                crate::runtime::selection::UiDeclaredSelectionBinding::new(action, registration)
            }
            None => crate::runtime::selection::UiDeclaredSelectionBinding::current(action),
        }))
    }
}
