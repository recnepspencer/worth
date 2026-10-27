//! One installed recipe joins descriptive schema meaning to a Native handle.
use super::{invalid_member, required_kind, WorthQueryPrimaryGraphInstallationDenial as Denial};
use std::collections::BTreeMap;
use std::sync::Arc;
use worth_query_declaration::facade::application_schema::{
    application_derived_relation_source_closure, ApplicationGraphMemberMeaning,
};
use worth_query_installation::facade::{
    ApplicationSchemaMember as Member, ErasedApplicationSchemaDeclaration,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::schema::{
    DerivedRelationDefinitionParts, DerivedRelationHandle, DerivedRelationIdentity,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};

mod expression;
mod laws;

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct DerivedRelationLayout {
    pub(in crate::domain_computation::primary_graph) orbit_definition:
        Option<worth_foundational::facade::AggregateDefinitionIdentity>,
    pub(in crate::domain_computation::primary_graph) from: ApplicationGraphMemberMeaning,
    pub(in crate::domain_computation::primary_graph) to: ApplicationGraphMemberMeaning,
    pub(in crate::domain_computation::primary_graph) handle: DerivedRelationHandle,
    pub(in crate::domain_computation::primary_graph) sources:
        Arc<[(Arc<str>, KindId, KindId, KindId)]>,
    pub(in crate::domain_computation::primary_graph) operations: Arc<[Arc<str>]>,
    pub(in crate::domain_computation::primary_graph) carriers: Arc<[(Arc<str>, KindId)]>,
}

pub(super) fn lower(
    registry: RelationalSchemaRegistry,
    schema: &ErasedApplicationSchemaDeclaration,
    basis: (&SchemaId, SchemaVersionId),
    entities: &BTreeMap<String, KindId>,
    relations: &BTreeMap<String, KindId>,
    domains: &BTreeMap<String, super::projected_member_domain::ProjectedMemberDomainLayout>,
) -> Result<
    (
        RelationalSchemaRegistry,
        BTreeMap<String, DerivedRelationLayout>,
    ),
    Denial,
> {
    let identity = |name: &str| DerivedRelationIdentity {
        schema: basis.0.clone(),
        version: basis.1,
        name: name.to_owned(),
    };
    let mut definitions = Vec::new();
    for member in schema.members() {
        let Member::DerivedRelation { definition } = member else {
            continue;
        };
        let parts = definition.parts();
        definitions.push(DerivedRelationDefinitionParts {
            identity: identity(&parts.relation),
            from: super::projected_member_domain::member_domain(&parts.from, entities, domains)?,
            to: super::projected_member_domain::member_domain(&parts.to, entities, domains)?,
            nodes: expression::lower(parts, &identity, entities, relations, domains)?,
            laws: laws::lower(parts, schema, entities, relations, domains, &registry)?,
        });
    }
    let registry = registry
        .register_derived_relations(definitions)
        .map_err(|cause| invalid_member(&format!("derived Native registration: {cause:?}")))?;
    let mut layouts = BTreeMap::new();
    for member in schema.members() {
        let Member::DerivedRelation { definition } = member else {
            continue;
        };
        let parts = definition.parts();
        let closure =
            application_derived_relation_source_closure(schema.members(), &parts.relation)
                .map_err(|_| invalid_member(&parts.relation))?;
        let carriers = closure
            .projected_domains
            .iter()
            .map(|domain| {
                Ok((
                    Arc::from(domain.carrier.as_str()),
                    required_kind(entities, &domain.carrier)?,
                ))
            })
            .collect::<Result<Arc<[_]>, Denial>>()?;
        let sources = closure
            .physical
            .into_iter()
            .map(|source| {
                Ok((
                    Arc::from(source.relation.as_str()),
                    required_kind(relations, &source.relation)?,
                    required_kind(
                        entities,
                        source
                            .from
                            .entity()
                            .ok_or_else(|| invalid_member(&parts.relation))?,
                    )?,
                    required_kind(
                        entities,
                        source
                            .to
                            .entity()
                            .ok_or_else(|| invalid_member(&parts.relation))?,
                    )?,
                ))
            })
            .collect::<Result<Arc<[_]>, Denial>>()?;
        layouts.insert(
            parts.relation.clone(),
            DerivedRelationLayout {
                orbit_definition: parts.laws.orbit.then(|| {
                    super::aggregate_identity::derived_orbit_identity(
                        schema,
                        &parts.relation,
                        parts.from.borrowed(),
                    )
                }),
                from: parts.from.clone(),
                to: parts.to.clone(),
                carriers,
                sources,
                operations: schema
                    .members()
                    .iter()
                    .filter_map(|member| match member {
                        Member::OperationDerivedRead {
                            operation,
                            relation,
                        } if relation == &parts.relation => Some(Arc::from(operation.as_str())),
                        _ => None,
                    })
                    .collect(),
                handle: registry
                    .derived_relation(&identity(&parts.relation))
                    .expect("registered recipe"),
            },
        );
    }
    Ok((registry, layouts))
}

impl super::WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn derived_relation(
        &self,
        name: &str,
    ) -> Option<&DerivedRelationLayout> {
        self.derived_relations.get(name)
    }
}
