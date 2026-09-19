use std::collections::{BTreeMap, BTreeSet};
use worth_runtime_bridge::facade::RuntimeBridge;

use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::runtime::{WorthQueryLiveArtifactTarget, WorthQueryMutationTargetCollectionIdentity};
use worth_foundational::facade::{AspectValue, CanonicalFieldPath};

pub(super) type NativeExternalRow = BTreeMap<CanonicalFieldPath, AspectValue>;

pub(super) struct StatefulBridgeState {
    pub(super) live_views:
        BTreeMap<WorthQueryLiveArtifactTarget, WorthQueryMutationTargetCollectionIdentity>,
    pub(super) installed_collections: BTreeSet<String>,
    pub(super) rows_by_collection: BTreeMap<String, BTreeMap<String, NativeExternalRow>>,
    pub(super) collection_by_identity: BTreeMap<String, String>,
    pub(super) identity_by_symbol: BTreeMap<String, WorthQueryEntityIdentity>,
    pub(super) identity_text_by_symbol: BTreeMap<String, String>,
    pub(super) identity_by_storage_key: BTreeMap<String, WorthQueryEntityIdentity>,
    pub(super) next_entity_identity: usize,
    pub(super) next_commit_identity: usize,
    pub(super) next_snapshot_token: usize,
    pub(super) bridge: RuntimeBridge,
    pub(super) relational_source:
        worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner,
}

impl StatefulBridgeState {
    pub(super) fn new(
        installed_collections: BTreeSet<String>,
        bridge: RuntimeBridge,
        relational_source: worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner,
    ) -> Self {
        Self {
            installed_collections,
            live_views: BTreeMap::new(),
            rows_by_collection: BTreeMap::new(),
            collection_by_identity: BTreeMap::new(),
            identity_by_symbol: BTreeMap::new(),
            identity_text_by_symbol: BTreeMap::new(),
            identity_by_storage_key: BTreeMap::new(),
            next_entity_identity: 0,
            next_commit_identity: 0,
            next_snapshot_token: 0,
            bridge,
            relational_source,
        }
    }
}
