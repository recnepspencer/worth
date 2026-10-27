//! Canonical law admission precedes any aggregate naming consequence.
use super::evidence::violation;
use super::{
    DerivedRelationGraphLaws as Laws, DerivedRelationLaw as Law,
    DerivedRelationLawViolation as Violation,
};
use crate::config::data::CrossContextPolicy;
use crate::derived_relation::affected::DerivedAffectedRoots;
use crate::derived_relation::evaluation::{
    evaluate_position, insert_member, member_work, SourceRead,
};
use crate::derived_relation::traversal::{available, walk};
use crate::derived_relation::{
    DerivedRelationData as Data, DerivedRelationDenial as DefinitionDenial,
    DerivedRelationHandle as Handle, DerivedRelationReadDenial as Denial,
};
use crate::graph_member::NativeMemberKey as Member;
use crate::schema::data::RelationalSchemaRegistry;
use std::collections::BTreeSet;

pub(crate) fn validate(
    registry: &RelationalSchemaRegistry,
    definition: &Handle,
    affected: &DerivedAffectedRoots,
    before: &mut impl SourceRead,
    after: &mut impl SourceRead,
) -> Result<Option<Violation>, Denial> {
    let Some(laws) = &definition.parts().laws else {
        return Ok(None);
    };
    after.checkpoint(
        definition
            .parts()
            .from
            .name_bytes()
            .and_then(|n| n.checked_add(definition.parts().to.name_bytes()?))
            .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
    )?;
    let mut predecessor_support = Vec::new();
    let mut sources = BTreeSet::new();
    for root in &affected.roots {
        insert_member(after, &mut sources, root.clone())?;
    }
    if definition.parts().from == definition.parts().to && (laws.symmetric || laws.orbit) {
        for target in &affected.targets {
            insert_member(after, &mut sources, target.clone())?;
        }
    }
    // A source delta can invalidate a predecessor's transitive minimum even
    // when that predecessor's immediate endpoint set is unchanged.
    if laws.reaches_at_least.is_some() {
        after.retain(
            sources
                .len()
                .checked_mul(size_of::<Member>())
                .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
        )?;
        let seeds = sources.iter().cloned().collect::<Vec<_>>();
        for traversal in [
            walk(registry, definition, &seeds, true, before)?,
            walk(registry, definition, &seeds, true, after)?,
        ] {
            for member in traversal.members {
                insert_member(after, &mut sources, member)?;
            }
            let count = predecessor_support
                .len()
                .checked_add(traversal.support.len())
                .and_then(|n| {
                    n.checked_mul(size_of::<
                        crate::derived_relation::traversal::DerivedTraversalStep,
                    >())
                })
                .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
            after.retain(count)?;
            predecessor_support.extend(traversal.support);
        }
    }
    for source in &sources {
        if !available(registry, &definition.parts().from, source, after)? {
            continue;
        }
        let data = read(registry, definition, source, false, after)?;
        if let Some(failure) = source_laws(registry, definition, laws, source, &data, after)? {
            return Ok(Some(failure));
        }
    }
    for target in &affected.targets {
        if !available(registry, &definition.parts().to, target, after)? {
            continue;
        }
        let data = read(registry, definition, target, true, after)?;
        let count = u64::try_from(data.endpoints().len())
            .map_err(|_| DefinitionDenial::WorkBudgetExceeded)?;
        let failed = if laws.orbit && count != 1 {
            Some((Law::Orbit, 1))
        } else if laws
            .cardinality
            .target_min
            .is_some_and(|minimum| count < minimum)
        {
            Some((Law::TargetMinimum, laws.cardinality.target_min.unwrap()))
        } else if laws
            .cardinality
            .target_max
            .is_some_and(|maximum| count > maximum)
        {
            Some((Law::TargetMaximum, laws.cardinality.target_max.unwrap()))
        } else {
            None
        };
        if let Some((law, limit)) = failed {
            return violation(law, target, None, count, limit, [&data], after).map(Some);
        }
    }
    Ok(None)
}

