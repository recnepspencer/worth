use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(crate) struct OrientationContract {
    pub(crate) machine_constitution: String,
    pub(crate) route_proof: BTreeMap<String, ExemplarRouteSpec>,
    pub(crate) band_rules: BTreeMap<String, Vec<String>>,
    pub(crate) replay_surface_summary: String,
    pub(crate) seed_skeleton_paths: Vec<String>,
    pub(crate) query_audience: QueryAudienceContractSpec,
    pub(crate) subworkspace_paths: Vec<String>,
    pub(crate) context_workspaces: Vec<ContextWorkspaceSpec>,
}

#[derive(Clone, Debug)]
pub(crate) struct ContextWorkspaceSpec {
    pub(crate) path: String,
    pub(crate) package_prefix: String,
    pub(crate) certification_packages: Vec<String>,
}

pub(crate) struct ExemplarRouteSpec {
    pub(crate) specimen: String,
    pub(crate) deferred_routes: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct QueryAudienceFacadeSpec {
    pub(crate) package: String,
    pub(crate) label: String,
    pub(crate) allowed_bands: Vec<String>,
    pub(crate) guidance: String,
    pub(crate) authority_packages: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct QueryAudienceContractSpec {
    pub(crate) workspace: String,
    pub(crate) engine_package: String,
    pub(crate) certification_package: Option<String>,
    pub(crate) certification_authority_packages: Vec<String>,
    pub(crate) certification_consumers: Vec<String>,
    pub(crate) audiences: Vec<QueryAudienceFacadeSpec>,
}

pub(crate) fn load_orientation_contract(path: &Path) -> Result<OrientationContract, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let config: BoundaryConfig =
        toml::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;

    let route_proof = config
        .routing_proof
        .exemplars
        .into_iter()
        .map(|exemplar| {
            (
                exemplar.package,
                ExemplarRouteSpec {
                    specimen: exemplar.specimen,
                    deferred_routes: exemplar.deferred_routes,
                },
            )
        })
        .collect();
    let band_rules = config
        .rule_contracts
        .band_rules
        .into_iter()
        .map(|rule| (rule.source_band, rule.allowed_target_bands))
        .collect();
    let replay_surface_summary = config
        .rule_contracts
        .replay_surfaces
        .iter()
        .map(|surface| {
            format!(
                "{} [{}; cert domains: {}]",
                surface.label,
                surface.package_prefixes.join(", "),
                surface.cert_domains.join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    let query_audience = QueryAudienceContractSpec {
        workspace: config.rule_contracts.query_audience.workspace,
        engine_package: config.rule_contracts.query_audience.engine_package,
        certification_package: config.rule_contracts.query_audience.certification_package,
        certification_authority_packages: config
            .rule_contracts
            .query_audience
            .certification_authority_packages,
        certification_consumers: config.rule_contracts.query_audience.certification_consumers,
        audiences: config
            .rule_contracts
            .query_audience
            .audiences
            .into_iter()
            .map(|audience| QueryAudienceFacadeSpec {
                package: audience.package,
                label: audience.label,
                allowed_bands: audience.allowed_bands,
                guidance: audience.guidance,
                authority_packages: audience.authority_packages,
            })
            .collect(),
    };

    Ok(OrientationContract {
        machine_constitution: config.machine_authority.canonical_config,
        route_proof,
        band_rules,
        replay_surface_summary,
        seed_skeleton_paths: config
            .seed_skeletons
            .into_iter()
            .map(|skeleton| skeleton.path)
            .collect(),
        query_audience,
        subworkspace_paths: config
            .subworkspaces
            .into_iter()
            .map(|subworkspace| subworkspace.path)
            .collect(),
        context_workspaces: config
            .context_workspaces
            .into_iter()
            .map(|workspace| ContextWorkspaceSpec {
                path: workspace.path,
                package_prefix: workspace.package_prefix,
                certification_packages: workspace.certification_packages,
            })
            .collect(),
    })
}

#[derive(Deserialize)]
struct BoundaryConfig {
    machine_authority: MachineAuthorityConfig,
    rule_contracts: RuleContracts,
    routing_proof: RoutingProofConfig,
    seed_skeletons: Vec<SeedSkeletonConfig>,
    subworkspaces: Vec<SubworkspaceConfig>,
    #[serde(default)]
    context_workspaces: Vec<ContextWorkspaceConfig>,
}

#[derive(Deserialize)]
struct MachineAuthorityConfig {
    canonical_config: String,
}

#[derive(Deserialize)]
struct RuleContracts {
    query_audience: QueryAudienceConfig,
    replay_surfaces: Vec<ReplaySurfaceConfig>,
    band_rules: Vec<BandRuleConfig>,
}

#[derive(Deserialize)]
struct QueryAudienceConfig {
    #[serde(default = "default_query_workspace")]
    workspace: String,
    engine_package: String,
    #[serde(default)]
    certification_package: Option<String>,
    #[serde(default)]
    certification_authority_packages: Vec<String>,
    #[serde(default)]
    certification_consumers: Vec<String>,
    audiences: Vec<QueryAudienceFacadeConfig>,
}

fn default_query_workspace() -> String {
    ".".to_owned()
}

#[derive(Deserialize)]
struct QueryAudienceFacadeConfig {
    package: String,
    label: String,
    allowed_bands: Vec<String>,
    guidance: String,
    authority_packages: Vec<String>,
}

#[derive(Deserialize)]
struct ReplaySurfaceConfig {
    label: String,
    package_prefixes: Vec<String>,
    cert_domains: Vec<String>,
}

#[derive(Deserialize)]
struct BandRuleConfig {
    source_band: String,
    allowed_target_bands: Vec<String>,
}

#[derive(Deserialize)]
struct SeedSkeletonConfig {
    path: String,
}

#[derive(Deserialize)]
struct SubworkspaceConfig {
    path: String,
}

#[derive(Deserialize)]
struct ContextWorkspaceConfig {
    path: String,
    package_prefix: String,
    #[serde(default)]
    certification_packages: Vec<String>,
}

#[derive(Deserialize)]
struct RoutingProofConfig {
    exemplars: Vec<ExemplarRouteConfig>,
}

#[derive(Deserialize)]
struct ExemplarRouteConfig {
    package: String,
    specimen: String,
    deferred_routes: Vec<String>,
}
