use super::overrides::RelationalConfigOverride;
use super::policies::RuntimeProfileBoundaryPolicy;
use super::provenance::ConfigProvenance;
use super::sections::{
    CommitStrategiesConfig, DiagnosticsConfig, DurabilityConfig, ExecutionConfig, HistoryConfig,
    IdentityConfig, PublicationRuntimeConfig, SchemaConfig, StorageConfig, VisibilityConfig,
};
use super::RelationalRuntimeProfile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalRuntimeConfig {
    pub profile: RelationalRuntimeProfile,
    pub execution: ExecutionConfig,
    pub diagnostics: DiagnosticsConfig,
    pub history: HistoryConfig,
    pub schema: SchemaConfig,
    pub commit_strategies: CommitStrategiesConfig,
    pub identity: IdentityConfig,
    pub storage: StorageConfig,
    pub visibility: VisibilityConfig,
    pub publication: PublicationRuntimeConfig,
    pub durability: DurabilityConfig,
    pub overrides: RelationalConfigOverride,
    pub provenance: ConfigProvenance,
}

impl RelationalRuntimeConfig {
    pub fn boundary_policy(&self) -> RuntimeProfileBoundaryPolicy {
        self.profile.boundary_policy()
    }

    pub fn profile_boundary_matches_defaults(&self) -> bool {
        let boundary = self.boundary_policy();
        let diagnostics = self.profile.default_diagnostics_profile();

        let diagnostics_match = self.diagnostics.profile == diagnostics;
        let log_match = self.durability.policy.log.compact_after_checkpoint
            == boundary.prefers_checkpoint_compaction;
        let compiled_lane_match = match boundary.allows_compiled_lane {
            true => self.execution.compiled_lane_policy != super::CompiledLanePolicy::Disabled,
            false => self.execution.compiled_lane_policy == super::CompiledLanePolicy::Disabled,
        };

        diagnostics_match && log_match && compiled_lane_match
    }
}
