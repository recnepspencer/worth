use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeResolvedAspectValueStrategy {
    SourceVisibleValue,
    TargetVisibleValue,
    BaseVisibleValue,
    InlineAspectValue(
        #[serde(with = "crate::aspect_wire::serde_canonical_aspect_value")]
        worth_foundational::facade::AspectValue,
    ),
}