fn source_laws(
    registry: &RelationalSchemaRegistry,
    definition: &Handle,
    laws: &Laws,
    source: &Member,
    data: &Data,
    read_source: &mut impl SourceRead,
) -> Result<Option<Violation>, Denial> {
    let count =
        u64::try_from(data.endpoints().len()).map_err(|_| DefinitionDenial::WorkBudgetExceeded)?;
    let failed = if laws.orbit && count != 1 {
        Some((Law::Orbit, 1))
    } else if laws
        .cardinality
        .source_min
        .is_some_and(|minimum| count < minimum)
    {
        Some((Law::SourceMinimum, laws.cardinality.source_min.unwrap()))
    } else if laws
        .cardinality
        .source_max
        .is_some_and(|maximum| count > maximum)
    {
        Some((Law::SourceMaximum, laws.cardinality.source_max.unwrap()))
    } else {
        None
    };
    if let Some((law, limit)) = failed {
        return violation(law, source, None, count, limit, [data], read_source).map(Some);
    }
    for target in data.endpoints() {
        member_work(read_source, target, 1)?;
        let failed = if !laws.self_edges_allowed && source == target {
            Some((Law::SelfEdges, 0))
        } else if laws.cross_context_policy != CrossContextPolicy::AllowExplicit
            && source.carrier().partition_id != target.carrier().partition_id
        {
            Some((Law::CrossContext, 0))
        } else if laws.same_partition_endpoints
            && source.carrier().partition_id != target.carrier().partition_id
        {
            Some((Law::SamePartition, 0))
        } else if laws.cardinality.pair_min.is_some_and(|minimum| minimum > 1) {
            Some((Law::PairMinimum, laws.cardinality.pair_min.unwrap()))
        } else {
            None
        };
        if let Some((law, limit)) = failed {
            return violation(law, source, Some(target), 1, limit, [data], read_source).map(Some);
        }
        if laws.symmetric {
            let counterpart = if available(registry, &definition.parts().from, target, read_source)?
            {
                Some(read(registry, definition, target, false, read_source)?)
            } else {
                None
            };
            member_work(
                read_source,
                source,
                counterpart
                    .as_ref()
                    .map_or(1, |data| data.endpoints().len()),
            )?;
            if counterpart
                .as_ref()
                .is_none_or(|counterpart| counterpart.endpoints().binary_search(source).is_err())
            {
                return violation(
                    Law::Symmetric,
                    source,
                    Some(target),
                    0,
                    1,
                    std::iter::once(data).chain(counterpart.as_ref()),
                    read_source,
                )
                .map(Some);
            }
        }
        if laws.acyclic {
            let traversal = walk(
                registry,
                definition,
                std::slice::from_ref(target),
                false,
                read_source,
            )?;
            member_work(read_source, source, traversal.members.len())?;
            if traversal.members.contains(source) {
                return violation(
                    Law::Acyclic,
                    source,
                    Some(target),
                    1,
                    0,
                    std::iter::once(data).chain(traversal.support.iter().map(|step| &step.data)),
                    read_source,
                )
                .map(Some);
            }
        }
    }
    if let Some(minimum) = &laws.reaches_at_least {
        let traversal = walk(
            registry,
            definition,
            std::slice::from_ref(source),
            false,
            read_source,
        )?;
        let mut count = 0_u64;
        for member in &traversal.members {
            member_work(read_source, member, 1)?;
            if member == source {
                continue;
            }
            for domain in &minimum.target_domains {
                if available(registry, domain, member, read_source)? {
                    count = count
                        .checked_add(1)
                        .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
                    break;
                }
            }
        }
        if count < u64::from(minimum.minimum_reachable_targets) {
            return violation(
                Law::MinimumReachability,
                source,
                None,
                count,
                u64::from(minimum.minimum_reachable_targets),
                std::iter::once(data).chain(traversal.support.iter().map(|step| &step.data)),
                read_source,
            )
            .map(Some);
        }
    }
    Ok(None)
}

fn read(
    registry: &RelationalSchemaRegistry,
    definition: &Handle,
    root: &Member,
    reverse: bool,
    source: &mut impl SourceRead,
) -> Result<Data, Denial> {
    evaluate_position(
        registry,
        definition,
        definition.parts().nodes.len() - 1,
        reverse,
        root.clone(),
        source,
    )
}
