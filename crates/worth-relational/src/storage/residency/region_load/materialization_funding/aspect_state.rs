//! Portable aspect and aspected-record readmission funding.
use super::*;
use crate::durability::data::*;
use crate::storage::substrate::*;
use worth_foundational::facade::{
    AspectKey, AspectValue, ContractValidatedAspectValue, FieldKey, PortableRecordAspectState,
    PortableRecordAspectStateEntry, StructAspectValue,
};

fn state_roots(count: usize, contract: u64) -> Result<Quote> {
    // Candidate entries, validation Vec, admission tree and final boxed values
    // can coexist. Nested keys/values and cloned contracts are separate below.
    let wire = sum(
        vector::<PortableRecordAspectStateEntry>(count)?,
        sum(
            vector::<ContractValidatedAspectValue>(count)?,
            tree::<AspectKey, ContractValidatedAspectValue>(count)?,
        )?,
    )?;
    let native = sum(
        array::<ContractValidatedAspectValue>(count)?,
        times(contract, count)?,
    )?;
    Ok(Quote::bytes(sum(wire, times(contract, count)?)?, native))
}
pub(in crate::storage::residency) fn wire(
    raw: Raw<'_>,
    contract: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Quote> {
    if raw.is_none() {
        return Ok(Quote::default());
    }
    let mut quote = Quote::default();
    record(raw, &["entries"], request, |_, entries| {
        let count = sequence(entries, request, |entry| {
            record(entry, &["basis", "value"], request, |index, value| {
                if index == 0 {
                    record(
                        value,
                        &["key", "identity", "revision"],
                        request,
                        |index, value| {
                            if index == 0 {
                                let key = text(value, request)?;
                                quote.add(Quote::bytes(times(key, 2)?, key))?;
                            }
                            Ok(())
                        },
                    )
                } else {
                    quote.add(input(value, request)?)
                }
            })
        })?;
        quote.add(state_roots(count, contract)?)
    })?;
    Ok(quote)
}
fn input(raw: Raw<'_>, request: &ExecutionRequest<'_, '_>) -> Result<Quote> {
    let mut quote = Quote::default();
    raw.fields(request, |variant, value| {
        quote
            .add(match variant {
                "Scalar" => scalar(value, request).map_err(to_wire_stop)?,
                "Struct" => structure(value, request).map_err(to_wire_stop)?,
                _ => {
                    return Err(
                        crate::durability::canonical_commit_artifact::borrowed_payload::malformed(),
                    )
                }
            })
            .map_err(to_wire_stop)
    })
    .map_err(wire_stop)?;
    Ok(quote)
}
fn scalar(raw: Raw<'_>, request: &ExecutionRequest<'_, '_>) -> Result<Quote> {
    if matches!(raw.0.first(), Some(0xa0..=0xbf | 0xd9..=0xdb)) {
        return Ok(Quote::default());
    }
    let mut quote = Quote::default();
    raw.fields(request, |variant, value| {
        let mut bytes = 0;
        match variant {
            "Decimal" | "BigInt" => bytes = text(value, request).map_err(to_wire_stop)?,
            "Rational" => record(value, &["numerator", "denominator"], request, |_, part| {
                bytes = sum(bytes, text(part, request)?)?;
                Ok(())
            })
            .map_err(to_wire_stop)?,
            "String" => value.fields(request, |kind, value| {
                if kind == "Raw" {
                    bytes = text(value, request).map_err(to_wire_stop)?;
                }
                Ok(())
            })?,
            "InlineBytes" => {
                let n = sequence(value, request, |_| Ok(())).map_err(to_wire_stop)?;
                bytes = vector::<u8>(n).map_err(to_wire_stop)?;
            }
            "Null" | "Bool" | "Int8" | "Int16" | "Int32" | "Int64" | "UInt8" | "UInt16"
            | "UInt32" | "UInt64" | "Float32" | "Float64" | "Uuid" | "Date" | "Time"
            | "Timestamp" | "TimestampTz" | "EntityRef" | "ContentRef" => {}
            _ => {
                return Err(
                    crate::durability::canonical_commit_artifact::borrowed_payload::malformed(),
                )
            }
        }
        quote.add(Quote::bytes(bytes, bytes)).map_err(to_wire_stop)
    })
    .map_err(wire_stop)?;
    Ok(quote)
}
fn structure(raw: Raw<'_>, request: &ExecutionRequest<'_, '_>) -> Result<Quote> {
    let mut quote = Quote::default();
    record(raw, &["fields"], request, |_, fields| {
        let count = pairs(fields, request, |key, value| {
            let key = text(key, request)?;
            quote.add(Quote::bytes(key, key))?;
            quote.add(scalar(value, request)?)
        })?;
        let bounds = StructAspectValue::decode_storage_bounds(count).ok_or_else(overflow)?;
        quote.wire = sum(
            quote.wire,
            sum(
                bounds.retained_fields_bytes as u64,
                bounds.transient_decode_bytes as u64,
            )?,
        )?;
        quote.native = sum(quote.native, array::<(FieldKey, AspectValue)>(count)?)?;
        Ok(())
    })?;
    Ok(quote)
}
pub(in crate::storage::residency) fn owned(
    state: Option<&PortableRecordAspectState>,
    contract: u64,
) -> Result<Quote> {
    let Some(state) = state else {
        return Ok(Quote::default());
    };
    let mut quote = state_roots(state.entries().len(), contract)?;
    for entry in state.entries() {
        let mut bytes = entry.basis().owned_allocation_capacity_bytes() as u64;
        bytes = sum(
            bytes,
            entry.value().owned_allocation_capacity_bytes() as u64,
        )?;
        // Owned clones need no Serde growth; readmission roots are separate.
        quote.add(Quote::bytes(bytes, bytes))?;
    }
    Ok(quote)
}

pub(in crate::storage::residency) fn metadata(
    raw: Raw<'_>,
    entity: bool,
    contract: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Quote> {
    let mut quote = Quote::default();
    record(
        raw,
        if entity {
            &[
                "effective_at",
                "retired_at",
                "generation",
                "kind_id",
                "lineage_id",
                "authoritative_aspect_state",
            ]
        } else {
            &[
                "effective_at",
                "retired_at",
                "generation",
                "kind_id",
                "endpoints",
                "authoritative_aspect_state",
            ]
        },
        request,
        |i, value| {
            if i == 5 {
                quote.add(wire(value, contract, request)?)?;
            }
            Ok(())
        },
    )?;
    quote.native = sum(
        quote.native,
        if entity {
            shared::<EntityExtra>()?
        } else {
            shared::<RelationExtra>()?
        },
    )?;
    Ok(quote)
}

pub(in crate::storage::residency) fn histories(
    raw: Raw<'_>,
    entity: bool,
    contract: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Quote> {
    let mut quote = Quote::default();
    let count = sequence(raw, request, |rows| {
        let count = sequence(rows, request, |meta| {
            quote.add(metadata(meta, entity, contract, request)?)
        })?;
        quote.add(if entity {
            Quote::bytes(
                vector::<VersionedEntityMetadataImage>(count)?,
                column::<StorageAllocation<VersionedEntityMetadata>>(count)?,
            )
        } else {
            Quote::bytes(
                vector::<VersionedRelationMetadataImage>(count)?,
                column::<StorageAllocation<VersionedRelationMetadata>>(count)?,
            )
        })
    })?;
    quote.add(if entity {
        Quote::bytes(
            vector::<Vec<VersionedEntityMetadataImage>>(count)?,
            column::<SharedColumn<StorageAllocation<VersionedEntityMetadata>>>(count)?,
        )
    } else {
        Quote::bytes(
            vector::<Vec<VersionedRelationMetadataImage>>(count)?,
            column::<SharedColumn<StorageAllocation<VersionedRelationMetadata>>>(count)?,
        )
    })?;
    Ok(quote)
}

pub(in crate::storage::residency) fn extra(
    raw: Raw<'_>,
    entity: bool,
    contract: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Quote> {
    let mut quote = Quote::default();
    let count = sequence(raw, request, |value| {
        let fields: &[&str] = if entity {
            &[
                "structural_fingerprint",
                "lineage_id",
                "authoritative_aspect_state",
            ]
        } else {
            &["endpoints", "authoritative_aspect_state"]
        };
        record(value, fields, request, |i, value| {
            if i == fields.len() - 1 {
                quote.add(wire(value, contract, request)?)?;
            }
            Ok(())
        })
    })?;
    quote.add(if entity {
        Quote::bytes(
            vector::<EntityExtraImage>(count)?,
            column::<StorageAllocation<EntityExtra>>(count)?,
        )
    } else {
        Quote::bytes(
            vector::<RelationExtraImage>(count)?,
            column::<StorageAllocation<RelationExtra>>(count)?,
        )
    })?;
    Ok(quote)
}

pub(in crate::storage::residency) fn histories_element(
    raw: Raw<'_>,
    entity: bool,
    contract: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Quote> {
    let mut quote = Quote::default();
    let count = sequence(raw, request, |row| {
        quote.add(metadata(row, entity, contract, request)?)
    })?;
    quote.wire = sum(
        quote.wire,
        if entity {
            vector::<VersionedEntityMetadataImage>(count)?
        } else {
            vector::<VersionedRelationMetadataImage>(count)?
        },
    )?;
    Ok(quote)
}

pub(in crate::storage::residency) fn entity_metadata(
    value: &VersionedEntityMetadataImage,
    contract: u64,
) -> Result<Quote> {
    let mut q = owned(value.authoritative_aspect_state.as_ref(), contract)?;
    q.native = sum(q.native, shared::<EntityExtra>()?)?;
    Ok(q)
}

pub(in crate::storage::residency) fn relation_metadata(
    value: &VersionedRelationMetadataImage,
    contract: u64,
) -> Result<Quote> {
    let mut q = owned(value.authoritative_aspect_state.as_ref(), contract)?;
    q.native = sum(q.native, shared::<RelationExtra>()?)?;
    Ok(q)
}

pub(in crate::storage::residency) fn contracts(
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(Funding, u64)> {
    use worth_foundational::facade::{
        AspectContract, AspectContractRevision, AspectIdentity, AspectKey, PortableAspectContract,
    };
    let mut bytes = vector::<PortableAspectContract>(schema.retained_aspect_contracts().len())?;
    bytes = sum(
        bytes,
        tree::<(AspectKey, AspectIdentity, AspectContractRevision), AspectContract>(
            schema.retained_aspect_contracts().len(),
        )?,
    )?;
    let mut largest = 0;
    for contract in schema.retained_aspect_contracts() {
        work(request, 1)?;
        let nested = crate::durability::checkpoints::aspect_state_images::contract_clone_bytes(
            contract, request,
        )
        .map_err(Failure::Request)?;
        largest = largest.max(nested);
        // Original-to-portable and portable-to-catalog copies coexist.
        bytes = sum(bytes, times(nested, 2)?)?;
    }
    work(request, bytes)?;
    Ok((Funding::reserve(bytes, request)?, largest))
}

pub(in crate::storage::residency) fn metadata_length(
    value: &impl serde::Serialize,
    request: &ExecutionRequest<'_, '_>,
) -> Result<u64> {
    struct Counter<'a, 'b, 'c> {
        length: u64,
        request: &'a ExecutionRequest<'b, 'c>,
        stop: Option<Failure>,
    }
    impl std::io::Write for Counter<'_, '_, '_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            match super::work(self.request, bytes.len() as u64)
                .and_then(|()| super::sum(self.length, bytes.len() as u64))
            {
                Ok(length) => {
                    self.length = length;
                    Ok(bytes.len())
                }
                Err(cause) => {
                    self.stop = Some(cause);
                    Err(std::io::Error::other("materialization accounting stopped"))
                }
            }
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter {
        length: 0,
        request,
        stop: None,
    };
    let result = rmp_serde::encode::write(&mut counter, value);
    if let Some(cause) = counter.stop {
        return Err(cause);
    }
    result.map_err(|_| super::damaged())?;
    Ok(counter.length)
}
