//! One finite grammar validator for registration and recovered snapshots.
use std::collections::{BTreeMap, BTreeSet};

use super::{DerivedRelationDefinitionParts as Parts, DerivedRelationDenial as Denial};
use super::{DerivedRelationIdentity as Identity, DerivedRelationNode as Node};
use crate::graph_member::NativeMemberDomain as Domain;
use crate::identity::data::KindId;
use crate::schema::data::RelationalSchemaRegistry;

pub(super) struct ValidatedExpression {
    pub(super) dependencies: BTreeSet<Identity>,
    pub(super) endpoints: Vec<(Domain, Domain)>,
}

pub(super) fn expression(
    parts: &Parts,
    definitions: &BTreeMap<Identity, Parts>,
    registry: &RelationalSchemaRegistry,
) -> Result<ValidatedExpression, Denial> {
    if parts.identity.name.trim().is_empty()
        || parts.identity.name.trim() != parts.identity.name
        || parts.identity.schema.0.is_empty()
    {
        return Err(Denial::InvalidIdentity);
    }
    for domain in [&parts.from, &parts.to] {
        endpoint_domain(registry, &parts.identity, domain)?;
    }
    if let Some(laws) = &parts.laws {
        if !laws.valid(&parts.from, &parts.to) {
            return Err(Denial::InvalidGraphLaws);
        }
        if let Some(minimum) = &laws.reaches_at_least {
            for domain in &minimum.target_domains {
                endpoint_domain(registry, &parts.identity, domain)?;
            }
        }
    }
    let mut endpoints: Vec<(Domain, Domain)> = Vec::new();
    let mut dependencies = BTreeSet::new();
    for node in &parts.nodes {
        let prior = |index: usize| {
            endpoints
                .get(index)
                .cloned()
                .ok_or(Denial::InvalidExpression)
        };
        let pair = match node {
            Node::Stored { relation, from, to } => {
                entity_kind(registry, &parts.identity, *from)?;
                entity_kind(registry, &parts.identity, *to)?;
                let registered = registry
                    .relation_registration(*relation)
                    .map_err(|_| Denial::InvalidPhysicalSource(*relation))?;
                if registered.schema_id != parts.identity.schema
                    || registered.schema_version_id != parts.identity.version
                    || !registered
                        .relation_integrity
                        .endpoint_kind_contracts
                        .iter()
                        .any(|contract| {
                            contract.allowed_source_kinds.contains(from)
                                && contract.allowed_target_kinds.contains(to)
                        })
                {
                    return Err(Denial::InvalidPhysicalSource(*relation));
                }
                ((*from).into(), (*to).into())
            }
            Node::Derived(identity) => {
                let target = definitions
                    .get(identity)
                    .ok_or_else(|| Denial::MissingDefinition(identity.clone()))?;
                dependencies.insert(identity.clone());
                (target.from.clone(), target.to.clone())
            }
            Node::Compose { left, right } => {
                let left = prior(*left)?;
                let right = prior(*right)?;
                if left.1 != right.0 {
                    return Err(Denial::DiscontinuousEndpoints);
                }
                (left.0, right.1)
            }
            Node::Reverse { input } => {
                let pair = prior(*input)?;
                (pair.1, pair.0)
            }
            Node::WithoutSelfEdges { input } => {
                let pair = prior(*input)?;
                if pair.0 != pair.1 {
                    return Err(Denial::DiscontinuousEndpoints);
                }
                pair
            }
            Node::CarrierLift { domain, case } => {
                let registered = registry
                    .projected_member_domain(domain)
                    .ok_or(Denial::InvalidMemberDomain)?;
                if domain.schema != parts.identity.schema
                    || domain.version != parts.identity.version
                {
                    return Err(Denial::InvalidMemberDomain);
                }
                if registered.case(case).is_none() {
                    return Err(Denial::InvalidMemberCase);
                }
                (
                    Domain::Entity(registered.parts().carrier),
                    Domain::Projected {
                        projected: domain.clone(),
                    },
                )
            }
            Node::Union { left, right } | Node::Intersection { left, right } => {
                let pair = prior(*left)?;
                if pair != prior(*right)? {
                    return Err(Denial::DiscontinuousEndpoints);
                }
                pair
            }
        };
        endpoints.push(pair);
    }
    if endpoints.last() != Some(&(parts.from.clone(), parts.to.clone())) {
        return Err(Denial::InvalidExpression);
    }
    let mut visited = BTreeSet::new();
    let mut pending = vec![parts.nodes.len() - 1];
    while let Some(index) = pending.pop() {
        if !visited.insert(index) {
            continue;
        }
        match parts.nodes[index] {
            Node::Compose { left, right }
            | Node::Union { left, right }
            | Node::Intersection { left, right } => pending.extend([left, right]),
            Node::Reverse { input } | Node::WithoutSelfEdges { input } => pending.push(input),
            Node::Stored { .. } | Node::Derived(_) | Node::CarrierLift { .. } => {}
        }
    }
    if visited.len() != parts.nodes.len() {
        return Err(Denial::InvalidExpression);
    }
    Ok(ValidatedExpression {
        dependencies,
        endpoints,
    })
}

fn endpoint_domain(
    registry: &RelationalSchemaRegistry,
    identity: &Identity,
    domain: &Domain,
) -> Result<(), Denial> {
    match domain {
        Domain::Entity(kind) => entity_kind(registry, identity, *kind),
        Domain::Projected { projected } => {
            let registered = registry
                .projected_member_domain(projected)
                .ok_or(Denial::InvalidMemberDomain)?;
            if projected.schema != identity.schema || projected.version != identity.version {
                return Err(Denial::InvalidMemberDomain);
            }
            entity_kind(registry, identity, registered.parts().carrier)
        }
    }
}

fn entity_kind(
    registry: &RelationalSchemaRegistry,
    identity: &Identity,
    kind: KindId,
) -> Result<(), Denial> {
    let registration = registry
        .entity_registration(kind)
        .map_err(|_| Denial::InvalidPhysicalSource(kind))?;
    if registration.schema_id != identity.schema
        || registration.schema_version_id != identity.version
    {
        return Err(Denial::InvalidPhysicalSource(kind));
    }
    Ok(())
}
