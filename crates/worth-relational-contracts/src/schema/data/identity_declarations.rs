//! Identity vocabulary retained by declared schema and original images.
use crate::identity::data::KindId;
use serde::{Deserialize, Serialize};
use worth_foundational::facade::AspectKey;

pub mod custom_name;
pub mod declared_keys;
pub use custom_name::IdentityBasisName;
pub use declared_keys::DeclaredIdentityKeys;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CustomIdentityBasisIdentity {
    pub name: IdentityBasisName,
    pub semantic_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IdentityBasisScope {
    EntityKind(KindId),
    RelationKind(KindId),
    AspectKey(AspectKey),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IdentityBasisKind {
    StorageIdentity,
    LineageIdentity,
    StructuralFingerprint,
    DeclaredKeySet(DeclaredIdentityKeys),
    Custom(CustomIdentityBasisIdentity),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IdentityBasisDeclaration {
    pub scope: IdentityBasisScope,
    pub basis: IdentityBasisKind,
}
