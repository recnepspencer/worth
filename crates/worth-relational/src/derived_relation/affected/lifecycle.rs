//! Existence/generation seeds include every case even with empty adjacency.
use super::super::evaluation::{insert_member, SourceRead};
use super::super::{
    DerivedRelationDenial as DefinitionDenial, DerivedRelationReadDenial as Denial,
};
use crate::graph_member::{
    NativeMemberDomain as Domain, NativeMemberKey as Member, ProjectedMemberCoordinate,
};
use crate::identity::data::{EntityId, KindId};
use crate::schema::data::RelationalSchemaRegistry;
use crate::transactions::data::RecordRef;
use std::collections::BTreeSet;
use std::sync::Arc;

pub(super) struct Lifecycle {
    pub(super) carrier: EntityId,
    pub(super) before: Option<KindId>,
    pub(super) after: Option<KindId>,
}

pub(super) fn kind(
    read: &mut impl SourceRead,
    carrier: EntityId,
) -> Result<Option<KindId>, Denial> {
    match read.entity_kind(carrier) {
        Ok(kind) => Ok(Some(kind)),
        Err(Denial::Definition(DefinitionDenial::RecordUnavailable)) => Ok(None),
        Err(cause) => Err(cause),
    }
}

pub(super) fn observe(
    records: &[RecordRef],
    before: &mut impl SourceRead,
    after: &mut impl SourceRead,
) -> Result<Vec<Lifecycle>, Denial> {
    let mut events = Vec::new();
    for record in records {
        after.checkpoint(1)?;
        let RecordRef::Entity(carrier) = record else {
            continue;
        };
        let old = kind(before, *carrier)?;
        let new = kind(after, *carrier)?;
        if old != new {
            after.retain(size_of::<Lifecycle>())?;
            events.push(Lifecycle {
                carrier: *carrier,
                before: old,
                after: new,
            });
        }
    }
    Ok(events)
}

pub(super) fn seed_domain(
    registry: &RelationalSchemaRegistry,
    domain: &Domain,
    events: &[Lifecycle],
    roots: &mut BTreeSet<Member>,
    read: &mut impl SourceRead,
) -> Result<(), Denial> {
    match domain {
        Domain::Entity(expected) => {
            for event in events {
                read.checkpoint(1)?;
                if event.before == Some(*expected) || event.after == Some(*expected) {
                    insert_member(read, roots, event.carrier.into())?;
                }
            }
        }
        Domain::Projected { projected } => {
            let names = projected
                .name_bytes()
                .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
            let levels = (usize::BITS
                - registry
                    .projected_member_domains
                    .len()
                    .max(1)
                    .leading_zeros()) as usize;
            read.checkpoint(
                names
                    .checked_mul(11)
                    .and_then(|n| n.checked_mul(levels))
                    .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
            )?;
            let registered = registry
                .projected_member_domain(projected)
                .ok_or(DefinitionDenial::InvalidMemberDomain)?;
            for event in events {
                read.checkpoint(1)?;
                if event.before != Some(registered.parts().carrier)
                    && event.after != Some(registered.parts().carrier)
                {
                    continue;
                }
                for case in &registered.parts().cases {
                    read.checkpoint(case.len())?;
                    let bytes = names
                        .checked_add(case.len())
                        .and_then(|n| {
                            n.checked_add(
                                size_of::<ProjectedMemberCoordinate>() + 2 * size_of::<usize>(),
                            )
                        })
                        .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
                    read.retain(bytes)?;
                    insert_member(
                        read,
                        roots,
                        Member::Projected(Arc::new(ProjectedMemberCoordinate {
                            domain: projected.clone(),
                            carrier: event.carrier,
                            case: case.clone(),
                        })),
                    )?;
                }
            }
        }
    }
    Ok(())
}
