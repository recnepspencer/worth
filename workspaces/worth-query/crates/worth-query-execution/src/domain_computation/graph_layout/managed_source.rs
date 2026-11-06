//! Compile the actual installed Source into Native's existing path owner.
use std::collections::BTreeMap;
use {
    super::required_kind, super::WorthQueryPrimaryFieldLayout,
    crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenial as Denial,
};

mod membership;
mod projection;
use worth_foundational::facade::RelationDirection;
use worth_query_declaration::facade::application_query::{
    ApplicationManagedIndexColdSource as Cold, ApplicationQueryRootPathTraversal,
    ErasedApplicationManagedIndexDefinition, ErasedApplicationQueryDefinition,
};
use worth_query_installation::facade::{
    ApplicationSchemaMember as Member, ErasedApplicationSchemaDeclaration,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::runtime::{ManagedIndexSourceField, ManagedIndexSourceProgram};
use worth_relational::facade::schema::{
    DerivedRelationDefinitionParts, DerivedRelationIdentity, DerivedRelationNode as Node,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};

/// The sole lowered Source requirement consumed by rule resolution and setup.
#[derive(Clone, Debug)]
pub(in crate::domain_computation) struct ManagedSource {
    pub(in crate::domain_computation) program: ManagedIndexSourceProgram,
    pub(in crate::domain_computation) definition: ErasedApplicationManagedIndexDefinition,
    pub(in crate::domain_computation) cold: Cold,
}

fn requirements(schema: &ErasedApplicationSchemaDeclaration) -> BTreeMap<&str, Cold> {
    let mut required = BTreeMap::new();
    for member in schema.members() {
        let requirement = match member {
            Member::ManagedIndex { definition } => definition
                .maintenance()
                .native_cold()
                .map(|cold| (definition.parts().name.as_str(), cold)),
            Member::InvariantManagedIndexSource { requirement, .. } => {
                Some((requirement.index(), requirement.cold()))
            }
            _ => None,
        };
        if let Some((index, cold)) = requirement {
            let effective = required.entry(index).or_insert(cold);
            if cold == Cold::Refuse {
                *effective = Cold::Refuse;
            }
        }
    }
    required
}

pub(super) fn lower(
    mut registry: RelationalSchemaRegistry,
    schema: &ErasedApplicationSchemaDeclaration,
    basis: (&SchemaId, SchemaVersionId),
    entities: &BTreeMap<String, KindId>,
    relations: &BTreeMap<String, KindId>,
    fields: &BTreeMap<(String, String, String), WorthQueryPrimaryFieldLayout>,
) -> Result<(RelationalSchemaRegistry, BTreeMap<String, ManagedSource>), Denial> {
    let required = requirements(schema);
    let mut programs = BTreeMap::new();
    for member in schema.members() {
        let Member::ManagedIndex { definition } = member else {
            continue;
        };
        let Some(&cold) = required.get(definition.parts().name.as_str()) else {
            continue;
        };
        let source_name = &definition.parts().source.binding.parts().query_name;
        let source = schema
            .members()
            .iter()
            .find_map(|member| match member {
                Member::ApplicationQuery { definition } if definition.name() == source_name => {
                    Some(definition)
                }
                _ => None,
            })
            .ok_or_else(|| unsupported(source_name))?;
        let identity = DerivedRelationIdentity {
            schema: basis.0.clone(),
            version: basis.1,
            name: format!("managed-index-source::{}", definition.parts().name),
        };
        let nodes = membership::path(source, entities, relations, &registry, basis)?;
        let (next_registry, projection, routes) =
            projection::lower(registry, source, &identity, entities, relations, fields)?;
        registry = next_registry;
        registry = registry
            .register_derived_relations(vec![DerivedRelationDefinitionParts {
                identity: identity.clone(),
                from: required_kind(entities, source.scope_entity())?.into(),
                to: required_kind(entities, source.root_entity())?.into(),
                nodes,
                laws: None,
            }])
            .map_err(|_| unsupported(source_name))?;
        let handle = registry
            .derived_relation(&identity)
            .ok_or_else(|| unsupported(source_name))?;
        let program = registry
            .bind_managed_index_source(&handle, projection, routes)
            .map_err(|cause| source_binding_denial(cause, source_name))?;
        programs.insert(
            definition.parts().name.clone(),
            ManagedSource {
                program,
                definition: definition.clone(),
                cold,
            },
        );
    }
    Ok((registry, programs))
}

fn unsupported(name: &str) -> Denial {
    crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenial::new(
        crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenialKind::UnsupportedManagedSource,
        name,
    )
}

fn source_binding_denial(
    cause: worth_relational::facade::schema::DerivedRelationDenial,
    name: &str,
) -> Denial {
    if cause
        == worth_relational::facade::schema::DerivedRelationDenial::SourceProgramIdentityExhausted
    {
        Denial::new(
            crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenialKind::SourceProgramIdentityExhausted,
            name,
        )
    } else {
        unsupported(name)
    }
}

#[test]
fn source_program_identity_exhaustion_is_preserved_at_installation() {
    use crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenialKind as Kind;
    use worth_relational::facade::schema::DerivedRelationDenial as Native;
    for (cause, expected) in [
        (
            Native::SourceProgramIdentityExhausted,
            Kind::SourceProgramIdentityExhausted,
        ),
        (Native::ForeignDefinition, Kind::UnsupportedManagedSource),
    ] {
        assert_eq!(
            source_binding_denial(cause, "bounded-source").kind(),
            expected
        );
    }
}

impl super::WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation) fn required_managed_sources(
        &self,
    ) -> impl Iterator<Item = &ManagedSource> {
        self.managed_sources.values()
    }

    pub(in crate::domain_computation) fn managed_source(
        &self,
        index: &str,
    ) -> Option<&ManagedIndexSourceProgram> {
        self.managed_sources
            .get(index)
            .map(|source| &source.program)
    }
}
