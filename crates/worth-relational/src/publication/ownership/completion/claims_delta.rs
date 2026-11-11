//! The desired origin is truth; only absent, new or changed claims write rows.
use super::*;
use crate::history::data::CanonicalRequiredOutputOrigin;
use crate::mvcc::RequiredOutputRolePosture::Retire;
use crate::publication::ownership::{read::PublicationRow, record};
use crate::schema::{current_publication_ownership::EDGE, data::CurrentPublicationStorage};

pub(super) fn advance(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    writer: &mut Writer<'_, '_, '_, '_, '_>,
    storage: &CurrentPublicationStorage,
    old: &PublicationRow,
    next: &CanonicalRequiredOutputOrigin,
) -> Result<(), TransactionCommitError> {
    let custody = old.value.custody();
    let mut claims = crate::publication::ownership::read::publication_claims(
        execution,
        basis,
        old.entity,
        custody,
        writer.context(),
    )?;
    let before_count = claims
        .links
        .iter()
        .filter(|link| [storage.member_link, storage.edge_link].contains(&link.kind))
        .count();
    let after_count = next
        .members
        .iter()
        .filter(|member| member.posture != Retire)
        .count()
        .checked_add(next.relations.len())
        .ok_or_else(overflow)?;
    let capacity = before_count.checked_add(after_count).ok_or_else(overflow)?;
    let lookup_count = old
        .value
        .origin
        .members
        .len()
        .checked_add(old.value.origin.relations.len())
        .and_then(|count| count.checked_add(next.relations.len()))
        .ok_or_else(overflow)?;
    let bytes = before_count
        .checked_mul(std::mem::size_of::<plan::StoredClaim>())
        .and_then(|bytes| {
            after_count
                .checked_mul(std::mem::size_of::<plan::Claim>())
                .and_then(|next| bytes.checked_add(next))
        })
        .and_then(|bytes| {
            capacity
                .checked_mul(std::mem::size_of::<plan::Change>())
                .and_then(|plan| bytes.checked_add(plan))
        })
        .and_then(|bytes| {
            lookup_count
                .checked_mul(std::mem::size_of::<
                    &crate::history::data::CanonicalOutputMember,
                >())
                .and_then(|lookup| bytes.checked_add(lookup))
        })
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(overflow)?;
    let _scratch = execution
        .reserve_retained_memory(bytes)
        .map_err(crate::publication::ownership::request)?;
    let visited = capacity.checked_add(lookup_count).ok_or_else(overflow)?;
    let navigation = (usize::BITS - visited.leading_zeros()) as usize + 1;
    writer.checkpoint(visited.checked_mul(navigation).ok_or_else(overflow)?)?;
    let mut old_members = old.value.origin.members.iter().collect::<Vec<_>>();
    let mut old_relations = old.value.origin.relations.iter().collect::<Vec<_>>();
    let mut next_relations = next.relations.iter().collect::<Vec<_>>();
    next_relations.sort_unstable_by_key(|edge| edge.id);
    old_members.sort_unstable_by_key(|member| member.id);
    old_relations.sort_unstable_by_key(|edge| edge.id);
    claims.edges.sort_unstable_by_key(|(id, _)| *id);
    let mut before = Vec::with_capacity(before_count);
    for link in &claims.links {
        writer.checkpoint(navigation)?;
        if link.kind == storage.member_link {
            if old_members
                .binary_search_by_key(&link.source, |member| member.id)
                .ok()
                .is_none_or(|index| old_members[index].posture == Retire)
            {
                return Err(crate::publication::ownership::corrupt(
                    custody,
                    RecordRef::Relation(link.id),
                ));
            }
            before.push(plan::StoredClaim::Member {
                entity: link.source,
                link: link.id,
            });
        } else if link.kind == storage.edge_link {
            let witness = claims
                .edges
                .binary_search_by_key(&link.source, |(id, _)| *id)
                .ok()
                .map(|index| &claims.edges[index])
                .ok_or_else(|| {
                    crate::publication::ownership::corrupt(custody, RecordRef::Entity(link.source))
                })?;
            if old_relations
                .binary_search_by_key(&witness.1.edge.id, |edge| edge.id)
                .ok()
                .is_none_or(|index| {
                    !record::RelationClaim::from(&witness.1.edge).matches(old_relations[index])
                })
            {
                return Err(crate::publication::ownership::corrupt(
                    custody,
                    RecordRef::Entity(link.source),
                ));
            }
            before.push(plan::StoredClaim::Relation {
                edge: (&witness.1.edge).into(),
                witness: link.source,
                link: link.id,
            });
        }
    }
    before.sort_unstable_by_key(plan::StoredClaim::key);
    if before.windows(2).any(|pair| pair[0].key() == pair[1].key()) {
        return Err(crate::publication::ownership::corrupt(
            custody,
            RecordRef::Entity(old.entity),
        ));
    }
    let mut desired = Vec::with_capacity(after_count);
    desired.extend(
        next.members
            .iter()
            .filter(|member| member.posture != Retire)
            .map(|member| plan::Claim::Member(member.id)),
    );
    desired.extend(
        next.relations
            .iter()
            .map(|edge| plan::Claim::Relation(edge.into())),
    );
    desired.sort_unstable_by_key(plan::Claim::key);
    let changes = plan::changes(&before, &desired);
    for change in changes {
        writer.checkpoint(1)?;
        match change {
            plan::Change::Retire(index) => {
                writer.delete_relation(before[index].link())?;
                if let plan::StoredClaim::Relation { witness, .. } = before[index] {
                    writer.delete_entity(witness)?;
                }
            }
            plan::Change::Insert(index) => match &desired[index] {
                plan::Claim::Member(entity) => {
                    writer.create_link(storage.member_link, custody, *entity, old.entity)?
                }
                plan::Claim::Relation(edge) => {
                    let index = next_relations
                        .binary_search_by_key(&edge.id, |edge| edge.id)
                        .expect("planned edge belongs to the desired origin");
                    insert_edge(writer, storage, old.entity, custody, next_relations[index])?
                }
            },
            plan::Change::Revise { previous, next } => {
                let (plan::StoredClaim::Relation { witness, .. }, plan::Claim::Relation(edge)) =
                    (&before[previous], &desired[next])
                else {
                    unreachable!("one entity identity always names the same member claim")
                };
                writer.update_entity(
                    *witness,
                    EDGE,
                    custody,
                    &record::Edge {
                        publication: custody,
                        edge: (*next_relations[next_relations
                            .binary_search_by_key(&edge.id, |edge| edge.id)
                            .expect("planned revision belongs to the desired origin")])
                        .clone(),
                    },
                )?;
            }
        }
    }
    Ok(())
}

