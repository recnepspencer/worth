//! Authorization support enters ordinary history selection before managed narrowing.
use super::*;
use crate::domain_computation::{
    authorization::SourceAuthorizationLocus as Locus,
    primary_graph::application_attempt::SourceReadAuthorization,
};

pub(super) fn measure(
    execution: ExecutionRequest<'_, '_>,
    observation: &SourceReadAuthorization,
) -> Result<(usize, u64), Stop> {
    let mut entries = 0usize;
    let mut bytes = 0u64;
    let conventional = observation.visit_loci(&mut |locus| {
        super::super::admit_work(execution, 1)?;
        let (count, payload) = match locus {
            Locus::Entity(_) | Locus::Relation(_) | Locus::Adjacency(_, _, _) => (1, 0),
            Locus::Field(_, field) => {
                let bytes = field
                    .aspect()
                    .aspect_key()
                    .owned_allocation_capacity_bytes() as u64;
                (
                    3,
                    bytes
                        .checked_mul(2)
                        .and_then(|n| {
                            n.checked_add(
                                field.field_path().owned_allocation_capacity_bytes() as u64
                            )
                        })
                        .ok_or(Stop::SizeOverflow)?,
                )
            }
        };
        entries = entries.checked_add(count).ok_or(Stop::SizeOverflow)?;
        bytes = bytes.checked_add(payload).ok_or(Stop::SizeOverflow)?;
        Ok(())
    })?;
    if !conventional {
        return Err(Stop::UnsupportedFact);
    }
    Ok((entries, bytes))
}

pub(super) fn append(
    execution: ExecutionRequest<'_, '_>,
    observation: &SourceReadAuthorization,
    keys: &mut ExecutionRequestArrayBuilder<'_, '_, Key>,
) -> Result<(), Stop> {
    use worth_relational::facade::{
        authorization::RelationalAuthorizationTraversalDirection as Direction,
        history::RelationalTouchAdjacencyDirection as Touch,
    };
    let conventional = observation.visit_loci(&mut |locus| {
        execution.consult().map_err(Stop::Request)?;
        match locus {
            Locus::Entity(entity) => keys
                .push(Key::Record {
                    record: RecordRef::Entity(entity),
                    scope: Scope::Lifecycle,
                })
                .map_err(array_stop)?,
            Locus::Relation(relation) => keys
                .push(Key::Record {
                    record: RecordRef::Relation(relation),
                    scope: Scope::Lifecycle,
                })
                .map_err(array_stop)?,
            Locus::Adjacency(anchor, kind, direction) => keys
                .push(Key::Adjacency {
                    anchor,
                    kind,
                    direction: match direction {
                        Direction::Forward => Touch::Outgoing,
                        Direction::Reverse => Touch::Incoming,
                    },
                })
                .map_err(array_stop)?,
            Locus::Field(entity, field) => {
                keys.push(Key::Record {
                    record: RecordRef::Entity(entity),
                    scope: Scope::Lifecycle,
                })
                .map_err(array_stop)?;
                keys.push(Key::Record {
                    record: RecordRef::Entity(entity),
                    scope: Scope::WholeAspect(field.aspect().aspect_key().clone()),
                })
                .map_err(array_stop)?;
                keys.push(Key::Record {
                    record: RecordRef::Entity(entity),
                    scope: Scope::Field(
                        field.aspect().aspect_key().clone(),
                        field.field_path().clone(),
                    ),
                })
                .map_err(array_stop)?;
            }
        }
        Ok(())
    })?;
    if !conventional {
        return Err(Stop::UnsupportedFact);
    }
    Ok(())
}
