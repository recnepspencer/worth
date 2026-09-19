use crate::capability::{
    CapabilityDiagnosticCode, CapabilitySupportKind, ComponentDescriptor, ComponentExecutionLane,
    RegistrationCandidate, RegistrationCandidateDiagnostic, RegistrationDependency,
    COMMAND_FAMILY_NAME, COMPONENT_FAMILY_NAME, THEME_TOKEN_FAMILY_NAME,
};

impl ComponentDescriptor {
    pub(crate) fn registration_candidate(&self) -> RegistrationCandidate {
        let candidate = RegistrationCandidate::new(
            COMPONENT_FAMILY_NAME,
            self.id().as_str(),
            CapabilitySupportKind::Admitted,
        );
        let candidate = add_component_descriptor_diagnostics(candidate, self);
        add_component_dependencies(candidate, self)
    }
}

fn add_component_descriptor_diagnostics(
    mut candidate: RegistrationCandidate,
    descriptor: &ComponentDescriptor,
) -> RegistrationCandidate {
    if descriptor.prop_schema().is_none() {
        candidate = candidate.with_descriptor_diagnostic(RegistrationCandidateDiagnostic::new(
            CapabilityDiagnosticCode::MissingComponentPropSchema,
            "component descriptor requires a typed prop schema",
        ));
    } else if descriptor
        .prop_schema()
        .is_some_and(|prop_schema| !prop_schema.is_typed())
    {
        candidate = candidate.with_descriptor_diagnostic(RegistrationCandidateDiagnostic::new(
            CapabilityDiagnosticCode::MissingComponentPropSchema,
            "component descriptor prop schema must be typed",
        ));
    }

    if descriptor.state_ownership().is_none() {
        candidate = candidate.with_descriptor_diagnostic(RegistrationCandidateDiagnostic::new(
            CapabilityDiagnosticCode::MissingComponentStateOwnership,
            "component descriptor requires state ownership classification",
        ));
    }

    if descriptor.child_policy().is_illegal() {
        candidate = candidate.with_descriptor_diagnostic(RegistrationCandidateDiagnostic::new(
            CapabilityDiagnosticCode::IllegalComponentChildPolicy,
            "component child policy cannot claim shell layout authority",
        ));
    }

    if descriptor.execution_lane() == ComponentExecutionLane::CanvasSpatial
        && descriptor.canvas_spatial_contract().is_none()
    {
        candidate = candidate.with_descriptor_diagnostic(RegistrationCandidateDiagnostic::new(
            CapabilityDiagnosticCode::MissingComponentCanvasSpatialContract,
            "canvas spatial execution requires a bounded canvas contract",
        ));
    }

    if descriptor.execution_lane() == ComponentExecutionLane::RealtimeOverlay
        && descriptor.realtime_overlay_contract().is_none()
    {
        candidate = candidate.with_descriptor_diagnostic(RegistrationCandidateDiagnostic::new(
            CapabilityDiagnosticCode::MissingComponentRealtimeOverlayContract,
            "realtime overlay execution requires an immutable frame policy contract",
        ));
    }

    if descriptor.has_conflicting_allocation_contracts() {
        candidate = candidate.with_descriptor_diagnostic(RegistrationCandidateDiagnostic::new(
            CapabilityDiagnosticCode::ConflictingComponentAllocationContract,
            "component paint, hit-test, and allocation contracts must agree on one allocation",
        ));
    }

    candidate
}

fn add_component_dependencies(
    mut candidate: RegistrationCandidate,
    descriptor: &ComponentDescriptor,
) -> RegistrationCandidate {
    for token_id in descriptor.theme_token_dependencies() {
        candidate = candidate.with_dependency(RegistrationDependency::new(
            THEME_TOKEN_FAMILY_NAME,
            THEME_TOKEN_FAMILY_NAME,
            token_id.as_str(),
        ));
    }

    for command_id in descriptor.command_binding_slots() {
        candidate = candidate.with_dependency(RegistrationDependency::new(
            COMMAND_FAMILY_NAME,
            COMMAND_FAMILY_NAME,
            command_id.as_str(),
        ));
    }

    if let Some(portal_child) = descriptor.portal_child_contract() {
        candidate = candidate.with_dependency(RegistrationDependency::new(
            COMPONENT_FAMILY_NAME,
            COMPONENT_FAMILY_NAME,
            portal_child.owner().as_str(),
        ));
    }
    candidate
}
