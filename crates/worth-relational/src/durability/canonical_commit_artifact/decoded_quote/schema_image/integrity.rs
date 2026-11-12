//! Full relation-integrity declarations retain IDs, kind rosters and source names.
use super::*;
use crate::identity::data::KindId;
use crate::schema::data::*;

pub(super) fn quote(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut total = OwnerAllocation::default();
    value.fields(request, |name, raw| {
        let bytes = match name {
            "endpoint_kind_contracts" => {
                vector::<EndpointKindContractDeclaration>(raw, request, |row| {
                    declaration(row, request)
                })?
            }
            "cardinality_contracts" => {
                vector::<CardinalityContractDeclaration>(raw, request, |row| {
                    declaration(row, request)
                })?
            }
            "uniqueness_contracts" => {
                vector::<UniquenessContractDeclaration>(raw, request, |row| {
                    declaration(row, request)
                })?
            }
            "symmetry_contracts" => vector::<SymmetryContractDeclaration>(raw, request, |row| {
                declaration(row, request)
            })?,
            "endpoint_deletion_integrity_contracts" => {
                vector::<EndpointDeletionIntegrityDeclaration>(raw, request, |row| {
                    declaration(row, request)
                })?
            }
            "acyclicity_contracts" => {
                vector::<AcyclicityContractDeclaration>(raw, request, |row| {
                    declaration(row, request)
                })?
            }
            "partition_isolation_contracts" => {
                vector::<PartitionIsolationContractDeclaration>(raw, request, |row| {
                    declaration(row, request)
                })?
            }
            "connectivity_minimum_contracts" => {
                vector::<ConnectivityMinimumContractDeclaration>(raw, request, |row| {
                    declaration(row, request)
                })?
            }
            "aggregate_identity" => optional(raw, |row| aggregate(row, request))?,
            "plan_revision" => OwnerAllocation::default(),
            _ => return Err(malformed()),
        };
        add(&mut total, bytes)
    })?;
    Ok(total)
}
fn declaration(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut total = OwnerAllocation::default();
    value.fields(request, |name, raw| {
        match name {
            "contract_id" => add(&mut total, text(raw, request)?)?,
            "allowed_source_kinds"
            | "allowed_target_kinds"
            | "source_kind_ids"
            | "target_kind_ids" => add(
                &mut total,
                vector::<KindId>(raw, request, |_| Ok(OwnerAllocation::default()))?,
            )?,
            _ => {}
        }
        Ok(())
    })?;
    Ok(total)
}
fn aggregate(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut total = OwnerAllocation::default();
    value.fields(request, |name, raw| {
        if name != "source_relation" || !raw.is_map() {
            return Ok(());
        }
        raw.fields(request, |name, raw| {
            let bytes = match name {
                "relations" => vector::<KindId>(raw, request, |_| Ok(OwnerAllocation::default()))?,
                "derived_orbit" | "derived_components" => {
                    let mut bytes = OwnerAllocation::default();
                    raw.fields(request, |name, raw| {
                        if matches!(name, "schema" | "name") {
                            add(&mut bytes, text(raw, request)?)?;
                        }
                        Ok(())
                    })?;
                    bytes
                }
                _ => return Err(malformed()),
            };
            add(&mut total, bytes)
        })
    })?;
    Ok(total)
}
