use super::WorthQueryCorrectionRestoredSupportDenial as Stop;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use worth_execution::{ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::CanonicalFieldPath;
use worth_relational::facade::history::{
    RelationalAuthoredEntityIndexDefinition as Definition, RelationalAuthoredWriteScope as Scope,
    RelationalAuthoredWriteSelector as Key,
};
use worth_relational::facade::indexes::DerivedIndexKind;
use worth_relational::facade::runtime::VisibilityProjectionView;
use worth_relational::facade::storage::AuthoritativeFieldComparisonKey as Comparison;
use worth_relational::facade::transactions::RecordRef;

mod aspect;

pub(super) struct Selection {
    pub(super) keys: ExecutionArray<Key>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// Project every eligible captured fact or refuse the whole selection.
pub(super) fn select(
    execution: ExecutionRequest<'_, '_>,
    facts: &FactPayload,
    original: &VisibilityProjectionView,
    target: &VisibilityProjectionView,
) -> Result<Selection, Stop> {
    super::admit_work(execution, facts.len() as u64)?;
    let mut count = 0usize;
    let mut bytes = 0u64;
    for fact in facts.iter() {
        execution.consult().map_err(Stop::Request)?;
        let (entries, payload) = match fact {
            Fact::SourceEntity { .. } | Fact::Entity { .. } => (1, 0),
            Fact::SourceFieldRevision {
                entity_id, locator, ..
            }
            | Fact::Field {
                entity_id, locator, ..
            }
            | Fact::AbsentField {
                entity_id, locator, ..
            } => {
                if entity_id.generation_value() == 0 {
                    return Err(unsupported("field-generation", fact));
                }
                if locator.aspect().authority()
                    != worth_foundational::facade::LocatorAuthority::Authoritative
                {
                    return Err(unsupported("field-authority", fact));
                }
                if locator.field_path().fields().len() != 1 {
                    return Err(unsupported("field-path", fact));
                }
                let roster_work = u64::try_from(locator.field_path().fields().len())
                    .ok()
                    .and_then(|count| count.checked_add(1))
                    .ok_or(Stop::Interrupted(
                        worth_execution::MapKernelStop::WorkCounterOverflow,
                    ))?;
                super::admit_work(execution, roster_work)?;
                let payload = (locator
                    .aspect()
                    .aspect_key()
                    .owned_allocation_capacity_bytes() as u64)
                    .checked_mul(2)
                    .and_then(|bytes| {
                        bytes.checked_add(
                            locator.field_path().owned_allocation_capacity_bytes() as u64
                        )
                    })
                    .ok_or(Stop::SizeOverflow)?;
                (3, payload)
            }
            Fact::IndexedEntitySelection {
                index_id,
                definition,
                locator,
                value,
                ..
            } => {
                let DerivedIndexKind::EntityField { field_locator } = &definition.kind else {
                    return Err(unsupported("index-kind", fact));
                };
                // The actual Native receiver admits this exact installed
                // coordinate grammar; retain Planned descriptors unchanged.
                if locator.field_path().fields().len() != 1 {
                    return Err(unsupported("index-path", fact));
                }
                super::admit_work(
                    execution,
                    (locator.aspect().aspect_key().as_str().len() as u64)
                        .checked_add(locator.field_path().fields()[0].as_str().len() as u64)
                        .and_then(|n| n.checked_add(value.semantic_byte_width() as u64))
                        .ok_or(Stop::Interrupted(
                            worth_execution::MapKernelStop::WorkCounterOverflow,
                        ))?,
                )?;
                if definition.index_id != *index_id {
                    return Err(unsupported("index-definition-identity", fact));
                }
                if field_locator != locator {
                    return Err(unsupported("index-definition-locator", fact));
                }
                let payload = (definition.name.capacity() as u64)
                    .checked_add(
                        locator
                            .aspect()
                            .aspect_key()
                            .owned_allocation_capacity_bytes() as u64,
                    )
                    .and_then(|n| {
                        n.checked_add(locator.field_path().owned_allocation_capacity_bytes() as u64)
                    })
                    .and_then(|n| {
                        n.checked_add(Comparison::required_encoded_capacity_bytes(value)?)
                    })
                    .ok_or(Stop::SizeOverflow)?;
                (1, payload)
            }
            Fact::OptionalAspectAbsent { .. } => {
                return Err(unsupported("OptionalAspectAbsent", fact))
            }
            Fact::AspectPresence {
                entity_id,
                kind,
                aspect,
                ..
            } => {
                let scalar = original
                    .entity_aspect_presence_scalar_field(
                        &execution, target, *entity_id, *kind, aspect,
                    )
                    .map_err(Stop::AspectPresence)?;
                let aspect_bytes = aspect.owned_allocation_capacity_bytes() as u64;
                match scalar {
                    None => (2, aspect_bytes),
                    Some(field) => {
                        let bytes = aspect_bytes
                            .checked_mul(2)
                            .and_then(|bytes| {
                                bytes.checked_add(std::mem::size_of_val(field) as u64)
                            })
                            .and_then(|bytes| {
                                bytes.checked_add(field.owned_allocation_capacity_bytes() as u64)
                            })
                            .ok_or(Stop::SizeOverflow)?;
                        (3, bytes)
                    }
                }
            }
            Fact::SourceAspectRevision {
                entity_id, aspect, ..
            } => aspect::measure(execution, original, target, *entity_id, aspect)?,
            Fact::SourceAdjacencyRevision { anchor, .. } => {
                if anchor.generation_value() == 0 {
                    return Err(unsupported("adjacency-generation", fact));
                }
                (2, 0)
            }
            Fact::Relation { .. } => return Err(unsupported("Relation", fact)),
            Fact::Adjacency { .. } => return Err(unsupported("Adjacency", fact)),
            Fact::StructuralWalk { .. } => return Err(unsupported("StructuralWalk", fact)),
            Fact::Aggregate { .. } => return Err(unsupported("Aggregate", fact)),
            Fact::ComponentSelection { .. } => return Err(unsupported("ComponentSelection", fact)),
            Fact::SourcePublication { .. } => return Err(unsupported("SourcePublication", fact)),
            Fact::RetiredOutputEntity { entity_id, .. } => {
                if entity_id.generation_value() == 0 {
                    return Err(unsupported("retirement-generation", fact));
                }
                // The sealer records the exact committed retirement tuple.
                // Lifecycle occurrences are selected here; strict original and
                // target comparison still read that same full-GEN retirement.
                (1, 0)
            }
            Fact::WorkflowDefinitionPredecessor { .. } => {
                return Err(unsupported("WorkflowDefinitionPredecessor", fact))
            }
            Fact::WorkflowDefinitionCurrent { .. } => {
                return Err(unsupported("WorkflowDefinitionCurrent", fact))
            }
            Fact::WorkflowInstanceCapacity { .. } => {
                return Err(unsupported("WorkflowInstanceCapacity", fact))
            }
            Fact::WorkflowHistoryBasis { .. } => {
                return Err(unsupported("WorkflowHistoryBasis", fact))
            }
        };
        count = count.checked_add(entries).ok_or(Stop::SizeOverflow)?;
        bytes = bytes.checked_add(payload).ok_or(Stop::SizeOverflow)?;
    }
    let copy_work = bytes.checked_add(count as u64).ok_or(Stop::Interrupted(
        worth_execution::MapKernelStop::WorkCounterOverflow,
    ))?;
    super::admit_work(execution, copy_work)?;
    let memory = execution
        .reserve_retained_memory(bytes)
        .map_err(Stop::Request)?;
    let mut keys = ExecutionRequestArrayBuilder::allocate(execution, count).map_err(array_stop)?;
    for fact in facts.iter() {
        execution.consult().map_err(Stop::Request)?;
        if let Fact::SourceAdjacencyRevision {
            relation_kind,
            anchor,
            direction,
            ..
        } = fact
        {
            use worth_relational::facade::history::RelationalTouchAdjacencyDirection as Direction;
            keys.push(Key::Adjacency {
                anchor: *anchor,
                kind: *relation_kind,
                direction: match direction {
                    worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing => {
                        Direction::Outgoing
                    }
                    worth_relational::facade::runtime::RelationalAdjacencyDirection::Incoming => {
                        Direction::Incoming
                    }
                },
            })
            .map_err(array_stop)?;
            keys.push(Key::Record {
                record: RecordRef::Entity(*anchor),
                scope: Scope::Lifecycle,
            })
            .map_err(array_stop)?;
            continue;
        }
        if let Fact::IndexedEntitySelection {
            definition,
            entity_kind,
            locator,
            value,
            ..
        } = fact
        {
            keys.push(Key::EntityFieldIndex {
                definition: Definition {
                    index_id: definition.index_id,
                    name: definition.name.clone(),
                    branch_scoped: definition.branch_scoped,
                    locator: locator.clone(),
                },
                kind: *entity_kind,
                value: Comparison::from_aspect_value(value),
            })
            .map_err(array_stop)?;
            continue;
        }
        let entity = match fact {
            Fact::SourceEntity { entity_id }
            | Fact::Entity { entity_id, .. }
            | Fact::SourceFieldRevision { entity_id, .. }
            | Fact::SourceAspectRevision { entity_id, .. }
            | Fact::Field { entity_id, .. }
            | Fact::AbsentField { entity_id, .. }
            | Fact::RetiredOutputEntity { entity_id, .. } => *entity_id,
            Fact::AspectPresence { entity_id, .. } => *entity_id,
            _ => unreachable!("fully projected above"),
        };
        keys.push(Key::Record {
            record: RecordRef::Entity(entity),
            scope: Scope::Lifecycle,
        })
        .map_err(array_stop)?;
        if let Fact::SourceAspectRevision {
            entity_id, aspect, ..
        } = fact
        {
            aspect::append(execution, original, target, *entity_id, aspect, &mut keys)?;
        }
        if let Fact::AspectPresence {
            entity_id,
            kind,
            aspect,
            ..
        } = fact
        {
            keys.push(Key::Record {
                record: RecordRef::Entity(entity),
                scope: Scope::WholeAspect(aspect.clone()),
            })
            .map_err(array_stop)?;
            if let Some(field) = original
                .entity_aspect_presence_scalar_field(&execution, target, *entity_id, *kind, aspect)
                .map_err(Stop::AspectPresence)?
            {
                keys.push(Key::Record {
                    record: RecordRef::Entity(entity),
                    scope: Scope::Field(aspect.clone(), CanonicalFieldPath::single(field.clone())),
                })
                .map_err(array_stop)?;
            }
        }
        if let Fact::SourceFieldRevision { locator, .. }
        | Fact::Field { locator, .. }
        | Fact::AbsentField { locator, .. } = fact
        {
            keys.push(Key::Record {
                record: RecordRef::Entity(entity),
                scope: Scope::WholeAspect(locator.aspect().aspect_key().clone()),
            })
            .map_err(array_stop)?;
            keys.push(Key::Record {
                record: RecordRef::Entity(entity),
                scope: Scope::Field(
                    locator.aspect().aspect_key().clone(),
                    locator.field_path().clone(),
                ),
            })
            .map_err(array_stop)?;
        }
    }
    Ok(Selection {
        keys: keys.seal().map_err(array_stop)?,
        _memory: memory,
    })
}

fn array_stop(cause: worth_execution::ExecutionRequestArrayDenial) -> Stop {
    match cause {
        worth_execution::ExecutionRequestArrayDenial::Request(cause) => Stop::Request(cause),
        worth_execution::ExecutionRequestArrayDenial::Allocation(cause) => Stop::Allocation(cause),
    }
}

fn unsupported(reason: &'static str, fact: &Fact) -> Stop {
    crate::domain_computation::primary_graph::fact_verification_observation::correction_restored_support_unsupported(reason);
    crate::domain_computation::primary_graph::fact_verification_observation::correction_rejection(
        "overlap-selection",
        Some(fact),
        &Stop::UnsupportedFact,
    );
    Stop::UnsupportedFact
}
