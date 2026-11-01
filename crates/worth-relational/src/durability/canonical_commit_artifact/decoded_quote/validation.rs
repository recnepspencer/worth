//! The original validation packet owns rule text, decisions and provenance.
use super::*;
mod admission;
mod touched;
use crate::transactions::data::{
    RelationalInvariantCheckMeaning, RelationalInvariantDecisionMeaning,
    RelationalValidationPreparationFailure,
};
pub(in super::super) use admission::prepare_canonical_readmission;
use touched::{touched_entities, touched_partitions, touched_relations};

pub(super) fn execution(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "metadata" => metadata(value, request)?,
                "checks" => vector::<RelationalInvariantCheckMeaning>(value, request, |entry| {
                    check(entry, request)
                })?,
                "decisions" => {
                    vector::<RelationalInvariantDecisionMeaning>(value, request, |entry| {
                        check(entry, request)
                    })?
                }
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn check(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "witness" => text(value, request)?,
                "rule" => rule(value, request)?,
                "custom_provenance" => optional(value, |entry| provenance(entry, request))?,
                "verdict" => verdict(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn metadata(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| add(&mut bytes, match field {
        "preparation_failures" => vector::<RelationalValidationPreparationFailure>(value, request, |_| Ok(OwnerAllocation::default()))?,
        "proof_scope" => optional(value, |scope| {
            let mut bytes = OwnerAllocation::default();
            scope.fields(request, |field, value| {
                if field == "widened_causes" { add(&mut bytes, vector::<crate::validation::engine::InvariantScopeWideningCause>(value, request, |_| Ok(OwnerAllocation::default()))?)?; }
                Ok(())
            })?;
            Ok(bytes)
        })?,
        _ => OwnerAllocation::default(),
    }))?;
    Ok(bytes)
}
fn identity(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "rule_id" {
            add(&mut bytes, shared_text(value, request)?)?;
            add(
                &mut bytes,
                crate::validation::data::CustomInvariantRuleId::control_bytes()
                    .ok_or_else(malformed)?,
            )?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn rule(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        add(
            &mut bytes,
            match variant {
                "Custom" => identity(value, request)?,
                "Native" => native_rule(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn native_rule(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        match variant {
            "UniqueEntityAspectField" => value.fields(request, |field, value| {
                if field == "field_locator" {
                    add(&mut bytes, super::aspects::locator(value, request)?)?;
                }
                Ok(())
            })?,
            "EndpointKindContract"
            | "CardinalityMaximumContract"
            | "CardinalityMinimumContract"
            | "UniquenessContract"
            | "SymmetryContract"
            | "EndpointDeletionIntegrityContract"
            | "AcyclicityContract"
            | "PartitionIsolationContract"
            | "ConnectivityMinimumContract" => {
                value.fields(request, |field, value| {
                    add(
                        &mut bytes,
                        match field {
                            "contract_id" => text(value, request)?,
                            "allowed_source_kinds"
                            | "allowed_target_kinds"
                            | "candidate_source_kinds"
                            | "candidate_target_kinds"
                            | "source_kind_ids"
                            | "target_kind_ids" => {
                                vector::<crate::identity::data::KindId>(value, request, |_| {
                                    Ok(OwnerAllocation::default())
                                })?
                            }
                            _ => OwnerAllocation::default(),
                        },
                    )
                })?;
            }
            _ => {}
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn provenance(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "touched" {
            value.fields(request, |field, value| {
                add(
                    &mut bytes,
                    match field {
                        "visible_entity_ids" => touched_entities(value, request)?,
                        "visible_relation_ids" => touched_relations(value, request)?,
                        "touched_partition_ids" => touched_partitions(value, request)?,
                        _ => OwnerAllocation::default(),
                    },
                )
            })?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn verdict(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    // Pass and NotApplicable are unit enum strings and own no nested heap.
    if matches!(value.0.first(), Some(0xa0..=0xbf | 0xd9..=0xdb)) {
        return Ok(OwnerAllocation::default());
    }
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        if variant == "Violation" {
            add(&mut bytes, violation(value, request)?)?;
        }
        if variant == "Advisory" {
            value.fields(request, |field, value| {
                if field == "violation" {
                    add(&mut bytes, violation(value, request)?)?;
                }
                Ok(())
            })?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn violation(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "detail" => text(value, request)?,
                "fields" => violation_fields(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn violation_fields(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    if matches!(value.0.first(), Some(0xa0..=0xbf | 0xd9..=0xdb)) {
        return Ok(OwnerAllocation::default());
    }
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        value.fields(request, |field, value| {
            add(
                &mut bytes,
                match field {
                    "contract_id" | "missing_label" | "detail" => {
                        optional(value, |value| text(value, request))?
                    }
                    "field" => optional(value, |value| text(value, request))?,
                    "field_locator" => super::aspects::locator(value, request)?,
                    "value" if variant == "UniqueEntityField" => {
                        super::aspects::scalar(value, request)?
                    }
                    "identity" if variant == "CustomInvariantFailure" => {
                        let mut bytes = OwnerAllocation::default();
                        value.fields(request, |field, value| {
                            if field == "semantic_identity" {
                                add(&mut bytes, identity(value, request)?)?;
                            }
                            Ok(())
                        })?;
                        bytes
                    }
                    "identity" if variant == "CustomInvariantViolation" => {
                        identity(value, request)?
                    }
                    "refusal" => custom_refusal(value, request)?,
                    "definition" if variant == "DerivedRelationGraph" => {
                        named_identity(value, request)?
                    }
                    "violation" if variant == "DerivedRelationGraph" => {
                        derived_violation(value, request)?
                    }
                    _ => OwnerAllocation::default(),
                },
            )
        })
    })?;
    Ok(bytes)
}
fn custom_refusal(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    if matches!(value.0.first(), Some(0xa0..=0xbf | 0xd9..=0xdb)) {
        return Ok(OwnerAllocation::default());
    }
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |_, value| {
        value.fields(request, |field, value| {
            add(
                &mut bytes,
                match field {
                    "value_identity" => text(value, request)?,
                    "bytes" => vector::<u8>(value, request, |_| Ok(OwnerAllocation::default()))?,
                    _ => OwnerAllocation::default(),
                },
            )
        })
    })?;
    Ok(bytes)
}
fn named_identity(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if matches!(field, "schema" | "name") {
            add(&mut bytes, text(value, request)?)?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn derived_violation(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "member" => member(value, request)?,
                "counterpart" => optional(value, |value| member(value, request))?,
                "physical_entities" => {
                    vector::<crate::identity::data::EntityId>(value, request, |_| {
                        Ok(OwnerAllocation::default())
                    })?
                }
                "physical_relations" => {
                    vector::<crate::identity::data::RelationId>(value, request, |_| {
                        Ok(OwnerAllocation::default())
                    })?
                }
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
pub(super) fn member(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        if variant == "Projected" {
            add(
                &mut bytes,
                shared_root::<crate::graph_member::ProjectedMemberCoordinate>()?,
            )?;
            add(
                &mut bytes,
                crate::graph_member::ProjectedMemberStorage::control_bytes()
                    .ok_or_else(malformed)?,
            )?;
            value.fields(request, |field, value| {
                add(
                    &mut bytes,
                    match field {
                        "domain" => named_identity(value, request)?,
                        "case" => text(value, request)?,
                        _ => OwnerAllocation::default(),
                    },
                )
            })?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
