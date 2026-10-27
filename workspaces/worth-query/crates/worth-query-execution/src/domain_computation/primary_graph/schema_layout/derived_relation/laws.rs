//! Preserve the old proved subset; all other laws use real Native enforcement.
use super::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationGraphMemberMeaning as Endpoint, ApplicationRelationGraphLaws,
    WorthQueryPortableDerivedRelationParts,
};
use worth_relational::facade::schema::{
    DerivedRelationCardinality, DerivedRelationGraphLaws, DerivedRelationMinimumReachability,
};

pub(super) fn lower(
    parts: &WorthQueryPortableDerivedRelationParts,
    schema: &ErasedApplicationSchemaDeclaration,
    entities: &BTreeMap<String, KindId>,
    relations: &BTreeMap<String, KindId>,
    domains: &BTreeMap<String, super::super::projected_member_domain::ProjectedMemberDomainLayout>,
    registry: &RelationalSchemaRegistry,
) -> Result<Option<DerivedRelationGraphLaws>, Denial> {
    if legacy_proved(parts, schema, registry, relations) {
        return Ok(None);
    }
    let laws = &parts.laws;
    let cardinality = laws.cardinality;
    let reaches_at_least = laws
        .reaches_at_least
        .as_ref()
        .map(|minimum| {
            let mut target_domains = minimum
                .target_kinds
                .iter()
                .map(|name| {
                    let member = if entities.contains_key(name) {
                        Endpoint::Entity(name.clone())
                    } else {
                        Endpoint::Projected(name.clone())
                    };
                    super::super::projected_member_domain::member_domain(&member, entities, domains)
                })
                .collect::<Result<Vec<_>, _>>()?;
            target_domains.sort();
            target_domains.dedup();
            Ok(DerivedRelationMinimumReachability {
                minimum_reachable_targets: minimum.minimum_reachable_targets,
                target_domains,
            })
        })
        .transpose()?;
    Ok(Some(DerivedRelationGraphLaws {
        self_edges_allowed: laws.endpoints.self_edges_allowed,
        cross_context_policy: super::super::registry_lowering::lower_cross_context(
            laws.endpoints.cross_context_policy,
        ),
        cardinality: DerivedRelationCardinality {
            source_min: cardinality.source_min,
            source_max: cardinality.source_max,
            target_min: cardinality.target_min,
            target_max: cardinality.target_max,
            pair_min: cardinality.pair_min,
            pair_max: cardinality.pair_max,
        },
        acyclic: laws.acyclic,
        symmetric: laws.symmetric,
        orbit: laws.orbit,
        same_partition_endpoints: laws.same_partition_endpoints,
        reaches_at_least,
    }))
}

fn legacy_proved(
    parts: &WorthQueryPortableDerivedRelationParts,
    schema: &ErasedApplicationSchemaDeclaration,
    registry: &RelationalSchemaRegistry,
    relations: &BTreeMap<String, KindId>,
) -> bool {
    if parts.laws != ApplicationRelationGraphLaws::unbounded()
        || !parts.expression.is_legacy_shape()
        || parts.from.entity().is_none()
        || parts.to.entity().is_none()
    {
        return false;
    }
    application_derived_relation_source_closure(schema.members(), &parts.relation).is_ok_and(
        |closure| {
            closure.physical.iter().all(|source| {
                relations
                    .get(&source.relation)
                    .and_then(|kind| registry.relation_registration(*kind).ok())
                    .is_some_and(|registration| {
                        registration.cross_context_policy
                            == worth_relational::facade::config::CrossContextPolicy::Forbid
                    })
            })
        },
    )
}
