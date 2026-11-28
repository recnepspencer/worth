//! Aspect declarations and historical wire labels used in schema images.
use serde::{Deserialize, Serialize};
use worth_foundational::facade::AspectKey;

pub mod custom_name;
pub use custom_name::MergePolicyName;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomMergePolicyIdentity {
    pub name: MergePolicyName,
    pub semantic_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AspectMergePolicyKind {
    FailOnConflict,
    #[doc(hidden)]
    #[serde(rename = "LastWriterWins")]
    RetiredWriterOrder,
    MonotonicCounter,
    AdditiveSet,
    #[doc(hidden)]
    #[serde(rename = "PreferRicher")]
    RetiredValueShape,
    Custom(CustomMergePolicyIdentity),
}

impl AspectMergePolicyKind {
    pub const fn ownership_class(&self) -> MergePolicyOwnershipClass {
        match self {
            AspectMergePolicyKind::Custom(_) => MergePolicyOwnershipClass::CustomPolicy,
            AspectMergePolicyKind::FailOnConflict
            | AspectMergePolicyKind::RetiredWriterOrder
            | AspectMergePolicyKind::MonotonicCounter
            | AspectMergePolicyKind::AdditiveSet
            | AspectMergePolicyKind::RetiredValueShape => MergePolicyOwnershipClass::RuntimeBuiltIn,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectMergePolicyDeclaration {
    pub aspect_key: AspectKey,
    pub policy: AspectMergePolicyKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergePolicyOwnershipClass {
    RuntimeBuiltIn,
    CustomPolicy,
}
