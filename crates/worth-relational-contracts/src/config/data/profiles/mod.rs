mod presets;

use crate::config::data::*;

impl RelationalRuntimeConfig {
    pub fn for_profile(profile: RelationalRuntimeProfile) -> Self {
        presets::default_profile_config(profile)
    }
}

impl Default for RelationalRuntimeConfig {
    fn default() -> Self {
        Self::for_profile(RelationalRuntimeProfile::CertificationCore)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::data::RelationalDiagnosticsProfile;

    #[test]
    fn runtime_profiles_expose_distinct_boundary_policies() {
        let geometry = RelationalRuntimeProfile::GeometryKernel.boundary_policy();
        let chip = RelationalRuntimeProfile::ChipSimulation.boundary_policy();
        let ai = RelationalRuntimeProfile::AiWorkflow.boundary_policy();

        assert_eq!(
            geometry.execution_lane,
            RuntimeExecutionLane::RichInteractive
        );
        assert_eq!(
            geometry.diagnostics_boundary,
            DiagnosticsBoundary::RichCertification
        );
        assert!(!geometry.allows_compiled_lane);

        assert_eq!(chip.execution_lane, RuntimeExecutionLane::OperationalThin);
        assert_eq!(
            chip.diagnostics_boundary,
            DiagnosticsBoundary::MinimalHotTruth
        );
        assert!(chip.allows_compiled_lane);

        assert_eq!(ai.execution_lane, RuntimeExecutionLane::AuditReplayHeavy);
        assert_eq!(
            ai.diagnostics_boundary,
            DiagnosticsBoundary::DurableWorkflow
        );
        assert!(!ai.keeps_replay_hot_path_thin);
    }

    #[test]
    fn resolved_profile_configs_match_boundary_defaults() {
        for profile in [
            RelationalRuntimeProfile::CertificationCore,
            RelationalRuntimeProfile::GeometryKernel,
            RelationalRuntimeProfile::ChipSimulation,
            RelationalRuntimeProfile::AiWorkflow,
        ] {
            let config = RelationalRuntimeConfig::for_profile(profile);
            assert!(
                config.profile_boundary_matches_defaults(),
                "profile {:?} drifted from its boundary defaults",
                profile
            );
        }
    }

    #[test]
    fn overriding_diagnostics_profile_breaks_boundary_default_match() {
        let mut config =
            RelationalRuntimeConfig::for_profile(RelationalRuntimeProfile::ChipSimulation);
        config.diagnostics.profile = RelationalDiagnosticsProfile::chip_rich_certification();

        assert!(!config.profile_boundary_matches_defaults());
    }
}
