use super::WorthQueryCorrectionRestoredSupportDenial as Stop;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use worth_execution::{ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_relational::facade::history::{
    RelationalAuthoredEntityIndexDefinition as Definition, RelationalAuthoredWriteScope as Scope,
    RelationalAuthoredWriteSelector as Key,
};
use worth_relational::facade::indexes::DerivedIndexKind;
use worth_relational::facade::storage::AuthoritativeFieldComparisonKey as Comparison;
use worth_relational::facade::transactions::RecordRef;

pub(super) struct Selection {
    pub(super) keys: ExecutionArray<Key>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// The scalar MVP projects every captured fact or refuses the whole selection.
pub(super) fn select(
    execution: ExecutionRequest<'_, '_>,
    facts: &FactPayload,
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
                    return Err(unsupported("field-generation"));
                }
                if locator.aspect().authority()
                    != worth_foundational::facade::LocatorAuthority::Authoritative
                {
                    return Err(unsupported("field-authority"));
                }
                if locator.field_path().fields().len() != 1 {
                    return Err(unsupported("field-path"));
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
                    return Err(unsupported("index-kind"));
                };
                // The actual Native receiver admits this exact installed
                // coordinate grammar; retain Planned descriptors unchanged.
                if locator.field_path().fields().len() != 1 {
                    return Err(unsupported("index-path"));
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
                    return Err(unsupported("index-definition-identity"));
                }
                if field_locator != locator {
                    return Err(unsupported("index-definition-locator"));
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
            Fact::OptionalAspectAbsent { .. } => return Err(unsupported("OptionalAspectAbsent")),
            Fact::AspectPresence { .. } => return Err(unsupported("AspectPresence")),
            Fact::SourceAspectRevision { .. } => return Err(unsupported("SourceAspectRevision")),
            Fact::SourceAdjacencyRevision { .. } => {
                return Err(unsupported("SourceAdjacencyRevision"))
            }
            Fact::Relation { .. } => return Err(unsupported("Relation")),
            Fact::Adjacency { .. } => return Err(unsupported("Adjacency")),
            Fact::StructuralWalk { .. } => return Err(unsupported("StructuralWalk")),
            Fact::Aggregate { .. } => return Err(unsupported("Aggregate")),
            Fact::ComponentSelection { .. } => return Err(unsupported("ComponentSelection")),
            Fact::SourcePublication { .. } => return Err(unsupported("SourcePublication")),
            Fact::RetiredOutputEntity { .. } => return Err(unsupported("RetiredOutputEntity")),
            Fact::WorkflowDefinitionPredecessor { .. } => {
                return Err(unsupported("WorkflowDefinitionPredecessor"))
            }
            Fact::WorkflowDefinitionCurrent { .. } => {
                return Err(unsupported("WorkflowDefinitionCurrent"))
            }
            Fact::WorkflowInstanceCapacity { .. } => {
                return Err(unsupported("WorkflowInstanceCapacity"))
            }
            Fact::WorkflowHistoryBasis { .. } => return Err(unsupported("WorkflowHistoryBasis")),
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
            | Fact::Field { entity_id, .. }
            | Fact::AbsentField { entity_id, .. } => *entity_id,
            _ => unreachable!("fully projected above"),
        };
        keys.push(Key::Record {
            record: RecordRef::Entity(entity),
            scope: Scope::Lifecycle,
        })
        .map_err(array_stop)?;
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

fn unsupported(reason: &'static str) -> Stop {
    crate::domain_computation::primary_graph::fact_verification_observation::correction_restored_support_unsupported(reason);
    Stop::UnsupportedFact
}
