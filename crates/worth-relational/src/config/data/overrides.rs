use crate::commit_strategies::data::CommitStrategyRegistration;
use crate::diagnostics::data::RelationalDiagnosticsProfile;
use crate::durability::data::{DurabilityMode, DurableStoreLayout};
use crate::history::data::{BranchId, HistoryRetentionClass, VersionGraphPolicy};
use crate::schema::data::RelationalSchemaRegistry;
use crate::symbols::data::{ClientKeySymbolPolicy, SymbolTableSnapshot};
use crate::validation::data::InvariantCatalog;

use super::policies::{
    AdjacencyPolicy, CascadeDeletePolicy, CompiledLanePolicy, CrossContextPolicy, DurabilityPolicy,
    DurableLogPolicy, MvccConfig, PublicationConfig, RetentionPolicy, StorageLayoutConfig,
    VisibilityCachePolicy,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionConfigOverride {
    pub runtime_name: Option<String>,
    pub compiled_lane_policy: Option<CompiledLanePolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiagnosticsConfigOverride {
    pub profile: Option<RelationalDiagnosticsProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HistoryConfigOverride {
    pub version_graph_policy: Option<VersionGraphPolicy>,
    pub retention: Option<HistoryRetentionClass>,
    pub main_branch: Option<BranchId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SchemaConfigOverride {
    pub registry: Option<RelationalSchemaRegistry>,
    pub invariant_catalog: Option<InvariantCatalog>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommitStrategiesConfigOverride {
    pub registrations: Option<Vec<CommitStrategyRegistration>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IdentityConfigOverride {
    pub client_key_symbol_policy: Option<ClientKeySymbolPolicy>,
    pub symbol_table: Option<SymbolTableSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StorageConfigOverride {
    pub initial_entity_capacity: Option<usize>,
    pub initial_relation_capacity: Option<usize>,
    pub mvcc: Option<MvccConfig>,
    pub retention: Option<RetentionPolicy>,
    pub layout: Option<StorageLayoutConfig>,
    pub adjacency_policy: Option<AdjacencyPolicy>,
    pub cross_context_policy: Option<CrossContextPolicy>,
    pub cascade_delete_policy: Option<CascadeDeletePolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VisibilityConfigOverride {
    pub cache_policy: Option<VisibilityCachePolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PublicationConfigOverride {
    pub policy: Option<PublicationConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DurabilityConfigOverride {
    pub policy: Option<DurabilityPolicy>,
    pub mode: Option<DurabilityMode>,
    pub log: Option<DurableLogPolicy>,
    pub store_layout: Option<DurableStoreLayout>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RelationalConfigOverride {
    pub execution: ExecutionConfigOverride,
    pub diagnostics: DiagnosticsConfigOverride,
    pub history: HistoryConfigOverride,
    pub schema: SchemaConfigOverride,
    pub commit_strategies: CommitStrategiesConfigOverride,
    pub identity: IdentityConfigOverride,
    pub storage: StorageConfigOverride,
    pub visibility: VisibilityConfigOverride,
    pub publication: PublicationConfigOverride,
    pub durability: DurabilityConfigOverride,
}

impl RelationalConfigOverride {
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}
