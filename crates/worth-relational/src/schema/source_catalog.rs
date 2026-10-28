//! Cold dependency admission shared by registry composition and durable recovery.
use crate::derived_relation::definition::RegisteredDerivedRelation;
use crate::derived_relation::{
    DerivedRelationDefinitionParts as Parts, DerivedRelationDenial, DerivedRelationIdentity,
    DerivedRelationNode,
};
use crate::identity::data::KindId;
use crate::schema::data::{
    RelationKindRegistration, RelationalSchemaRegistry, SchemaRegistryError,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub(crate) enum DerivedCatalogEntry {
    Meaning(Parts),
    Retained(Arc<RegisteredDerivedRelation>),
}
impl DerivedCatalogEntry {
    fn parts(&self) -> &Parts {
        match self {
            Self::Meaning(parts) => parts,
            Self::Retained(entry) => &entry.parts,
        }
    }
    fn admit(
        self,
        registry: RelationalSchemaRegistry,
    ) -> Result<RelationalSchemaRegistry, SourceCatalogDenial> {
        match self {
            Self::Meaning(parts) => registry
                .register_derived_relations(vec![parts])
                .map_err(SourceCatalogDenial::Derived),
            Self::Retained(entry) => {
                // Validate the exact merged physical/domain/named sources before
                // preserving the actual issuer held by installed typed bindings.
                let mut registry = registry
                    .register_derived_relations(vec![entry.parts.clone()])
                    .map_err(SourceCatalogDenial::Derived)?;
                if registry.derived_relations[&entry.parts.identity].as_ref() != entry.as_ref() {
                    return Err(SourceCatalogDenial::CompiledMeaningMismatch(
                        entry.parts.identity.clone(),
                    ));
                }
                registry
                    .derived_relations
                    .insert(entry.parts.identity.clone(), entry);
                Ok(registry)
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum SourceCatalogNode {
    Membership(KindId),
    Derived(DerivedRelationIdentity),
}
#[derive(Debug)]
pub(crate) enum SourceCatalogDenial {
    Schema(SchemaRegistryError),
    Derived(DerivedRelationDenial),
    MissingPhysicalSource(KindId),
    MissingDerivedSource(DerivedRelationIdentity),
    DependencyCycle(Vec<SourceCatalogNode>),
    CompiledMeaningMismatch(DerivedRelationIdentity),
}
enum Entry {
    Membership(RelationKindRegistration),
    Derived(DerivedCatalogEntry),
}

/// Domains/carriers are already installed. Ordinary physical relations have no
/// recipe dependencies; memberships and recipes form one finite admitted DAG.
/// Missing dependencies refuse explicitly, never by retrying a failed validator.
pub(crate) fn install(
    mut registry: RelationalSchemaRegistry,
    relations: impl IntoIterator<Item = RelationKindRegistration>,
    derived: impl IntoIterator<Item = DerivedCatalogEntry>,
) -> Result<RelationalSchemaRegistry, SourceCatalogDenial> {
    let mut pending = BTreeMap::new();
    for registration in relations {
        if registration.relation_integrity.aggregate_identity.is_none() {
            registry = registry
                .register_relation_kind(registration)
                .map_err(SourceCatalogDenial::Schema)?;
        } else {
            let kind = registration.kind_id;
            if registry.relation_kinds.contains_key(&kind)
                || pending
                    .insert(
                        SourceCatalogNode::Membership(kind),
                        Entry::Membership(registration),
                    )
                    .is_some()
            {
                return Err(SourceCatalogDenial::Schema(
                    SchemaRegistryError::duplicate_relation_kind(kind),
                ));
            }
        }
    }
    for entry in derived {
        let identity = entry.parts().identity.clone();
        if registry.derived_relations.contains_key(&identity)
            || pending
                .insert(
                    SourceCatalogNode::Derived(identity.clone()),
                    Entry::Derived(entry),
                )
                .is_some()
        {
            return Err(SourceCatalogDenial::Derived(
                DerivedRelationDenial::DuplicateDefinition(identity),
            ));
        }
    }
    let mut remaining = BTreeMap::new();
    let mut users: BTreeMap<SourceCatalogNode, BTreeSet<SourceCatalogNode>> = BTreeMap::new();
    let mut ready = BTreeSet::new();
    for (node, entry) in &pending {
        let dependencies = dependencies(entry, &registry, &pending)?;
        if dependencies.is_empty() {
            ready.insert(node.clone());
        }
        remaining.insert(node.clone(), dependencies.len());
        for dependency in dependencies {
            users.entry(dependency).or_default().insert(node.clone());
        }
    }
    while let Some(node) = ready.pop_first() {
        let entry = pending.remove(&node).expect("ready catalog entry");
        registry = match entry {
            Entry::Membership(registration) => registry
                .register_relation_kind(registration)
                .map_err(SourceCatalogDenial::Schema)?,
            Entry::Derived(entry) => entry.admit(registry)?,
        };
        for user in users.remove(&node).unwrap_or_default() {
            let count = remaining.get_mut(&user).expect("pending dependent");
            *count -= 1;
            if *count == 0 {
                ready.insert(user);
            }
        }
    }
    if !pending.is_empty() {
        return Err(SourceCatalogDenial::DependencyCycle(
            pending.into_keys().collect(),
        ));
    }
    Ok(registry)
}

fn dependencies(
    entry: &Entry,
    registry: &RelationalSchemaRegistry,
    pending: &BTreeMap<SourceCatalogNode, Entry>,
) -> Result<BTreeSet<SourceCatalogNode>, SourceCatalogDenial> {
    let mut dependencies = BTreeSet::new();
    let mut physical = |kind| -> Result<(), SourceCatalogDenial> {
        if registry.relation_kinds.contains_key(&kind) {
            return Ok(());
        }
        let node = SourceCatalogNode::Membership(kind);
        if !pending.contains_key(&node) {
            return Err(SourceCatalogDenial::MissingPhysicalSource(kind));
        }
        dependencies.insert(node);
        Ok(())
    };
    match entry {
        Entry::Membership(registration) => {
            let contract = registration
                .relation_integrity
                .aggregate_identity
                .as_ref()
                .expect("catalog membership");
            if let Some(kinds) = contract.source.stored_relations() {
                for &kind in kinds {
                    physical(kind)?;
                }
            }
        }
        Entry::Derived(entry) => {
            for node in &entry.parts().nodes {
                if let DerivedRelationNode::Stored { relation, .. } = node {
                    physical(*relation)?;
                }
            }
        }
    }
    let mut named = |identity: &DerivedRelationIdentity| -> Result<(), SourceCatalogDenial> {
        if registry.derived_relations.contains_key(identity) {
            return Ok(());
        }
        let node = SourceCatalogNode::Derived(identity.clone());
        if !pending.contains_key(&node) {
            return Err(SourceCatalogDenial::MissingDerivedSource(identity.clone()));
        }
        dependencies.insert(node);
        Ok(())
    };
    match entry {
        Entry::Membership(registration) => {
            if let Some(identity) = registration
                .relation_integrity
                .aggregate_identity
                .as_ref()
                .expect("catalog membership")
                .source
                .derived_relation()
            {
                named(identity)?;
            }
        }
        Entry::Derived(entry) => {
            for node in &entry.parts().nodes {
                if let DerivedRelationNode::Derived(identity) = node {
                    named(identity)?;
                }
            }
        }
    }
    Ok(dependencies)
}

impl std::fmt::Display for SourceCatalogDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Schema(cause) => write!(formatter, "schema admission: {cause:?}"),
            Self::Derived(cause) => write!(formatter, "derived source admission: {cause:?}"),
            Self::MissingPhysicalSource(kind) => {
                write!(formatter, "missing physical source {kind:?}")
            }
            Self::MissingDerivedSource(identity) => {
                write!(formatter, "missing derived source {identity:?}")
            }
            Self::DependencyCycle(nodes) => write!(formatter, "source dependency cycle {nodes:?}"),
            Self::CompiledMeaningMismatch(identity) => {
                write!(formatter, "compiled source meaning mismatch {identity:?}")
            }
        }
    }
}
