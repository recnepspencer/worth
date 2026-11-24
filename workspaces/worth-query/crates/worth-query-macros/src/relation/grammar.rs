//! Attribute grammar for `#[derive(WorthQueryRelation)]`.
use super::DERIVE;
use crate::grammar::{unknown_key, value, visit_keys, Once};
use crate::schema::grammar::{
    parse_reference, relation_reference, Reference, RelationLawKeys, RelationLaws,
};
use syn::meta::ParseNestedMeta;
use syn::{DeriveInput, LitStr, Path, Type};

const KEYS: &[&str] = &[
    "name",
    "to",
    "from",
    "reference",
    "cardinality",
    "acyclic",
    "self_edges",
    "unique",
    "deletion",
    "cross_context",
    "symmetric",
    "orbit",
    "same_partition_endpoints",
    "reaches_at_least",
];

pub(super) struct RelationDeclaration {
    pub(super) name: Option<LitStr>,
    pub(super) to: Type,
    pub(super) reference: Option<(Path, Reference)>,
    pub(super) laws: RelationLaws,
}

struct RelationKeys {
    name: Once<LitStr>,
    to: Once<Type>,
    from: Once<Path>,
    reference: Once<Reference>,
    laws: RelationLawKeys,
}

impl RelationDeclaration {
    pub(super) fn parse(input: &DeriveInput) -> syn::Result<Self> {
        let mut keys = RelationKeys::new();
        visit_keys(&input.attrs, |key, meta| keys.read(key, meta))?;
        let missing = input.ident.span();
        let to = keys.to.required(missing, DERIVE)?;
        let reference = relation_reference(keys.from, keys.reference, missing)?;
        Ok(Self {
            name: keys.name.optional(),
            to,
            reference,
            laws: keys.laws.finish(),
        })
    }
}

impl RelationKeys {
    fn new() -> Self {
        Self {
            name: Once::new("name"),
            to: Once::new("to"),
            from: Once::new("from"),
            reference: Once::new("reference"),
            laws: RelationLawKeys::new(),
        }
    }

    fn read(&mut self, key: &str, meta: &ParseNestedMeta<'_>) -> syn::Result<()> {
        if self.laws.try_read(key, meta)? {
            return Ok(());
        }
        match key {
            "name" => self.name.set(meta, value(meta)?),
            "to" => self.to.set(meta, value(meta)?),
            "from" => self.from.set(meta, value(meta)?),
            "reference" => self.reference.set(meta, parse_reference(meta)?),
            _ => Err(unknown_key(meta, DERIVE, KEYS)),
        }
    }
}
