//! Canonical before/after pullback from actual physical and carrier changes.
mod expression;
mod lifecycle;

use super::evaluation::SourceRead;
use super::{DerivedRelationData, DerivedRelationHandle, DerivedRelationReadDenial};
use crate::authority::mutation::AdjacencyDelta;
use crate::graph_member::NativeMemberKey;
use crate::schema::data::RelationalSchemaRegistry;
use crate::transactions::data::RecordRef;
use std::collections::BTreeSet;

pub(crate) struct DerivedAffectedRoots {
    pub(crate) roots: BTreeSet<NativeMemberKey>,
    pub(crate) targets: BTreeSet<NativeMemberKey>,
    // Every before/after inverse-prefix read remains present until this whole
    // candidate frontier has been consumed, including empty adjacency support.
    pub(crate) support: Vec<DerivedRelationData>,
}

pub(crate) fn collect(
    registry: &RelationalSchemaRegistry,
    definition: &DerivedRelationHandle,
    adjacency: &[AdjacencyDelta],
    records: &[RecordRef],
    before: &mut impl SourceRead,
    after: &mut impl SourceRead,
) -> Result<DerivedAffectedRoots, DerivedRelationReadDenial> {
    if !registry.admits_derived_relation(definition) {
        return Err(super::DerivedRelationDenial::ForeignDefinition.into());
    }
    let lifecycle = lifecycle::observe(records, before, after)?;
    let (mut roots, mut support) = expression::collect(
        registry, definition, false, adjacency, &lifecycle, before, after,
    )?;
    let (mut targets, reverse_support) = expression::collect(
        registry, definition, true, adjacency, &lifecycle, before, after,
    )?;
    // Root-domain existence is independent of every coupling/prefix. A new
    // isolated carrier must still satisfy degree/orbit laws and get a cohort.
    lifecycle::seed_domain(
        registry,
        &definition.parts().from,
        &lifecycle,
        &mut roots,
        after,
    )?;
    lifecycle::seed_domain(
        registry,
        &definition.parts().to,
        &lifecycle,
        &mut targets,
        after,
    )?;
    let capacity = support
        .len()
        .checked_add(reverse_support.len())
        .and_then(|n| n.checked_mul(size_of::<DerivedRelationData>()))
        .ok_or(super::DerivedRelationDenial::WorkBudgetExceeded)?;
    after.retain(capacity)?;
    support.extend(reverse_support);
    Ok(DerivedAffectedRoots {
        roots,
        targets,
        support,
    })
}
