use std::fmt::{self, Write};
use worth_foundational::facade::{AspectKey, CanonicalDigestId, FieldKey};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadRelationAuthority {
    schema_basis_digest: CanonicalDigestId,
    relation_name: String,
}

impl WorthQueryGraphReadRelationAuthority {
    pub fn new(schema_basis_digest: CanonicalDigestId, relation_name: impl Into<String>) -> Self {
        Self {
            schema_basis_digest,
            relation_name: relation_name.into(),
        }
    }

    pub const fn schema_basis_digest(&self) -> &CanonicalDigestId {
        &self.schema_basis_digest
    }

    pub fn relation_name(&self) -> &str {
        &self.relation_name
    }

    pub fn digest_part(&self) -> String {
        let mut text = String::new();
        self.write_digest_part(&mut text)
            .expect("String formatting cannot fail");
        text
    }

    pub(crate) fn write_digest_part(&self, output: &mut dyn Write) -> fmt::Result {
        write!(output, "relation_authority:{}", self.relation_name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadPredicateFieldAuthority {
    schema_basis_digest: CanonicalDigestId,
    aspect: AspectKey,
    field: FieldKey,
    field_kind: String,
}

impl WorthQueryGraphReadPredicateFieldAuthority {
    pub fn new(
        schema_basis_digest: CanonicalDigestId,
        aspect: AspectKey,
        field: FieldKey,
        field_kind: impl Into<String>,
    ) -> Self {
        Self {
            schema_basis_digest,
            aspect,
            field,
            field_kind: field_kind.into(),
        }
    }

    pub const fn schema_basis_digest(&self) -> &CanonicalDigestId {
        &self.schema_basis_digest
    }

    pub fn native_aspect_key(&self) -> &AspectKey {
        &self.aspect
    }

    pub fn native_field_key(&self) -> &FieldKey {
        &self.field
    }

    pub fn field_kind(&self) -> &str {
        &self.field_kind
    }

    pub fn digest_part(&self) -> String {
        let mut text = String::new();
        self.write_digest_part(&mut text)
            .expect("String formatting cannot fail");
        text
    }

    pub(crate) fn write_digest_part(&self, output: &mut dyn Write) -> fmt::Result {
        write!(
            output,
            "predicate_authority:{}:{}:{}",
            self.aspect.as_str(),
            self.field.as_str(),
            self.field_kind
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadOrderingFieldAuthority {
    schema_basis_digest: CanonicalDigestId,
    collection_path: String,
    aspect: AspectKey,
    field: FieldKey,
    direction: String,
    field_kind: String,
}

impl WorthQueryGraphReadOrderingFieldAuthority {
    pub fn new(
        schema_basis_digest: CanonicalDigestId,
        collection_path: impl Into<String>,
        aspect: AspectKey,
        field: FieldKey,
        direction: impl Into<String>,
        field_kind: impl Into<String>,
    ) -> Self {
        Self {
            schema_basis_digest,
            collection_path: collection_path.into(),
            aspect,
            field,
            direction: direction.into(),
            field_kind: field_kind.into(),
        }
    }

    pub const fn schema_basis_digest(&self) -> &CanonicalDigestId {
        &self.schema_basis_digest
    }

    pub fn collection_path(&self) -> &str {
        &self.collection_path
    }

    pub fn native_aspect_key(&self) -> &AspectKey {
        &self.aspect
    }

    pub fn native_field_key(&self) -> &FieldKey {
        &self.field
    }

    pub fn direction(&self) -> &str {
        &self.direction
    }

    pub fn field_kind(&self) -> &str {
        &self.field_kind
    }

    pub fn digest_part(&self) -> String {
        let mut text = String::new();
        self.write_digest_part(&mut text)
            .expect("String formatting cannot fail");
        text
    }

    pub(crate) fn write_digest_part(&self, output: &mut dyn Write) -> fmt::Result {
        write!(
            output,
            "ordering_authority:{}:{}:{}:{}:{}",
            self.collection_path,
            self.aspect.as_str(),
            self.field.as_str(),
            self.direction,
            self.field_kind
        )
    }
}
