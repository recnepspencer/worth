mod override_application;
mod presets;

use std::collections::BTreeMap;

use crate::config::data::*;

use override_application::apply_config_overrides;
use presets::default_profile_config;

impl RelationalRuntimeConfig {
    pub fn resolved(
        profile: RelationalRuntimeProfile,
        overrides: RelationalConfigOverride,
    ) -> Self {
        let mut config = default_profile_config(profile);
        let mut provenance_entries = BTreeMap::new();

        apply_config_overrides(&mut config, &overrides, &mut provenance_entries);

        config.overrides = overrides;
        config.provenance = ConfigProvenance {
            profile,
            entries: provenance_entries,
        };
        config
    }
}

impl Default for RelationalRuntimeConfig {
    fn default() -> Self {
        Self::resolved(
            RelationalRuntimeProfile::CertificationCore,
            RelationalConfigOverride::default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commit_strategies::data::{
        CommitStrategyDescriptor, CommitStrategyFamilyName, CommitStrategyId,
        CommitStrategyRegistration, CommitStrategySemanticName, CommitStrategyVersion,
        PersistentArtifactName, StrategyInputSchemaName, StrategyInputSchemaVersion,
        StrategyIntentName, StrategyOutputSchemaName, StrategyPacketContract, StrategyReadContract,
        StrategyReadCostClass, StrategyReadLocalityClass, StrategyReadScopeClass,
        StrategyTraversalBasis,
    };
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
            let config =
                RelationalRuntimeConfig::resolved(profile, RelationalConfigOverride::default());
            assert!(
                config.profile_boundary_matches_defaults(),
                "profile {:?} drifted from its boundary defaults",
                profile
            );
        }
    }

    #[test]
    fn overriding_diagnostics_profile_breaks_boundary_default_match() {
        let mut overrides = RelationalConfigOverride::default();
        overrides.diagnostics.profile =
            Some(RelationalDiagnosticsProfile::chip_rich_certification());

        let config =
            RelationalRuntimeConfig::resolved(RelationalRuntimeProfile::ChipSimulation, overrides);

        assert!(!config.profile_boundary_matches_defaults());
    }

    #[test]
    fn commit_strategy_registration_provenance_digest_is_order_independent() {
        let left = strategy_registration(1, "strategy.alpha", "alpha.intent");
        let right = strategy_registration(2, "strategy.beta", "beta.intent");

        let first_detail = resolved_registration_detail(vec![left.clone(), right.clone()]);
        let reversed_detail = resolved_registration_detail(vec![right, left]);

        assert_eq!(first_detail, reversed_detail);
        assert!(first_detail.contains("count=2"));
        assert!(first_detail.contains("descriptor_set_digest="));
    }

    fn resolved_registration_detail(registrations: Vec<CommitStrategyRegistration>) -> String {
        let mut overrides = RelationalConfigOverride::default();
        overrides.commit_strategies.registrations = Some(registrations);

        RelationalRuntimeConfig::resolved(RelationalRuntimeProfile::CertificationCore, overrides)
            .provenance
            .entries
            .get("commit_strategies.registrations")
            .expect("registration provenance")
            .detail
            .clone()
    }

    fn strategy_registration(
        id: u32,
        semantic_name: &str,
        intent_name: &str,
    ) -> CommitStrategyRegistration {
        CommitStrategyRegistration::new(CommitStrategyDescriptor::new(
            CommitStrategyId(id),
            CommitStrategySemanticName::new(semantic_name),
            CommitStrategyFamilyName::new("strategy.config.test"),
            CommitStrategyVersion::new(1, id as u16),
            StrategyIntentName::new(intent_name),
            StrategyInputSchemaName::new("config.input.v1"),
            StrategyInputSchemaVersion(1),
            StrategyOutputSchemaName::new("config.output.v1"),
            StrategyReadContract {
                scope_class: StrategyReadScopeClass::ExplicitTargetsOnly,
                locality_class: StrategyReadLocalityClass::SinglePartition,
                traversal_basis: StrategyTraversalBasis::NoTraversal,
                packet_contract: StrategyPacketContract::ProjectionOnly,
                cost_class: StrategyReadCostClass::ORequestedSurface,
            },
            PersistentArtifactName::new(format!("{semantic_name}.artifact")),
        ))
        .expect("valid registration")
    }
}
