use crate::identity::data::KindId;
use crate::merge::data::artifacts::schema_snapshot_digest_basis::{
    MergeSchemaKindClass, MergeSchemaKindSemanticSnapshot,
};
use crate::merge::data::identity::{
    IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope,
};
use crate::merge::data::policy::AspectMergePolicyDeclaration;
use crate::schema::data::{AspectContractPlanRevision, RelationIntegrityPlanRevision};

impl super::CanonicalDigestBytes {
    pub(super) fn optional_schema_id(&mut self, value: Option<&crate::schema::data::SchemaId>) {
        match value {
            Some(value) => {
                self.tag(1);
                self.str(&value.0);
            }
            None => self.tag(0),
        }
    }

    pub(super) fn optional_schema_version_id(
        &mut self,
        value: Option<crate::schema::data::SchemaVersionId>,
    ) {
        match value {
            Some(value) => {
                self.tag(1);
                self.u32(value.0);
            }
            None => self.tag(0),
        }
    }

    pub(super) fn schema_kind_snapshots(&mut self, values: &[MergeSchemaKindSemanticSnapshot]) {
        self.usize(values.len());
        for value in values {
            self.merge_schema_kind_class(value.kind_class);
            self.kind_id(value.kind_id);
            self.str(&value.kind_name);
            self.str(&value.schema_id.0);
            self.u32(value.schema_version_id.0);
            self.aspect_plan_revision(value.aspect_plan_revision);
            self.identity_declarations(&value.identity_declarations);
            self.aspect_merge_policy_declarations(&value.merge_policy_declarations);
            self.optional_relation_integrity_plan_revision(value.relation_integrity_plan_revision);
        }
    }

    fn kind_id(&mut self, value: KindId) {
        self.u32(value.0);
    }

    fn identity_declarations(&mut self, values: &[IdentityBasisDeclaration]) {
        self.usize(values.len());
        for value in values {
            self.identity_basis_scope(&value.scope);
            self.identity_basis_kind(&value.basis);
        }
    }

    fn identity_basis_scope(&mut self, value: &IdentityBasisScope) {
        match value {
            IdentityBasisScope::EntityKind(kind_id) => {
                self.tag(1);
                self.kind_id(*kind_id);
            }
            IdentityBasisScope::RelationKind(kind_id) => {
                self.tag(2);
                self.kind_id(*kind_id);
            }
            IdentityBasisScope::AspectKey(aspect_key) => {
                self.tag(3);
                self.str(aspect_key.as_str());
            }
        }
    }

    fn identity_basis_kind(&mut self, value: &IdentityBasisKind) {
        match value {
            IdentityBasisKind::StorageIdentity => self.tag(1),
            IdentityBasisKind::LineageIdentity => self.tag(2),
            IdentityBasisKind::StructuralFingerprint => self.tag(3),
            IdentityBasisKind::DeclaredKeySet(keys) => {
                self.tag(4);
                self.usize(keys.len());
                for key in keys.iter() {
                    self.str(key.as_str());
                }
            }
            IdentityBasisKind::Custom(custom) => {
                self.tag(5);
                self.str(&custom.name);
                self.u32(custom.semantic_version);
            }
        }
    }

    fn aspect_merge_policy_declarations(&mut self, values: &[AspectMergePolicyDeclaration]) {
        self.usize(values.len());
        for value in values {
            self.str(value.aspect_key.as_str());
            self.aspect_merge_policy_kind(&value.policy);
        }
    }

    fn aspect_plan_revision(&mut self, value: AspectContractPlanRevision) {
        self.u128(value.0);
    }

    fn optional_relation_integrity_plan_revision(
        &mut self,
        value: Option<RelationIntegrityPlanRevision>,
    ) {
        match value {
            Some(value) => {
                self.tag(1);
                self.u128(value.0);
            }
            None => self.tag(0),
        }
    }

    fn merge_schema_kind_class(&mut self, value: MergeSchemaKindClass) {
        match value {
            MergeSchemaKindClass::Entity => self.tag(1),
            MergeSchemaKindClass::Relation => self.tag(2),
        }
    }
}
