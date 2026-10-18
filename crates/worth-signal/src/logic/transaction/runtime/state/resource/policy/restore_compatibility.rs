use super::super::ResourceRuntimeState;
use super::registry::resource_policy_resolution_signal_error;
use crate::data::resource::*;
use crate::data::telemetry::ResourceTelemetry;

impl ResourceRuntimeState {
    pub fn classify_policy_compatibility_optional(
        &self,
        declaration: &ResourceNodeDeclaration,
        telemetry: Option<&mut ResourceTelemetry>,
    ) -> Result<ResourcePolicyCompatibilityReport, crate::data::error::SignalError> {
        let Some(historical_descriptor) = self.descriptor_for_node(declaration.node()) else {
            return Err(crate::data::error::SignalError::invalid_input(format!(
                "cannot classify resource policy compatibility for undeclared resource node {}",
                declaration.node().node()
            )));
        };
        let validated = self.validated_policy_declaration(declaration)?;
        let report = ResourcePolicyCompatibilityReport::classify_against_validated_declaration(
            historical_descriptor.descriptor_id(),
            historical_descriptor.node(),
            historical_descriptor.lowered_policy_bundle(),
            &validated,
            &self.policy_registry,
        )
        .map_err(resource_policy_resolution_signal_error)?;
        if let Some(telemetry) = telemetry {
            telemetry.resource_policy_compatibility_count += 1;
            telemetry.resource_policy_descriptor_comparison_count = telemetry
                .resource_policy_descriptor_comparison_count
                .saturating_add(report.compared_width() as u64);
            telemetry.resource_policy_descriptor_incompatibility_count = telemetry
                .resource_policy_descriptor_incompatibility_count
                .saturating_add(report.incompatible_width() as u64);
            telemetry.record_boundary_performance_envelope(report.performance());
        }
        Ok(report)
    }

    pub fn admit_policy_restore_compatibility_optional(
        &self,
        declaration: &ResourceNodeDeclaration,
        mut telemetry: Option<&mut ResourceTelemetry>,
    ) -> Result<
        Result<ResourcePolicyRestoreCompatibilityProof, DeniedResourcePolicyRestoreCompatibility>,
        crate::data::error::SignalError,
    > {
        let validated = self.validated_policy_declaration(declaration)?;
        let replay_decision_plan = self.replay_decision_plan_from_validated(&validated)?;
        let Some(historical_descriptor) = self.descriptor_for_node(declaration.node()) else {
            return Err(crate::data::error::SignalError::invalid_input(format!(
                "cannot classify resource policy compatibility for undeclared resource node {}",
                declaration.node().node()
            )));
        };
        let compatibility =
            ResourcePolicyCompatibilityReport::classify_against_validated_declaration(
                historical_descriptor.descriptor_id(),
                historical_descriptor.node(),
                historical_descriptor.lowered_policy_bundle(),
                &validated,
                &self.policy_registry,
            )
            .map_err(resource_policy_resolution_signal_error)?;
        if let Some(telemetry) = telemetry.as_deref_mut() {
            telemetry.resource_policy_compatibility_count += 1;
            telemetry.resource_policy_descriptor_comparison_count = telemetry
                .resource_policy_descriptor_comparison_count
                .saturating_add(compatibility.compared_width() as u64);
            telemetry.resource_policy_descriptor_incompatibility_count = telemetry
                .resource_policy_descriptor_incompatibility_count
                .saturating_add(compatibility.incompatible_width() as u64);
            telemetry.record_boundary_performance_envelope(compatibility.performance());
            telemetry.resource_replay_compatibility_decision_count += 1;
        }
        if compatibility.is_compatible() {
            if compatibility
                .families()
                .iter()
                .all(|family| replay_decision_plan.admits_compatible_class(family.class()))
            {
                if let Some(telemetry) = telemetry.as_deref_mut() {
                    telemetry.resource_replay_compatible_count += 1;
                }
                Ok(Ok(
                    ResourcePolicyRestoreCompatibilityProof::from_compatibility(
                        compatibility,
                        &replay_decision_plan,
                    )
                    .expect("compatible report must admit restore compatibility proof"),
                ))
            } else {
                if let Some(telemetry) = telemetry.as_deref_mut() {
                    telemetry.resource_replay_incompatible_count += 1;
                }
                let primary_incompatible_kind = compatibility
                    .families()
                    .iter()
                    .find(|family| !replay_decision_plan.admits_compatible_class(family.class()))
                    .map(|family| family.kind())
                    .expect("compatible report with replay gate denial must have a gated family");
                Ok(Err(
                    DeniedResourcePolicyRestoreCompatibility::from_replay_policy_gate(
                        compatibility,
                        &replay_decision_plan,
                        primary_incompatible_kind,
                    ),
                ))
            }
        } else {
            if let Some(telemetry) = telemetry.as_deref_mut() {
                telemetry.resource_replay_incompatible_count += 1;
            }
            if compatibility
                .families()
                .iter()
                .any(|family| family.class() == ResourcePolicyCompatibilityClass::MissingDescriptor)
                && let Some(telemetry) = telemetry
            {
                telemetry.resource_replay_missing_policy_count += 1;
            }
            Ok(Err(
                DeniedResourcePolicyRestoreCompatibility::from_compatibility(
                    compatibility,
                    &replay_decision_plan,
                ),
            ))
        }
    }
}
