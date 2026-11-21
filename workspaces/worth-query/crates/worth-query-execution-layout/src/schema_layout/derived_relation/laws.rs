//! Discharge proved endpoint laws; stronger laws retain real Native enforcement.
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
    if (parts.expression.is_legacy_shape() || maintained_ordered(parts, schema))
        && endpoint_laws_proved(parts, schema, registry, relations)
    {
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
            Ok::<_, Denial>(DerivedRelationMinimumReachability {
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

fn endpoint_laws_proved(
    parts: &WorthQueryPortableDerivedRelationParts,
    schema: &ErasedApplicationSchemaDeclaration,
    registry: &RelationalSchemaRegistry,
    relations: &BTreeMap<String, KindId>,
) -> bool {
    if parts.laws != ApplicationRelationGraphLaws::unbounded()
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

/// A maintained ordered Source owns event-chain definedness on every affected
/// scope. Physical same-context leaves prove the default endpoint laws; doing
/// a second complete ordered traversal here adds no evidence. No stronger law
/// is discharged, and installation still binds the exact Native Source recipe.
fn maintained_ordered(
    parts: &WorthQueryPortableDerivedRelationParts,
    schema: &ErasedApplicationSchemaDeclaration,
) -> bool {
    use worth_foundational::facade::RelationDirection;
    use worth_query_declaration::facade::{
        application_query::{
            ApplicationManagedIndexSourceScope, ApplicationQueryRootPathSourceKind,
            ApplicationQueryRootPathTraversal,
        },
        application_schema::ApplicationDerivedRelationNode,
    };
    if !matches!(
        parts.expression.nodes.last(),
        Some(ApplicationDerivedRelationNode::OrderedEvents { .. })
    ) {
        return false;
    }
    let required = super::super::managed_source::requirements(schema);
    schema.members().iter().any(|member| {
        let Member::ManagedIndex { definition } = member else {
            return false;
        };
        let index = definition.parts();
        if index.source.scope != ApplicationManagedIndexSourceScope::OrderedPredecessor
            || !required.contains_key(index.name.as_str())
        {
            return false;
        }
        schema.members().iter().any(|member| {
            let Member::ApplicationQuery { definition } = member else {
                return false;
            };
            if definition.name() != index.source.binding.parts().query_name {
                return false;
            }
            let [path] = definition.root_paths() else {
                return false;
            };
            let [step] = path.steps() else { return false };
            path.guards().is_empty()
                && definition.predicates().is_empty()
                && step.source_kind() == ApplicationQueryRootPathSourceKind::Derived
                && step.direction() == RelationDirection::Forward
                && step.traversal() == ApplicationQueryRootPathTraversal::OneHop
                && step.relation() == parts.relation
        })
    })
}
