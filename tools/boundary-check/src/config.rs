use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Road1Config {
    pub(crate) machine_authority: MachineAuthorityConfig,
    pub(crate) root_manifest: String,
    pub(crate) forbidden_root_prefixes: Vec<String>,
    pub(crate) naming: NamingConfig,
    pub(crate) rule_contracts: RuleContracts,
    pub(crate) born_crates: Vec<BornCrateConfig>,
    pub(crate) seed_skeletons: Vec<SeedSkeletonConfig>,
    pub(crate) subworkspaces: Vec<SubworkspaceConfig>,
    #[serde(default)]
    pub(crate) context_workspaces: Vec<ContextWorkspaceConfig>,
    pub(crate) legacy_reference_ratchet: LegacyReferenceRatchetConfig,
    /// Compile-time law substrates legal outside the band grammar (e.g. worth-proof).
    #[serde(default)]
    pub(crate) law_substrates: Vec<LawSubstrateConfig>,
    #[serde(default)]
    pub(crate) dependency_denials: Vec<DependencyDenialConfig>,
    #[serde(default)]
    pub(crate) dependency_target_allowlists: Vec<DependencyTargetAllowlistConfig>,
    #[serde(default)]
    pub(crate) source_dependency_allowlists: Vec<SourceDependencyAllowlistConfig>,
    #[serde(default)]
    pub(crate) source_identifier_denials: Vec<SourceIdentifierDenialConfig>,
    #[serde(default)]
    pub(crate) source_owner_isolations: Vec<SourceOwnerIsolationConfig>,
    #[serde(default)]
    pub(crate) raw_geometry_denials: Vec<RawGeometryDenialConfig>,
    #[serde(default)]
    pub(crate) truth_type_denials: Vec<TruthTypeDenialConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ContextWorkspaceConfig {
    pub(crate) path: String,
    pub(crate) package_prefix: String,
    #[serde(default)]
    pub(crate) certification_packages: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct DependencyDenialConfig {
    pub(crate) workspace_manifest: String,
    pub(crate) sources: Vec<String>,
    #[serde(default)]
    pub(crate) source_prefixes: Vec<String>,
    pub(crate) forbidden_targets: Vec<String>,
    #[serde(default)]
    pub(crate) forbidden_target_prefixes: Vec<String>,
    pub(crate) guidance: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct DependencyTargetAllowlistConfig {
    pub(crate) workspace_manifest: String,
    pub(crate) governed_source_prefixes: Vec<String>,
    pub(crate) target: String,
    pub(crate) allowed_sources: Vec<String>,
    pub(crate) guidance: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct SourceDependencyAllowlistConfig {
    pub(crate) workspace_manifest: String,
    pub(crate) sources: Vec<String>,
    pub(crate) allowed_targets: Vec<String>,
    #[serde(default)]
    pub(crate) dependency_contracts: Vec<SourceDependencyContractConfig>,
    pub(crate) guidance: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct SourceDependencyContractConfig {
    pub(crate) target: String,
    pub(crate) version_requirement: String,
    pub(crate) uses_default_features: bool,
    pub(crate) features: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct SourceIdentifierDenialConfig {
    pub(crate) root: String,
    #[serde(default)]
    pub(crate) exclude_paths: Vec<String>,
    pub(crate) forbidden_identifiers: Vec<String>,
    #[serde(default)]
    pub(crate) forbidden_identifier_fragments: Vec<String>,
    pub(crate) guidance: String,
}

/// Source roots that may never reach an isolated owner, by path or by any
/// type the owner declares.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceOwnerIsolationConfig {
    pub(crate) owner_roots: Vec<String>,
    pub(crate) guarded_roots: Vec<String>,
    /// Consecutive path segments that name the owner, such as
    /// `["primary_graph", "workflow"]`.
    pub(crate) forbidden_paths: Vec<Vec<String>>,
    pub(crate) guidance: String,
}

/// One crate whose production source may hold raw coordinates only at the
/// declared edges.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawGeometryDenialConfig {
    pub(crate) crate_root: String,
    pub(crate) guidance: String,
    pub(crate) edges: Vec<RawGeometryEdgeConfig>,
}

/// A source file or directory, relative to the crate root, and optionally the
/// item paths within it, where raw coordinates are the declared form.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawGeometryEdgeConfig {
    pub(crate) path: String,
    #[serde(default)]
    pub(crate) items: Vec<String>,
    pub(crate) kind: RawGeometryEdgeKind,
    pub(crate) reason: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RawGeometryEdgeKind {
    Serialization,
    PlatformEvent,
    GpuUpload,
    SpatialIndex,
    SealedOwner,
    CommittedLayout,
}

impl RawGeometryEdgeKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Serialization => "serialization",
            Self::PlatformEvent => "platform-event",
            Self::GpuUpload => "gpu-upload",
            Self::SpatialIndex => "spatial-index",
            Self::SealedOwner => "sealed-owner",
            Self::CommittedLayout => "committed-layout",
        }
    }
}

/// One crate whose sealed truth types are constructed only by their owners and
/// at declared callers, and whose lifecycle state has no `Default`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TruthTypeDenialConfig {
    pub(crate) crate_root: String,
    pub(crate) guidance: String,
    #[serde(default)]
    pub(crate) sealed: Vec<SealedTruthTypeConfig>,
    #[serde(default)]
    pub(crate) mints: Vec<SealedMintConfig>,
    #[serde(default)]
    pub(crate) lifecycle_states: Vec<String>,
}

