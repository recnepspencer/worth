pub use std::collections::BTreeMap;

pub use worth_foundational::facade::{AspectValue, CanonicalFieldPath};
pub use worth_query::facade::foundation::WorthQueryEntityIdentity;
pub use worth_query::facade::runtime::{
    WorthQueryAspectTouch, WorthQueryExistingTruthTargetBinding, WorthQueryLiveArtifactTarget,
    WorthQueryMutationTargetCollectionIdentity,
};

pub type NativeExternalRow = BTreeMap<CanonicalFieldPath, AspectValue>;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PublicExistingTruthKey {
    pub binding_digest: String,
    pub target_collection: String,
    pub aspect_touch: WorthQueryAspectTouch,
}

impl PublicExistingTruthKey {
    pub fn new(
        binding: &WorthQueryExistingTruthTargetBinding,
        aspect_touch: WorthQueryAspectTouch,
    ) -> Self {
        Self {
            binding_digest: binding.binding_digest(),
            target_collection: binding
                .terminal_target_collection_projection()
                .unwrap_or("none")
                .to_string(),
            aspect_touch,
        }
    }

    pub fn binding_digest(&self) -> &str {
        &self.binding_digest
    }

    pub fn target_collection(&self) -> &str {
        &self.target_collection
    }

    pub fn admitted_aspect_touch_reporting_projection(&self) -> String {
        let field_path = self
            .aspect_touch
            .native_field_path()
            .map(|path| {
                path.fields()
                    .iter()
                    .map(|field| field.as_str())
                    .collect::<Vec<_>>()
                    .join(".")
            })
            .unwrap_or_else(|| "<whole-aspect>".to_string());
        format!(
            "{}:{field_path}",
            self.aspect_touch.native_aspect_key().as_str()
        )
    }
}

#[derive(Default)]
pub struct PublicBridgeRuntimeState {
    pub live_views:
        BTreeMap<WorthQueryLiveArtifactTarget, WorthQueryMutationTargetCollectionIdentity>,
    pub rows_by_collection: BTreeMap<String, BTreeMap<WorthQueryEntityIdentity, NativeExternalRow>>,
    pub collection_by_identity: BTreeMap<WorthQueryEntityIdentity, String>,
    pub identity_by_symbol: BTreeMap<String, WorthQueryEntityIdentity>,
    pub existing_truth_values: BTreeMap<PublicExistingTruthKey, AspectValue>,
    pub next_entity_identity: usize,
    pub next_commit_identity: usize,
    pub next_snapshot_token: usize,
    pub current_snapshot_parts: Option<(u64, u64)>,
}
