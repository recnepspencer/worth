use super::appearance;
use super::service_installation_reconciliation::{
    reconcile_focus_installation, reconcile_motion_installation, reconcile_portal_installation,
};
use super::*;
use crate::facade::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    pub(in crate::facade::entry::application_replacement) fn commit_application_activation(
        &mut self,
        mut prepared: Box<WorthUiPreparedApplicationActivation>,
        mounted_successor: crate::mounting::UiMountedGraphReplacementSuccessor,
        lifecycle: super::portal_lifecycle::WorthUiPreparedApplicationLifecycle,
        scroll: super::scroll_replacement::UiPreparedScrollReplacement,
        selection: super::selection_replacement::UiPreparedSelectionReplacement,
    ) -> WorthUiApplicationCutoverReceipt {
        let transition = prepared
            .transition
            .take()
            .expect("prepared application transition is present");
        let activation = match transition {
            WorthUiApplicationCutoverTransition::Prepared(activation) => activation,
            WorthUiApplicationCutoverTransition::Committed { .. } => {
                unreachable!("prepared application transition cannot already be committed")
            }
        };
        let publication = self.application.commit_application_activation(activation);
        if let Some(appearance_succession) = prepared.appearance_succession.take() {
            self.commit_appearance_generation_succession(appearance_succession);
        }
        let service_policy_plan = self.application.prepared_authority().service_policy_plan();
        if let Some(command_routing) = self.command_routing.as_mut() {
            command_routing.shutdown();
        }
        self.command_routing = crate::runtime::UiRuntimeServiceInstallation::from_optional(
            self.application
                .prepared_authority()
                .service_policy_plan()
                .command_routing()
                .map(|policy| {
                    crate::runtime::command_routing::UiCommandRoutingRuntimeState::new(
                        crate::runtime::UiServiceStatePersistencePosture::Ephemeral,
                        self.application.capabilities().commands(),
                        policy,
                    )
                }),
        );
        reconcile_focus_installation(
            &mut self.focus,
            service_policy_plan.focus().map(|policy| {
                let restoration = service_policy_plan
                    .portal()
                    .is_none_or(crate::declaration::UiPortalPolicy::restores_focus);
                policy.with_scope_restoration(policy.restores_on_scope_close() && restoration)
            }),
        );
        self.commit_application_lifecycle(lifecycle);
        reconcile_portal_installation(
            &mut self.portal,
            service_policy_plan.portal(),
            &mut self.dormant_portal_stack_ordinal_issuer,
        );
        reconcile_motion_installation(&mut self.motion, service_policy_plan.motion());
        let successor_appearance_demand = self
            .application
            .prepared_authority()
            .consumed_fact_index()
            .appearance_axis_demand();
        self.intent_application_facts =
            crate::runtime::intent::UiIntentApplicationFactState::activate(
                self.application.intent_application_fact_plan(),
                successor_appearance_demand
                    .contains(worth_ui_dsl::UiAppearanceStateAxis::Validation),
            );
        self.intent_confirmation.cancel_all(
            crate::runtime::intent::UiIntentConfirmationCancellationReason::ApplicationRebound,
        );
        self.intent_admission.cancel_all(
            &mut self.intent_execution,
            crate::runtime::intent::UiIntentAdmissionCancellationReason::ApplicationRebound,
        );
        let scroll = scroll.into_state();
        if !scroll.is_installed() {
            let _ = self
                .scroll
                .as_mut()
                .map(crate::runtime::scroll::UiScrollRuntimeState::shutdown);
        }
        self.scroll = scroll;
        let selection = selection.into_state();
        if !selection.is_installed() {
            let _ = self
                .selection
                .as_mut()
                .map(crate::runtime::selection::UiSelectionRuntimeState::shutdown);
        }
        self.selection = selection;
        appearance::reconcile_successor_owners(self);
        self.mounted
            .commit_graph_replacement_successor(mounted_successor);
        self.cancel_all_interactions(
            crate::runtime::interaction::UiInteractionLifecycleStopReason::ApplicationRebound,
        );
        let observation_resources = self.application.retire_observation_resources(
            crate::runtime::observation::UiObservationResourceRetirementCause::
                ApplicationReplacement,
        );
        let intent_evidence = self
            .intent_evidence
            .retire(worth_ui_inspection::UiIntentEvidenceRetirementCause::ApplicationReplacement);
        let (plan_swap, query_retirement, plan_decision, allocation_catalog_successor) =
            publication.into_parts();
        prepared.transition = Some(WorthUiApplicationCutoverTransition::Committed {
            plan_swap,
            plan_decision,
            query_retirement,
            allocation_catalog_successor,
        });
        WorthUiApplicationCutoverReceipt {
            transition: prepared,
            observation_resources,
            intent_evidence,
        }
    }
}
