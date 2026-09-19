use std::sync::Arc;

#[must_use]
pub struct UiPreparedIntentPayload {
    graph_node: crate::graph::UiGraphNodeIdentity,
    definition_id: crate::capability::UiIntentId,
    declaration: Arc<crate::declaration::UiCanonicalIntentDeclaration>,
    basis: super::UiIntentInputBasis,
    execution: crate::runtime::intent_execution::UiPreparedIntentExecution,
}

impl UiPreparedIntentPayload {
    pub(crate) const fn new(
        graph_node: crate::graph::UiGraphNodeIdentity,
        definition_id: crate::capability::UiIntentId,
        declaration: Arc<crate::declaration::UiCanonicalIntentDeclaration>,
        basis: super::UiIntentInputBasis,
        execution: crate::runtime::intent_execution::UiPreparedIntentExecution,
    ) -> Self {
        Self {
            graph_node,
            definition_id,
            declaration,
            basis,
            execution,
        }
    }

    pub const fn definition_id(&self) -> crate::capability::UiIntentId {
        self.definition_id
    }

    pub fn declaration_identity(&self) -> &str {
        self.declaration.identity().as_str()
    }

    pub const fn input_basis(&self) -> &super::UiIntentInputBasisReceipt {
        self.basis.receipt()
    }

    pub(crate) const fn portal_declaration(&self) -> Option<worth_ui_dsl::UiPortalDeclarationId> {
        self.input_basis().portal_declaration()
    }

    pub fn retained_owner_reference_count(&self) -> usize {
        self.basis.retained_owner_reference_count()
    }

    pub fn retained_payload_count(&self) -> usize {
        self.execution.retained_payload_count()
    }

    pub const fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.graph_node
    }

    pub fn retained_operability_dependency_reference_count(&self) -> usize {
        self.basis
            .operability()
            .retained_dependency_reference_count()
    }

    pub(crate) const fn operability_basis(
        &self,
    ) -> &super::super::operability::UiIntentOperabilityBasis {
        self.basis.operability()
    }

    pub(crate) fn interaction_family(&self) -> crate::capability::UiSemanticInteractionFamily {
        self.declaration.interaction()
    }

    pub(crate) fn selection_option(
        &self,
    ) -> Option<&worth_ui_query_binding::UiProjectionOptionReference> {
        self.basis.selection_option()
    }

    pub(crate) const fn command_route_receipt(
        &self,
    ) -> Option<&crate::runtime::UiCommandRouteReceipt> {
        self.basis.command_route_receipt()
    }

    pub(crate) fn interaction_time_basis(
        &self,
    ) -> worth_ui_host_contract::UiHostObservationTimeBasis {
        self.basis.interaction_time_basis()
    }

    pub(crate) fn payload_inputs_are_current(
        &self,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        application_facts: &super::UiIntentApplicationFactState,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> bool {
        self.basis
            .payload_inputs_are_current(mounted, application_facts, generation)
    }

    pub(crate) fn operability_dependencies_are_current(
        &self,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        application_facts: &super::UiIntentApplicationFactState,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Result<(), super::super::operability::UiIntentOperabilityDependencyDrift> {
        self.basis
            .operability()
            .currentness(mounted, application_facts, generation)
    }

    pub(crate) fn declaration_reference(
        &self,
    ) -> &Arc<crate::declaration::UiCanonicalIntentDeclaration> {
        &self.declaration
    }

    pub(crate) fn execution_reservation_basis(
        &self,
    ) -> crate::runtime::intent_execution::UiIntentExecutionReservationBasis {
        self.execution.reservation_basis(
            self.definition_id,
            self.basis.receipt().cost().admitted_utf8_bytes(),
        )
    }

    pub(crate) fn into_execution(
        self,
    ) -> crate::runtime::intent_execution::UiPreparedIntentExecution {
        self.execution
    }
}
