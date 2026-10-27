//! New named source associations append behind every accepted physical definition.
use super::*;
use worth_relational::facade::schema::DerivedRelationIdentity;

pub(super) fn append<'a>(
    definitions: &mut Vec<Definition<'a>>,
    schema: &'a ErasedApplicationSchemaDeclaration,
    registry: &RelationalSchemaRegistry,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let basis = super::super::super::registry_lowering::relational_schema_basis(schema, registry)?;
    let source = |name: &str| {
        registry
            .derived_relation(&DerivedRelationIdentity {
                schema: basis.0.clone(),
                version: basis.1,
                name: name.to_owned(),
            })
            .ok_or_else(|| super::super::super::invalid_member(name))
    };
    let mut orbits = BTreeMap::new();
    let mut components = BTreeMap::new();
    for member in schema.members() {
        match member {
            ApplicationSchemaMember::DerivedRelation { definition }
                if definition.parts().laws.orbit =>
            {
                orbits.insert(definition.parts().relation.as_str(), definition.parts());
            }
            ApplicationSchemaMember::DerivedComponentSet { definition } => {
                components.insert(definition.name.as_str(), definition);
            }
            _ => {}
        }
    }
    for (name, parts) in orbits {
        let handle = source(name)?;
        definitions.push(Definition {
            component: None,
            member: parts.from.borrowed(),
            identity: orbit_identity(schema, name, parts.from.borrowed()),
            source: AggregateIdentitySource::DerivedOrbit {
                derived_orbit: handle.parts().identity.clone(),
            },
            native_source: Some(handle),
        });
    }
    for meaning in components.into_values() {
        let handle = source(&meaning.relation)?;
        definitions.push(Definition {
            component: Some(ComponentMeaning::Derived(meaning)),
            member: meaning.member.borrowed(),
            identity: identity(
                schema,
                b"WORTH_QUERY_DERIVED_COMPONENT_DEFINITION_V1\0",
                &meaning.name,
                meaning.member.borrowed(),
                &meaning.relation,
            ),
            source: AggregateIdentitySource::DerivedComponents {
                derived_components: handle.parts().identity.clone(),
            },
            native_source: Some(handle),
        });
    }
    Ok(())
}

pub(in crate::domain_computation::primary_graph::schema_layout) fn orbit_identity(
    schema: &ErasedApplicationSchemaDeclaration,
    name: &str,
    member: Member<'_>,
) -> AggregateDefinitionIdentity {
    identity(
        schema,
        b"WORTH_QUERY_DERIVED_ORBIT_DEFINITION_V1\0",
        name,
        member,
        name,
    )
}

fn identity(
    schema: &ErasedApplicationSchemaDeclaration,
    prefix: &[u8],
    name: &str,
    member: Member<'_>,
    relation: &str,
) -> AggregateDefinitionIdentity {
    let mut hash = Sha256::new();
    hash.update(prefix);
    for text_value in [schema.owner(), schema.name(), name, member.name(), relation] {
        text(&mut hash, text_value);
    }
    hash.update([match member {
        Member::Entity(_) => 1,
        Member::Projected(_) => 2,
    }]);
    hash.update(schema.major().to_be_bytes());
    AggregateDefinitionIdentity::from_bytes(hash.finalize().into())
}
