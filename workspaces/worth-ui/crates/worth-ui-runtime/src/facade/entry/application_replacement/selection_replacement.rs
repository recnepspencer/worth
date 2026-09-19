use super::WorthUiActiveApplicationSession;

pub(super) struct UiPreparedSelectionReplacement(
    Option<crate::runtime::selection::UiSelectionRuntimeState>,
);

impl UiPreparedSelectionReplacement {
    pub(super) fn appearance_snapshot(
        &self,
    ) -> Option<crate::runtime::selection::UiSelectionAppearanceOwnerSnapshot> {
        self.0
            .as_ref()
            .map(|owner| owner.appearance_owner_snapshot())
    }

    pub(super) fn into_state(
        self,
    ) -> crate::runtime::UiRuntimeServiceInstallation<
        crate::runtime::selection::UiSelectionRuntimeState,
    > {
        crate::runtime::UiRuntimeServiceInstallation::from_optional(self.0)
    }
}

impl WorthUiActiveApplicationSession {
    pub(super) fn prepare_selection_replacement(
        &self,
        application: &super::WorthUiPreparedApplicationActivation,
        successor: &crate::mounting::UiMountedGraphReplacementSuccessor,
        frame: Option<&crate::mounting::UiAssembledMountedFrame>,
    ) -> UiPreparedSelectionReplacement {
        if application
            .candidate_service_policy_plan()
            .selection()
            .is_none()
        {
            return UiPreparedSelectionReplacement(None);
        }
        let policy = application
            .candidate_service_policy_plan()
            .selection()
            .expect("installed Selection carries normalized policy");
        let mut selection = self.selection.as_ref().cloned().unwrap_or_else(|| {
            crate::runtime::selection::UiSelectionRuntimeState::new_session_restore_candidate_with_policy(
                policy,
            )
        });
        selection.apply_policy(policy);
        let predecessor = self.mounted.view();
        let successor_view = successor.identity_view();
        for prior in predecessor.mounted_instances() {
            let retained_exactly = successor_view
                .mounted_instances()
                .iter()
                .any(|next| next.identity() == prior.identity() && next.basis() == prior.basis());
            if retained_exactly {
                continue;
            }
            selection.retire_mounted_owner(
                prior.basis().semantic_surface_identity(),
                prior.graph_node_identity(),
                crate::runtime::selection::UiSelectionOwnerIncarnation::from_mount_incarnation(
                    prior.mount_incarnation(),
                ),
            );
        }
        if let Some(frame) = frame {
            reconcile_successor_catalogs(&mut selection, frame);
        } else {
            selection.suspend_projection_catalogs();
        }
        UiPreparedSelectionReplacement(Some(selection))
    }
}

fn reconcile_successor_catalogs(
    selection: &mut crate::runtime::selection::UiSelectionRuntimeState,
    frame: &crate::mounting::UiAssembledMountedFrame,
) {
    for family in selection.projection_families().iter().copied() {
        let Some(slot) = family.projection_input_slot() else {
            continue;
        };
        let Some(worth_ui_query_binding::UiProjectionInputFactReference::Collection(collection)) =
            frame.projection_input(slot)
        else {
            selection.retire_family(family);
            continue;
        };
        if collection.posture() != worth_ui_query_binding::UiProjectionInputPosture::Current {
            selection.retire_family(family);
            continue;
        }
        let revision = collection.revision().observation_order();
        if !selection.family_requires_catalog_reconciliation(family, revision) {
            continue;
        }
        let (Some(keys), Some(completeness)) = (
            collection.current_application_item_keys(),
            collection.completeness(),
        ) else {
            selection.retire_family(family);
            continue;
        };
        let posture = match completeness {
            worth_ui_query_binding::UiCollectionCompleteness::Complete => {
                crate::runtime::selection::UiSelectionCatalogPosture::Complete
            }
            worth_ui_query_binding::UiCollectionCompleteness::Partial => {
                crate::runtime::selection::UiSelectionCatalogPosture::Partial
            }
        };
        selection
            .reconcile_projection_catalog(family, revision, &keys, posture)
            .expect("prepared replacement retains a valid Selection catalog");
    }
}
