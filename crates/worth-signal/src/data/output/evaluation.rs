use serde::{Deserialize, Serialize};

use crate::data::aspect::{Aspect, AspectVersion};

use super::{
    ArtifactContinuityToken, ChangedRegion, ComputationFamily, ComputationKey, OutputIdentity,
    StructuralMemoKey,
};

/// Host-declared output continuity after evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OutputChange {
    /// Default assumption when no richer semantics are supplied.
    #[default]
    Replaced,
    /// Same artifact identity refreshed with meaningful internal change.
    Refreshed,
    /// Inputs changed but the output artifact identity is unchanged.
    Unchanged,
}

/// How one evaluation result was produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MemoizedResultOrigin {
    #[default]
    DirectCompute,
    MemoizedFromCache,
}

/// Keyed execution metadata used by advanced runtime APIs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct KeyedComputation {
    pub family: ComputationFamily,
    pub key: ComputationKey,
    #[serde(default)]
    pub memo_key: Option<StructuralMemoKey>,
}

impl KeyedComputation {
    pub fn new(family: impl Into<ComputationFamily>, key: impl Into<ComputationKey>) -> Self {
        Self {
            family: family.into(),
            key: key.into(),
            memo_key: None,
        }
    }

    pub fn with_memo_key(mut self, memo_key: impl Into<StructuralMemoKey>) -> Self {
        self.memo_key = Some(memo_key.into());
        self
    }
}

/// Rich evaluation report for diff-aware execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeEvaluationResult {
    pub aspect_version: AspectVersion,
    #[serde(default)]
    pub output_identity: Option<OutputIdentity>,
    #[serde(default)]
    pub continuity_token: Option<ArtifactContinuityToken>,
    #[serde(default)]
    pub output_change: OutputChange,
    #[serde(default)]
    pub changed_regions: Vec<ChangedRegion>,
    /// Exact producer-aspect to changed-region correlation.
    #[serde(default)]
    pub changed_aspect_regions: Vec<(Aspect, ChangedRegion)>,
    #[serde(default)]
    pub labels: Vec<String>,
}

impl NodeEvaluationResult {
    pub fn from_version(aspect_version: AspectVersion) -> Self {
        Self {
            aspect_version,
            output_identity: None,
            continuity_token: None,
            output_change: OutputChange::Replaced,
            changed_regions: Vec::new(),
            changed_aspect_regions: Vec::new(),
            labels: Vec::new(),
        }
    }

    pub fn with_output_identity(mut self, output_identity: impl Into<OutputIdentity>) -> Self {
        self.output_identity = Some(output_identity.into());
        self
    }

    pub fn with_output_change(mut self, output_change: OutputChange) -> Self {
        self.output_change = output_change;
        self
    }

    pub fn with_continuity_token(
        mut self,
        continuity_token: impl Into<ArtifactContinuityToken>,
    ) -> Self {
        self.continuity_token = Some(continuity_token.into());
        self
    }

    pub fn with_changed_region(mut self, region: ChangedRegion) -> Self {
        self.changed_regions.push(region);
        self
    }

    pub fn with_changed_aspect_region(mut self, aspect: Aspect, region: ChangedRegion) -> Self {
        self.changed_aspect_regions.push((aspect, region));
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.labels.push(label.into());
        self
    }
}

pub trait IntoNodeEvaluationResult {
    fn into_evaluation_result(self) -> NodeEvaluationResult;
}

impl IntoNodeEvaluationResult for AspectVersion {
    fn into_evaluation_result(self) -> NodeEvaluationResult {
        NodeEvaluationResult::from_version(self)
    }
}

impl IntoNodeEvaluationResult for NodeEvaluationResult {
    fn into_evaluation_result(self) -> NodeEvaluationResult {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_aspect_region_authoring_does_not_flatten_into_legacy_union() {
        let risk = Aspect::new(4);
        let result = NodeEvaluationResult::from_version(AspectVersion::from_updates([(risk, 2)]))
            .with_changed_aspect_region(risk, ChangedRegion::new("rates").with_detail("2y"));

        assert!(result.changed_regions.is_empty());
        assert_eq!(result.changed_aspect_regions.len(), 1);
        assert_eq!(result.changed_aspect_regions[0].0, risk);
        assert_eq!(
            result.changed_aspect_regions[0].1.path().segments(),
            ["rates", "2y"]
        );
    }
}
