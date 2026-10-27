//! Stable semantic definition identities, independent of native kind allocation.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationComponentSetMeaning;

pub(super) struct Definition<'a> {
    pub member: &'a str,
    pub identity: AggregateDefinitionIdentity,
    pub source: AggregateIdentitySource,
}

pub(super) fn collect<'a>(
    schema: &'a ErasedApplicationSchemaDeclaration,
    relation_kinds: &BTreeMap<String, KindId>,
) -> Result<Vec<Definition<'a>>, WorthQueryPrimaryGraphInstallationDenial> {
    let mut orbits = BTreeMap::new();
    let mut components = BTreeMap::new();
    for member in schema.members() {
        match member {
            ApplicationSchemaMember::Relation {
                relation,
                from,
                integrity,
                ..
            } if integrity.orbit => {
                orbits.insert(relation.as_str(), from.as_str());
            }
            ApplicationSchemaMember::ComponentSet { definition } => {
                components.insert(definition.name(), definition);
            }
            _ => {}
        }
    }
    let mut definitions = Vec::new();
    // Keep the accepted Orbit registration order and every Orbit identity byte.
    for (relation, member) in orbits {
        definitions.push(Definition {
            member,
            identity: orbit_identity(schema, relation),
            source: AggregateIdentitySource::Orbit(required_kind(relation_kinds, relation)?),
        });
    }
    for meaning in components.into_values() {
        let relations = meaning
            .relations()
            .iter()
            .map(|relation| required_kind(relation_kinds, relation))
            .collect::<Result<_, _>>()?;
        definitions.push(Definition {
            member: meaning.member(),
            identity: component_identity(schema, meaning),
            source: AggregateIdentitySource::Components { relations },
        });
    }
    Ok(definitions)
}

fn orbit_identity(
    schema: &ErasedApplicationSchemaDeclaration,
    relation: &str,
) -> AggregateDefinitionIdentity {
    let mut hash = Sha256::new();
    hash.update(b"WORTH_QUERY_ORBIT_DEFINITION_V1\0");
    for name in [schema.owner(), schema.name(), relation] {
        text(&mut hash, name);
    }
    hash.update(schema.major().to_be_bytes());
    AggregateDefinitionIdentity::from_bytes(hash.finalize().into())
}

fn component_identity(
    schema: &ErasedApplicationSchemaDeclaration,
    meaning: &ApplicationComponentSetMeaning,
) -> AggregateDefinitionIdentity {
    let mut hash = Sha256::new();
    hash.update(b"WORTH_QUERY_COMPONENT_DEFINITION_V1\0");
    for name in [
        schema.owner(),
        schema.name(),
        meaning.name(),
        meaning.member(),
    ] {
        text(&mut hash, name);
    }
    hash.update(schema.major().to_be_bytes());
    hash.update((meaning.relations().len() as u64).to_be_bytes());
    for relation in meaning.relations() {
        text(&mut hash, relation);
    }
    AggregateDefinitionIdentity::from_bytes(hash.finalize().into())
}

fn text(hash: &mut Sha256, value: &str) {
    hash.update((value.len() as u64).to_be_bytes());
    hash.update(value.as_bytes());
}
