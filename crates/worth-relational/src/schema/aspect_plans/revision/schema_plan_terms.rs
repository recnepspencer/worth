use worth_foundational::facade::{AspectKey, FieldKey};

use crate::merge::data::identity::{
    IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope,
};
use crate::merge::data::policy::{AspectMergePolicyDeclaration, AspectMergePolicyKind};
use crate::schema::data::{AspectBinding, DeclaredAspectContractBinding};

const FNV_OFFSET: u128 = 0x6c62272e07bb014262b821756295c58d;
const FNV_PRIME: u128 = 0x0000000001000000000000000000013B;

pub(super) struct RevisionHasher {
    hash: u128,
}

impl RevisionHasher {
    pub(super) const fn new() -> Self {
        Self { hash: FNV_OFFSET }
    }

    pub(super) const fn finish(self) -> u128 {
        self.hash
    }

    pub(super) fn mix_u8(&mut self, value: u8) {
        self.mix_bytes(&[value]);
    }

    pub(super) fn mix_bool(&mut self, value: bool) {
        self.mix_u8(u8::from(value));
    }

    pub(super) fn mix_u32(&mut self, value: u32) {
        self.mix_bytes(&value.to_le_bytes());
    }

    pub(super) fn mix_i32(&mut self, value: i32) {
        self.mix_bytes(&value.to_le_bytes());
    }

    pub(super) fn mix_u64(&mut self, value: u64) {
        self.mix_bytes(&value.to_le_bytes());
    }

    pub(super) fn mix_i64(&mut self, value: i64) {
        self.mix_bytes(&value.to_le_bytes());
    }

    pub(super) fn mix_u128(&mut self, value: u128) {
        self.mix_bytes(&value.to_le_bytes());
    }

    pub(super) fn mix_i128(&mut self, value: i128) {
        self.mix_bytes(&value.to_le_bytes());
    }

    pub(super) fn mix_text(&mut self, value: &str) {
        self.mix_bytes(value.as_bytes());
    }

    pub(super) fn mix_aspect_key(&mut self, value: &AspectKey) {
        self.mix_text("aspect_key");
        self.mix_text(value.as_str());
    }

    pub(super) fn mix_field_path<'a>(&mut self, fields: impl IntoIterator<Item = &'a FieldKey>) {
        self.mix_text("field_path");
        for field in fields {
            self.mix_field_key(field);
        }
    }

    pub(super) fn mix_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.hash ^= *byte as u128;
            self.hash = self.hash.wrapping_mul(FNV_PRIME);
        }
        self.hash ^= 0xff_u128;
        self.hash = self.hash.wrapping_mul(FNV_PRIME);
    }

    fn mix_field_key(&mut self, value: &FieldKey) {
        self.mix_text("field_key");
        self.mix_text(value.as_str());
    }
}

pub(super) fn mix_aspect_binding_terms(
    revision: &mut RevisionHasher,
    aspect: &DeclaredAspectContractBinding,
) {
    revision.mix_text("declared_aspect");
    revision.mix_aspect_key(&aspect.aspect_key());
    match &aspect.binding {
        AspectBinding::EntityField { field } => {
            revision.mix_text("entity_field");
            revision.mix_field_key(field);
        }
        AspectBinding::RelationField { field } => {
            revision.mix_text("relation_field");
            revision.mix_field_key(field);
        }
        AspectBinding::RelationSourceEndpoint => revision.mix_text("source_endpoint"),
        AspectBinding::RelationTargetEndpoint => revision.mix_text("target_endpoint"),
        AspectBinding::LifecycleTransition => revision.mix_text("lifecycle"),
        _ => revision.mix_text(&aspect.binding.canonical_name()),
    }
}

pub(super) fn mix_identity_declaration_terms(
    revision: &mut RevisionHasher,
    declaration: &IdentityBasisDeclaration,
) {
    revision.mix_text("identity_declaration");
    match &declaration.scope {
        IdentityBasisScope::EntityKind(kind_id) => {
            revision.mix_text("identity_scope_entity_kind");
            revision.mix_u32(kind_id.0);
        }
        IdentityBasisScope::RelationKind(kind_id) => {
            revision.mix_text("identity_scope_relation_kind");
            revision.mix_u32(kind_id.0);
        }
        IdentityBasisScope::AspectKey(aspect_key) => {
            revision.mix_text("identity_scope_aspect_key");
            revision.mix_aspect_key(aspect_key);
        }
    }
    match &declaration.basis {
        IdentityBasisKind::StorageIdentity => revision.mix_text("identity_basis_storage"),
        IdentityBasisKind::LineageIdentity => revision.mix_text("identity_basis_lineage"),
        IdentityBasisKind::StructuralFingerprint => revision.mix_text("identity_basis_structural"),
        IdentityBasisKind::DeclaredKeySet(keys) => {
            revision.mix_text("identity_basis_declared_key_set");
            for key in keys.iter() {
                revision.mix_aspect_key(key);
            }
        }
        IdentityBasisKind::Custom(custom) => {
            revision.mix_text("identity_basis_custom");
            revision.mix_text(&custom.name);
            revision.mix_u32(custom.semantic_version);
        }
    }
}

pub(super) fn mix_merge_policy_declaration_terms(
    revision: &mut RevisionHasher,
    declaration: &AspectMergePolicyDeclaration,
) {
    revision.mix_text("merge_policy_declaration");
    revision.mix_aspect_key(&declaration.aspect_key);
    match &declaration.policy {
        AspectMergePolicyKind::FailOnConflict => revision.mix_text("merge_policy_fail"),
        AspectMergePolicyKind::LastWriterWins => revision.mix_text("merge_policy_lww"),
        AspectMergePolicyKind::MonotonicCounter => {
            revision.mix_text("merge_policy_monotonic_counter")
        }
        AspectMergePolicyKind::AdditiveSet => revision.mix_text("merge_policy_additive_set"),
        AspectMergePolicyKind::PreferRicher => revision.mix_text("merge_policy_prefer_richer"),
        AspectMergePolicyKind::Custom(custom) => {
            revision.mix_text("merge_policy_custom");
            revision.mix_text(&custom.name);
            revision.mix_u32(custom.semantic_version);
        }
    }
}
