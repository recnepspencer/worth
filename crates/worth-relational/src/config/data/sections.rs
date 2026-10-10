use crate::commit_strategies::data::CommitStrategyRegistration;
use crate::diagnostics::data::RelationalDiagnosticsProfile;
use crate::history::data::{BranchId, HistoryRetentionClass, VersionGraphPolicy};
use crate::schema::data::{
    DescriptorCanonicalBasisSupportPolicy, DescriptorSemanticsSupportPolicy,
    RelationalSchemaRegistry,
};
use crate::symbols::data::{ClientKeySymbolPolicy, SymbolTableSnapshot};
use crate::validation::data::InvariantCatalog;

use super::policies::{
    AdjacencyPolicy, CascadeDeletePolicy, CompiledLanePolicy, CrossContextPolicy, DurabilityPolicy,
    MvccConfig, PublicationConfig, RetentionPolicy, StorageLayoutConfig, VisibilityCachePolicy,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionConfig {
    pub runtime_name: String,
    pub compiled_lane_policy: CompiledLanePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticsConfig {
    pub profile: RelationalDiagnosticsProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryConfig {
    pub version_graph_policy: VersionGraphPolicy,
    pub retention: HistoryRetentionClass,
    pub main_branch: BranchId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaConfig {
    pub registry: RelationalSchemaRegistry,
    pub invariant_catalog: InvariantCatalog,
    pub descriptor_semantics_policy: DescriptorSemanticsSupportPolicy,
    pub descriptor_canonical_basis_policy: DescriptorCanonicalBasisSupportPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommitStrategiesConfig {
    pub registrations: Vec<CommitStrategyRegistration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityConfig {
    pub client_key_symbol_policy: ClientKeySymbolPolicy,
    pub symbol_table: SymbolTableSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageConfig {
    pub initial_entity_capacity: usize,
    pub initial_relation_capacity: usize,
    pub mvcc: MvccConfig,
    pub retention: RetentionPolicy,
    pub layout: StorageLayoutConfig,
    pub adjacency_policy: AdjacencyPolicy,
    pub cross_context_policy: CrossContextPolicy,
    pub cascade_delete_policy: CascadeDeletePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibilityConfig {
    pub cache_policy: VisibilityCachePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationRuntimeConfig {
    pub policy: PublicationConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurabilityConfig {
    pub policy: DurabilityPolicy,
}