fn overflow() -> TransactionCommitError {
    crate::publication::ownership::request(
        crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
    )
}
mod plan;

pub(super) fn insert(
    writer: &mut Writer<'_, '_, '_, '_, '_>,
    storage: &CurrentPublicationStorage,
    row: crate::identity::data::EntityId,
    custody: CommitId,
    origin: &CanonicalRequiredOutputOrigin,
) -> Result<(), TransactionCommitError> {
    for member in origin
        .members
        .iter()
        .filter(|member| member.posture != Retire)
    {
        writer.create_link(storage.member_link, custody, member.id, row)?;
    }
    for edge in &origin.relations {
        insert_edge(writer, storage, row, custody, edge)?;
    }
    Ok(())
}

fn insert_edge(
    writer: &mut Writer<'_, '_, '_, '_, '_>,
    storage: &CurrentPublicationStorage,
    row: crate::identity::data::EntityId,
    custody: CommitId,
    edge: &crate::history::data::CanonicalRequiredOutputRelation,
) -> Result<(), TransactionCommitError> {
    writer
        .create_entity(
            storage.edge_kind,
            EDGE,
            custody,
            &record::Edge {
                publication: custody,
                edge: edge.clone(),
            },
            Some((storage.edge_link, row, custody)),
        )
        .map(|_| ())
}

pub(super) fn retire(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    writer: &mut Writer<'_, '_, '_, '_, '_>,
    storage: &CurrentPublicationStorage,
    old: &PublicationRow,
    row_retired: bool,
) -> Result<(), TransactionCommitError> {
    let claims = crate::publication::ownership::read::publication_claims(
        execution,
        basis,
        old.entity,
        old.value.custody(),
        writer.context(),
    )?;
    for link in &claims.links {
        writer.checkpoint(1)?;
        if row_retired && link.kind == storage.publication_link
            || [storage.member_link, storage.edge_link].contains(&link.kind)
        {
            writer.delete_relation(link.id)?;
            if link.kind == storage.edge_link {
                writer.delete_entity(link.source)?;
            }
        }
    }
    if row_retired {
        writer.delete_entity(old.entity)?;
    }
    Ok(())
}