/// A requirement 1 or 3 type and the sources, relative to the crate root,
/// that own it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SealedTruthTypeConfig {
    pub(crate) name: String,
    pub(crate) requirement: u8,
    pub(crate) owners: Vec<String>,
}

/// A constructor called outside its owner, and the only sites that call it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SealedMintConfig {
    /// `Type::constructor`, as a path names it.
    pub(crate) call: String,
    /// The method's path in the root `clippy.toml` `disallowed-methods`, for a
    /// mint called as a method, whose receiver only Clippy resolves. Each
    /// caller then carries the `expect` that admits it.
    #[serde(default)]
    pub(crate) clippy: Option<String>,
    pub(crate) callers: Vec<SealedMintCallerConfig>,
    pub(crate) reason: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SealedMintCallerConfig {
    pub(crate) path: String,
    #[serde(default)]
    pub(crate) items: Vec<String>,
}

/// One machine-owned law substrate: package identity plus legal tier/band sets.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct LawSubstrateConfig {
    pub(crate) package: String,
    pub(crate) tiers: Vec<String>,
    pub(crate) bands: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct LegacyReferenceRatchetConfig {
    pub(crate) governed_roots: Vec<String>,
    pub(crate) forbidden_fragments: Vec<String>,
    pub(crate) snapshot: String,
    pub(crate) exclude_paths: Vec<String>,
    pub(crate) replacement_guidance: String,
}

#[derive(Deserialize)]
pub(crate) struct MachineAuthorityConfig {
    pub(crate) canonical_config: String,
    pub(crate) mirrored_docs: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct NamingConfig {
    pub(crate) bands: Vec<String>,
    pub(crate) reserved_domains: Vec<ReservedDomainConfig>,
}

#[derive(Deserialize)]
pub(crate) struct ReservedDomainConfig {
    pub(crate) tier: String,
    pub(crate) band: String,
    pub(crate) domains: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct RuleContracts {
    pub(crate) query_audience: QueryAudienceContract,
    pub(crate) replay_surfaces: Vec<ReplaySurfaceConfig>,
    pub(crate) band_rules: Vec<BandRuleConfig>,
    #[serde(default)]
    pub(crate) worth_ui_query_edge: Option<WorthUiQueryEdgeContract>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WorthUiQueryEdgeContract {
    pub(crate) workspace: String,
    pub(crate) engine_package: String,
    pub(crate) allowed_production_consumers: Vec<String>,
    pub(crate) guidance: String,
}

/// Machine-owned Query audience matrix: one engine package plus leaf facade rows.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct QueryAudienceContract {
    #[serde(default = "default_query_workspace")]
    pub(crate) workspace: String,
    pub(crate) engine_package: String,
    #[serde(default)]
    pub(crate) certification_package: Option<String>,
    #[serde(default)]
    pub(crate) certification_authority_packages: Vec<String>,
    #[serde(default)]
    pub(crate) certification_consumers: Vec<String>,
    #[serde(default)]
    pub(crate) internal_packages: Vec<String>,
    #[serde(default)]
    pub(crate) facade_surfaces: Vec<QueryFacadeSurfaceConfig>,
    pub(crate) audiences: Vec<QueryAudienceFacadeConfig>,
}

fn default_query_workspace() -> String {
    ".".to_owned()
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct QueryFacadeSurfaceConfig {
    pub(crate) label: String,
    pub(crate) source: String,
    #[serde(default)]
    pub(crate) namespace: Option<String>,
    #[serde(default)]
    pub(crate) reexport: Option<String>,
    #[serde(default)]
    pub(crate) owner_source: Option<String>,
}

/// One audience facade row: package identity, legal bands, and repair guidance.
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct QueryAudienceFacadeConfig {
    pub(crate) package: String,
    pub(crate) label: String,
    pub(crate) allowed_bands: Vec<String>,
    pub(crate) guidance: String,
    #[serde(default)]
    pub(crate) authority_packages: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct BandRuleConfig {
    pub(crate) source_band: String,
    pub(crate) allowed_target_bands: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct ReplaySurfaceConfig {
    pub(crate) label: String,
    pub(crate) package_prefixes: Vec<String>,
    pub(crate) cert_domains: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct BornCrateConfig {
    pub(crate) path: String,
    pub(crate) package: String,
}

#[derive(Deserialize)]
pub(crate) struct SeedSkeletonConfig {
    pub(crate) path: String,
    pub(crate) package: String,
    pub(crate) lib_rs: String,
    pub(crate) facade_rs: String,
    pub(crate) allowed_entries: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct SubworkspaceConfig {
    pub(crate) path: String,
    pub(crate) allowed_crate_prefixes: Vec<String>,
    pub(crate) member_lane: String,
}
