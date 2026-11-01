//! Concrete schema image owners in the canonical authoritative projection.
use super::*;
mod admission;
use crate::schema::data::{
    SchemaAuthorityKindSnapshot, SchemaAuthorityRelationSnapshot, SchemaChangeText, SchemaDiffAtom,
    SchemaId, SchemaStratum, SchemaVersionId,
};
pub(in super::super) use admission::prepare_canonical_readmission;
#[cfg(test)]
pub(in super::super) use admission::prepare_readmission;

pub(super) fn authority(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "primary_schema_id" => optional(value, |value| text(value, request))?,
                "entity_kinds" => vector::<SchemaAuthorityKindSnapshot>(value, request, |value| {
                    kind(value, request)
                })?,
                "relation_kinds" => {
                    vector::<SchemaAuthorityRelationSnapshot>(value, request, |value| {
                        kind(value, request)
                    })?
                }
                "derived_relations" => vector::<
                    crate::derived_relation::DerivedRelationDefinitionParts,
                >(value, request, |value| {
                    derived(value, request)
                })?,
                "projected_member_domains" => vector::<
                    crate::projected_member_domain::ProjectedMemberDomainParts,
                >(value, request, |value| {
                    projected(value, request)
                })?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn kind(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if matches!(field, "schema_id" | "kind_name") {
            add(&mut bytes, text(value, request)?)?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn identity(
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
fn domain(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    if !value.is_map() {
        return Ok(OwnerAllocation::default());
    }
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "projected" {
            add(&mut bytes, identity(value, request)?)?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn projected(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "identity" => identity(value, request)?,
                "cases" => vector::<String>(value, request, |value| text(value, request))?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn derived(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "identity" => identity(value, request)?,
                "from" | "to" => domain(value, request)?,
                "nodes" => vector::<crate::derived_relation::DerivedRelationNode>(
                    value,
                    request,
                    |value| derived_node(value, request),
                )?,
                "laws" => optional(value, |value| laws(value, request))?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn derived_node(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        if variant == "Derived" {
            add(&mut bytes, identity(value, request)?)?;
        }
        if variant == "CarrierLift" {
            value.fields(request, |field, value| {
                add(
                    &mut bytes,
                    match field {
                        "domain" => identity(value, request)?,
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
fn laws(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "reaches_at_least" {
            add(
                &mut bytes,
                optional(value, |value| {
                    let mut bytes = OwnerAllocation::default();
                    value.fields(request, |field, value| {
                        if field == "target_domains" {
                            add(
                                &mut bytes,
                                vector::<crate::graph_member::NativeMemberDomain>(
                                    value,
                                    request,
                                    |value| domain(value, request),
                                )?,
                            )?;
                        }
                        Ok(())
                    })?;
                    Ok(bytes)
                })?,
            )?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
pub(super) fn transition(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "source_schema_id" | "target_schema_id" => text(value, request)?,
                "diff_atoms" => {
                    vector::<SchemaDiffAtom>(value, request, |value| diff_atom(value, request))?
                }
                "continuation_descriptor" => continuation(value, request)?,
                "reconciliation_descriptor" => reconciliation(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
pub(super) fn continuation(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "bridge" {
            value.fields(request, |field, value| {
                if field == "changed_strata" {
                    add(
                        &mut bytes,
                        vector::<SchemaStratum>(
                            value,
                            request,
                            |_| Ok(OwnerAllocation::default()),
                        )?,
                    )?;
                }
                Ok(())
            })?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
pub(super) fn reconciliation(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "resulting_lineage" {
            value.fields(request, |field, value| {
                add(
                    &mut bytes,
                    match field {
                        "resulting_schema_id" => text(value, request)?,
                        "parent_schema_ids" => {
                            vector::<SchemaId>(value, request, |value| text(value, request))?
                        }
                        "parent_schema_version_ids" => {
                            vector::<SchemaVersionId>(value, request, |_| {
                                Ok(OwnerAllocation::default())
                            })?
                        }
                        "branch_context" => optional(value, |value| text(value, request))?,
                        _ => OwnerAllocation::default(),
                    },
                )
            })?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn diff_atom(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "strata" => {
                    vector::<SchemaStratum>(value, request, |_| Ok(OwnerAllocation::default()))?
                }
                "element" => {
                    let mut bytes = OwnerAllocation::default();
                    value.fields(request, |field, value| {
                        add(
                            &mut bytes,
                            match field {
                                "schema_id" => text(value, request)?,
                                "element_name" => change_text(value, request)?,
                                _ => OwnerAllocation::default(),
                            },
                        )
                    })?;
                    bytes
                }
                "detail" => diff_detail(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn diff_detail(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |_, value| {
        value.fields(request, |field, value| {
            add(
                &mut bytes,
                match field {
                    "field" => text(value, request)?,
                    "default_expression" => optional(value, |value| change_text(value, request))?,
                    "from_type" | "to_type" | "contract_name" | "projection_name" | "detail" => {
                        change_text(value, request)?
                    }
                    "added_variants" => vector::<SchemaChangeText>(value, request, |value| {
                        change_text(value, request)
                    })?,
                    _ => OwnerAllocation::default(),
                },
            )
        })
    })?;
    Ok(bytes)
}

fn change_text(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = shared_text(value, request)?;
    add(
        &mut bytes,
        SchemaChangeText::control_bytes().ok_or_else(malformed)?,
    )?;
    Ok(bytes)
}
