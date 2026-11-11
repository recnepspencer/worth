//! Bounded copies of portable recipe rosters, without creating registered owners.
use crate::{
    derived_relation::{
        DerivedRelationDefinitionParts, DerivedRelationGraphLaws,
        DerivedRelationMinimumReachability,
    },
    execution::RelationalExecutionDenialCause as Denial,
    projected_member_domain::ProjectedMemberDomainParts,
    schema::SchemaCompilationControl,
};

pub(super) fn definition(
    source: &DerivedRelationDefinitionParts,
    control: SchemaCompilationControl<'_, '_>,
) -> Result<DerivedRelationDefinitionParts, Denial> {
    let mut nodes = Vec::with_capacity(source.nodes.len());
    for node in &source.nodes {
        control.checkpoint()?;
        nodes.push(node.clone());
    }
    let laws = match &source.laws {
        Some(source) => Some(laws(source, control)?),
        None => None,
    };
    control.checkpoint()?;
    Ok(DerivedRelationDefinitionParts {
        identity: source.identity.clone(),
        from: source.from.clone(),
        to: source.to.clone(),
        nodes,
        laws,
    })
}
fn laws(
    source: &DerivedRelationGraphLaws,
    control: SchemaCompilationControl<'_, '_>,
) -> Result<DerivedRelationGraphLaws, Denial> {
    let reaches_at_least = match &source.reaches_at_least {
        Some(minimum) => {
            let mut target_domains = Vec::with_capacity(minimum.target_domains.len());
            for domain in &minimum.target_domains {
                control.checkpoint()?;
                target_domains.push(domain.clone());
            }
            Some(DerivedRelationMinimumReachability {
                minimum_reachable_targets: minimum.minimum_reachable_targets,
                target_domains,
            })
        }
        None => None,
    };
    Ok(DerivedRelationGraphLaws {
        self_edges_allowed: source.self_edges_allowed,
        cross_context_policy: source.cross_context_policy,
        cardinality: source.cardinality,
        acyclic: source.acyclic,
        symmetric: source.symmetric,
        orbit: source.orbit,
        same_partition_endpoints: source.same_partition_endpoints,
        reaches_at_least,
    })
}
pub(super) fn domain(
    source: &ProjectedMemberDomainParts,
    control: SchemaCompilationControl<'_, '_>,
) -> Result<ProjectedMemberDomainParts, Denial> {
    let mut cases = Vec::with_capacity(source.cases.len());
    for case in &source.cases {
        control.checkpoint()?;
        cases.push(case.clone());
    }
    Ok(ProjectedMemberDomainParts {
        identity: source.identity.clone(),
        carrier: source.carrier,
        cases,
    })
}
