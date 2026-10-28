//! Complete affected source acquisition; endpoint pairs never become stored edges.
use super::*;
use crate::derived_relation::evaluation::SourceRead;
use crate::derived_relation::traversal::{available, walk, walk_undirected, DerivedTraversalStep};
use std::mem::size_of;

pub(super) struct Discovery {
    pub(super) groups: Vec<BTreeSet<Member>>,
    pub(super) pairs: BTreeSet<(Member, Member)>,
    // Keep both positive and explored empty physical support through candidate
    // construction, even when Boolean/path outputs deduplicate to equal pairs.
    pub(super) support: Vec<DerivedTraversalStep>,
}

pub(super) fn collect(
    registry: &RelationalSchemaRegistry,
    handle: &DerivedRelationHandle,
    seeds: &BTreeSet<Member>,
    components: bool,
    source: &mut impl SourceRead,
) -> Result<Discovery, DerivedRelationReadDenial> {
    let mut result = Discovery {
        groups: Vec::new(),
        pairs: BTreeSet::new(),
        support: Vec::new(),
    };
    let mut visited = BTreeSet::new();
    for seed in seeds {
        lookup(seed, visited.len(), source)?;
        if visited.contains(seed) || !available(registry, &handle.parts().from, seed, source)? {
            continue;
        }
        let traversal = if components {
            walk_undirected(registry, handle, std::slice::from_ref(seed), source)?
        } else {
            walk(registry, handle, std::slice::from_ref(seed), false, source)?
        };
        for member in &traversal.members {
            insert(&mut visited, member.clone(), source)?;
        }
        for step in traversal.support {
            for endpoint in step.data.endpoints() {
                source.checkpoint(1)?;
                lookup(&step.anchor, 1, source)?;
                let (a, b) = if step.reverse {
                    (endpoint, &step.anchor)
                } else {
                    (&step.anchor, endpoint)
                };
                if a == b {
                    continue;
                }
                let pair = if components && a > b {
                    (b.clone(), a.clone())
                } else {
                    (a.clone(), b.clone())
                };
                lookup(&pair.0, result.pairs.len(), source)?;
                lookup(&pair.1, result.pairs.len(), source)?;
                source.retain(11 * size_of::<(Member, Member)>() + 14 * size_of::<usize>())?;
                result.pairs.insert(pair);
            }
            source.retain(4 * size_of::<DerivedTraversalStep>())?;
            result.support.push(step);
        }
        source.retain(4 * size_of::<BTreeSet<Member>>())?;
        result.groups.push(traversal.members);
    }
    Ok(result)
}

pub(super) fn insert(
    set: &mut BTreeSet<Member>,
    member: Member,
    source: &mut impl SourceRead,
) -> Result<(), DerivedRelationReadDenial> {
    lookup(&member, set.len(), source)?;
    if !set.contains(&member) {
        source.retain(11 * size_of::<Member>() + 14 * size_of::<usize>())?;
        lookup(&member, set.len(), source)?;
        set.insert(member);
    }
    Ok(())
}
fn lookup(
    member: &Member,
    count: usize,
    source: &mut impl SourceRead,
) -> Result<(), DerivedRelationReadDenial> {
    source.checkpoint(
        member
            .comparison_work(count)
            .ok_or(DerivedRelationDenial::WorkBudgetExceeded)?,
    )
}
