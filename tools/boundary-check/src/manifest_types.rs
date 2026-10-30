use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct WorkspaceManifest {
    pub(crate) workspace: Option<WorkspaceSection>,
}

#[derive(Deserialize)]
pub(crate) struct WorkspaceSection {
    pub(crate) exclude: Option<Vec<String>>,
    pub(crate) members: Option<Vec<String>>,
    pub(crate) metadata: Option<WorkspaceMetadata>,
}

#[derive(Deserialize)]
pub(crate) struct WorkspaceMetadata {
    pub(crate) worth_topology: Option<WorthTopologyMetadata>,
}

#[derive(Deserialize)]
pub(crate) struct WorthTopologyMetadata {
    pub(crate) role: Option<String>,
    pub(crate) constitutional_lane: Option<String>,
    pub(crate) member_lane: Option<String>,
    pub(crate) allowed_crate_prefixes: Option<Vec<String>>,
    pub(crate) forbidden_member_prefixes: Option<Vec<String>>,
    pub(crate) boundary_check_manifest: Option<String>,
    pub(crate) boundary_check_config: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct Road1Package {
    pub(crate) name: String,
    pub(crate) dependencies: Vec<String>,
    pub(crate) manifest_path: String,
}

#[derive(Deserialize)]
pub(crate) struct CargoMetadata {
    pub(crate) workspace_members: Vec<String>,
    pub(crate) packages: Vec<CargoMetadataPackage>,
    pub(crate) resolve: Option<CargoMetadataResolve>,
}

#[derive(Deserialize)]
pub(crate) struct CargoMetadataResolve {
    pub(crate) nodes: Vec<CargoMetadataNode>,
}

#[derive(Deserialize)]
pub(crate) struct CargoMetadataNode {
    pub(crate) id: String,
    pub(crate) deps: Vec<CargoMetadataResolvedDependency>,
}

#[derive(Deserialize)]
pub(crate) struct CargoMetadataResolvedDependency {
    pub(crate) pkg: String,
    pub(crate) dep_kinds: Vec<CargoMetadataDependencyKind>,
}

#[derive(Deserialize)]
pub(crate) struct CargoMetadataDependencyKind {
    pub(crate) kind: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct CargoMetadataPackage {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) manifest_path: String,
    pub(crate) dependencies: Vec<CargoMetadataDependency>,
    #[serde(default)]
    pub(crate) targets: Vec<CargoMetadataTarget>,
}

#[derive(Deserialize)]
pub(crate) struct CargoMetadataTarget {
    pub(crate) kind: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct CargoMetadataDependency {
    pub(crate) name: String,
    pub(crate) kind: Option<String>,
    pub(crate) req: String,
    #[serde(default)]
    pub(crate) features: Vec<String>,
    pub(crate) uses_default_features: bool,
}
