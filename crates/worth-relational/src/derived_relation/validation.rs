//! One finite grammar validator for registration and recovered snapshots.
use std::collections::{BTreeMap, BTreeSet};

use super::{DerivedRelationDefinitionParts as Parts, DerivedRelationDenial as Denial};
use super::{DerivedRelationIdentity as Identity, DerivedRelationNode as Node};
use crate::identity::data::KindId;
use crate::schema::data::RelationalSchemaRegistry;

pub(super) fn expression(
    parts: &Parts,
    definitions: &BTreeMap<Identity, Parts>,
    registry: &RelationalSchemaRegistry,
) -> Result<BTreeSet<Identity>, Denial> {
    if parts.identity.name.trim().is_empty()
        || parts.identity.name.trim() != parts.identity.name
        || parts.identity.schema.0.is_empty()
    {
        return Err(Denial::InvalidIdentity);
    }
    for kind in [parts.from, parts.to] {
        entity_kind(registry, &parts.identity, kind)?;
    }
    let mut endpoints: Vec<(KindId, KindId)> = Vec::new();
    let mut dependencies = BTreeSet::new();
    for node in &parts.nodes {
        let prior = |index: usize| {
            endpoints
                .get(index)
                .copied()
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
                (*from, *to)
            }
            Node::Derived(identity) => {
                let target = definitions
                    .get(identity)
                    .ok_or_else(|| Denial::MissingDefinition(identity.clone()))?;
                dependencies.insert(identity.clone());
                (target.from, target.to)
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
        };
        endpoints.push(pair);
    }
    if endpoints.last() != Some(&(parts.from, parts.to)) {
        return Err(Denial::InvalidExpression);
    }
    let mut visited = BTreeSet::new();
    let mut pending = vec![parts.nodes.len() - 1];
    while let Some(index) = pending.pop() {
        if !visited.insert(index) {
            continue;
        }
        match parts.nodes[index] {
            Node::Compose { left, right } => pending.extend([left, right]),
            Node::Reverse { input } | Node::WithoutSelfEdges { input } => pending.push(input),
            Node::Stored { .. } | Node::Derived(_) => {}
        }
    }
    if visited.len() != parts.nodes.len() {
        return Err(Denial::InvalidExpression);
    }
    Ok(dependencies)
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
