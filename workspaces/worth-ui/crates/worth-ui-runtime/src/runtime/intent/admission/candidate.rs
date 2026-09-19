pub(crate) struct UiPreparedIntentAdmissionCandidate {
    payload: super::super::payload::UiPreparedIntentPayload,
    decision: super::super::operability::UiIntentOperabilityDecision,
    lineage: Option<super::super::UiIntentAttemptLineage>,
    origin: UiIntentAdmissionCandidateOrigin,
}

pub(crate) struct UiCurrentIntentAdmissionCandidate {
    prepared: UiPreparedIntentAdmissionCandidate,
    currentness_checks: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UiIntentAdmissionCandidateOrigin {
    Direct,
    Confirmed,
}

impl UiPreparedIntentAdmissionCandidate {
    pub(crate) fn direct(proof: super::super::operability::UiIntentOperabilityProof) -> Self {
        let (payload, decision) = proof.into_parts();
        Self {
            payload,
            decision,
            lineage: None,
            origin: UiIntentAdmissionCandidateOrigin::Direct,
        }
    }

    pub(crate) fn confirmed(candidate: super::super::UiConfirmedIntentCandidate) -> Self {
        let (payload, decision, lineage) = candidate.into_parts();
        Self {
            payload,
            decision,
            lineage: Some(lineage),
            origin: UiIntentAdmissionCandidateOrigin::Confirmed,
        }
    }

    pub(crate) const fn payload(&self) -> &super::super::payload::UiPreparedIntentPayload {
        &self.payload
    }

    pub(crate) const fn payload_projection_cost(
        &self,
    ) -> super::super::payload::UiIntentPayloadProjectionCost {
        self.payload.input_basis().cost()
    }

    pub(crate) const fn route_resolution_cost(
        &self,
    ) -> crate::declaration::UiIntentRouteResolutionCost {
        self.payload.input_basis().route_resolution_cost()
    }

    pub(crate) const fn decision(&self) -> &super::super::operability::UiIntentOperabilityDecision {
        &self.decision
    }

    pub(super) const fn origin(&self) -> UiIntentAdmissionCandidateOrigin {
        self.origin
    }

    pub(crate) const fn lineage(&self) -> Option<super::super::UiIntentAttemptLineage> {
        self.lineage
    }

    pub(crate) const fn occupancy_observation(
        &self,
    ) -> &super::super::operability::UiIntentOccupancyObservation {
        self.payload.operability_basis().occupancy()
    }

    pub(crate) fn retained_payload_count(&self) -> usize {
        self.payload.retained_payload_count()
    }

    pub(crate) fn retained_owner_reference_count(&self) -> usize {
        self.payload.retained_owner_reference_count()
            + self
                .payload
                .retained_operability_dependency_reference_count()
    }

    pub(crate) const fn definition_id(&self) -> crate::capability::UiIntentId {
        self.payload.definition_id()
    }

    pub(crate) fn declaration_identity_value(
        &self,
    ) -> &crate::declaration::UiIntentDeclarationIdentity {
        self.payload.declaration_reference().identity()
    }

    pub(crate) const fn target(
        &self,
    ) -> crate::runtime::interaction::UiPresentedInteractionTargetView {
        self.payload.input_basis().target()
    }

    pub(crate) const fn portal_declaration(&self) -> Option<worth_ui_dsl::UiPortalDeclarationId> {
        self.payload.portal_declaration()
    }

    pub(crate) const fn command_route_receipt(
        &self,
    ) -> Option<&crate::runtime::UiCommandRouteReceipt> {
        self.payload.command_route_receipt()
    }

    pub(crate) const fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.payload.graph_node()
    }

    pub(crate) fn generation(&self) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        self.payload.input_basis().generation()
    }

    pub(crate) fn declaration_reference(
        &self,
    ) -> &std::sync::Arc<crate::declaration::UiCanonicalIntentDeclaration> {
        self.payload.declaration_reference()
    }

    pub(super) const fn seal_current(
        self,
        currentness_checks: usize,
    ) -> UiCurrentIntentAdmissionCandidate {
        UiCurrentIntentAdmissionCandidate {
            prepared: self,
            currentness_checks,
        }
    }

    fn into_execution(self) -> crate::runtime::intent_execution::UiPreparedIntentExecution {
        self.payload.into_execution()
    }
}

impl UiCurrentIntentAdmissionCandidate {
    pub(crate) const fn prepared(&self) -> &UiPreparedIntentAdmissionCandidate {
        &self.prepared
    }

    pub(crate) const fn payload_projection_cost(
        &self,
    ) -> super::super::payload::UiIntentPayloadProjectionCost {
        self.prepared.payload_projection_cost()
    }

    pub(crate) const fn route_resolution_cost(
        &self,
    ) -> crate::declaration::UiIntentRouteResolutionCost {
        self.prepared.route_resolution_cost()
    }

    pub(crate) const fn decision(&self) -> &super::super::operability::UiIntentOperabilityDecision {
        self.prepared.decision()
    }

    pub(crate) const fn lineage(&self) -> Option<super::super::UiIntentAttemptLineage> {
        self.prepared.lineage()
    }

    pub(crate) const fn occupancy_observation(
        &self,
    ) -> &super::super::operability::UiIntentOccupancyObservation {
        self.prepared.occupancy_observation()
    }

    pub(crate) const fn currentness_checks(&self) -> usize {
        self.currentness_checks
    }

    pub(crate) fn retained_payload_count(&self) -> usize {
        self.prepared.retained_payload_count()
    }

    pub(crate) fn retained_owner_reference_count(&self) -> usize {
        self.prepared.retained_owner_reference_count()
    }

    pub(crate) const fn definition_id(&self) -> crate::capability::UiIntentId {
        self.prepared.definition_id()
    }

    pub(crate) fn declaration_identity_value(
        &self,
    ) -> &crate::declaration::UiIntentDeclarationIdentity {
        self.prepared.declaration_identity_value()
    }

    pub(crate) const fn target(
        &self,
    ) -> crate::runtime::interaction::UiPresentedInteractionTargetView {
        self.prepared.target()
    }

    pub(crate) const fn portal_declaration(&self) -> Option<worth_ui_dsl::UiPortalDeclarationId> {
        self.prepared.portal_declaration()
    }

    pub(crate) const fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.prepared.graph_node()
    }

    pub(crate) fn generation(&self) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        self.prepared.generation()
    }

    pub(crate) fn declaration_reference(
        &self,
    ) -> &std::sync::Arc<crate::declaration::UiCanonicalIntentDeclaration> {
        self.prepared.declaration_reference()
    }

    pub(crate) fn selection_option(
        &self,
    ) -> Option<&worth_ui_query_binding::UiProjectionOptionReference> {
        self.prepared.payload().selection_option()
    }

    pub(crate) const fn command_route_receipt(
        &self,
    ) -> Option<&crate::runtime::UiCommandRouteReceipt> {
        self.prepared.command_route_receipt()
    }

    pub(crate) fn execution_reservation_basis(
        &self,
    ) -> crate::runtime::intent_execution::UiIntentExecutionReservationBasis {
        self.prepared.payload().execution_reservation_basis()
    }

    pub(crate) fn into_execution(
        self,
    ) -> crate::runtime::intent_execution::UiPreparedIntentExecution {
        self.prepared.into_execution()
    }
}
