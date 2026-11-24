use serde::{Deserialize, Serialize};
use worth_foundational::facade::{
    AspectBinding, AspectContractRevision, AspectIdentity, AspectKey,
    AuthoritativeAspectChangeKind, CanonicalFieldPath,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PublishedAspectChangePrecision {
    Exact,
    DeclaredWidening,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishedAuthoritativeAspectChange {
    aspect_key: AspectKey,
    aspect_identity: AspectIdentity,
    contract_revision: AspectContractRevision,
    binding: AspectBinding,
    kind: AuthoritativeAspectChangeKind,
    field_path: Option<CanonicalFieldPath>,
    precision: PublishedAspectChangePrecision,
}

impl PublishedAuthoritativeAspectChange {
    pub fn aspect_key(&self) -> &AspectKey {
        &self.aspect_key
    }
    pub const fn aspect_identity(&self) -> AspectIdentity {
        self.aspect_identity
    }
    pub const fn contract_revision(&self) -> AspectContractRevision {
        self.contract_revision
    }
    pub fn binding(&self) -> &AspectBinding {
        &self.binding
    }
    pub const fn kind(&self) -> AuthoritativeAspectChangeKind {
        self.kind
    }
    pub fn field_path(&self) -> Option<&CanonicalFieldPath> {
        self.field_path.as_ref()
    }
    pub const fn precision(&self) -> PublishedAspectChangePrecision {
        self.precision
    }
}

/// Operations used by the Relational owners.
pub trait PublishedAuthoritativeAspectChangeOwnerPort: Sized {
    fn exact(
        aspect_key: AspectKey,
        aspect_identity: AspectIdentity,
        contract_revision: AspectContractRevision,
        binding: AspectBinding,
        kind: AuthoritativeAspectChangeKind,
        field_path: Option<CanonicalFieldPath>,
    ) -> Self;
    fn canonical_key(&self) -> String;
    fn owned_allocation_capacity_bytes(&self) -> u64;
}

impl PublishedAuthoritativeAspectChangeOwnerPort for PublishedAuthoritativeAspectChange {
    fn exact(
        aspect_key: AspectKey,
        aspect_identity: AspectIdentity,
        contract_revision: AspectContractRevision,
        binding: AspectBinding,
        kind: AuthoritativeAspectChangeKind,
        field_path: Option<CanonicalFieldPath>,
    ) -> Self {
        Self {
            aspect_key,
            aspect_identity,
            contract_revision,
            binding,
            kind,
            field_path,
            precision: PublishedAspectChangePrecision::Exact,
        }
    }

    fn canonical_key(&self) -> String {
        let path = self
            .field_path
            .as_ref()
            .map(|path| {
                path.fields()
                    .iter()
                    .map(|field| field.as_str())
                    .collect::<Vec<_>>()
                    .join(".")
            })
            .unwrap_or_else(|| "whole".to_string());
        let fields = [
            self.aspect_key.as_str().to_string(),
            self.aspect_identity.0.to_string(),
            self.contract_revision.0.to_string(),
            self.binding.canonical_name(),
            self.kind.canonical_name().to_string(),
            path,
        ];
        fields
            .into_iter()
            .map(|field| format!("{}:{field}", field.len()))
            .collect()
    }

    fn owned_allocation_capacity_bytes(&self) -> u64 {
        (self.aspect_key.owned_allocation_capacity_bytes()
            + self.binding.owned_allocation_capacity_bytes()
            + self
                .field_path
                .as_ref()
                .map_or(0, CanonicalFieldPath::owned_allocation_capacity_bytes)) as u64
    }
}
