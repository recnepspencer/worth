//! Derive native continuity meaning from proven physical canonical field changes.
use crate::runtime::{
    CodecFieldLocus, RelationalSchemaTransitionAdmissionDenial as Denial,
    RelationalSchemaTransitionAdmissionDenialKind as Kind,
};
use crate::schema::data::*;

pub(super) fn derive_codec_transition(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &RelationalSchemaRegistry,
    target: &RelationalSchemaRegistry,
    changes: &[CodecFieldLocus],
) -> Result<
    (
        ProposedSchemaTransition,
        worth_execution::ExecutionMemoryReservation,
    ),
    Denial,
> {
    let result = execution
        .in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    let mut bytes = std::mem::size_of::<super::RecoveredCodecPublicationCustody>()
                        + 2 * std::mem::size_of::<usize>();
                    for registry in [source, target] {
                        // Includes the two top-level basis copies and the source-owned
                        // target schema identifier copied into each diff atom.
                        let mut maximum = 0_usize;
                        for (id, _) in registry
                            .entity_kinds
                            .values()
                            .map(|r| (&r.schema_id, r.schema_version_id))
                            .chain(
                                registry
                                    .relation_kinds
                                    .values()
                                    .map(|r| (&r.schema_id, r.schema_version_id)),
                            )
                        {
                            context
                                .checkpoint(
                                    u64::try_from(id.0.len().saturating_add(1))
                                        .map_err(|_| overflow())?,
                                )
                                .map_err(|stop| Denial::request(stop.into()))?;
                            maximum = maximum.max(id.0.len());
                        }
                        bytes = bytes
                            .checked_add(
                                maximum
                                    .checked_mul(changes.len().checked_add(1).ok_or_else(overflow)?)
                                    .ok_or_else(overflow)?,
                            )
                            .ok_or_else(overflow)?;
                    }
                    for change in changes {
                        let projection = change
                            .aspect_key
                            .as_str()
                            .len()
                            .checked_add(change.field_key.as_str().len())
                            .and_then(|n| n.checked_add(1))
                            .ok_or_else(overflow)?;
                        context
                            .checkpoint(u64::try_from(projection).map_err(|_| overflow())?)
                            .map_err(|stop| Denial::request(stop.into()))?;
                        // String-to-Arc conversion briefly owns both text allocations.
                        bytes = bytes
                            .checked_add(std::mem::size_of::<SchemaDiffAtom>())
                            .and_then(|n| n.checked_add(2 * std::mem::size_of::<SchemaStratum>()))
                            .and_then(|n| n.checked_add(2 * std::mem::size_of::<usize>()))
                            .and_then(|n| {
                                projection
                                    .checked_mul(2)
                                    .and_then(|text| n.checked_add(text))
                            })
                            .ok_or_else(overflow)?;
                    }
                    let memory = worth_execution::ExecutionMemoryReservation::reserve_in_scope(
                        lease,
                        u64::try_from(bytes).map_err(|_| overflow())?,
                    )
                    .map_err(|stop| {
                        Denial::request(crate::execution::denial::lease_denial(stop))
                    })?;
                    build_transition(source, target, changes).map(|transition| (transition, memory))
                })
        })
        .map_err(|stop| Denial::request(stop.into()))
        .and_then(|result| result.map_err(|stop| Denial::request(stop.into())))
        .and_then(|result| result.0);
    crate::execution::check_request(execution).map_err(Denial::request)?;
    result
}

fn overflow() -> Denial {
    Denial::request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
}

fn build_transition(
    source: &RelationalSchemaRegistry,
    target: &RelationalSchemaRegistry,
    changes: &[CodecFieldLocus],
) -> Result<ProposedSchemaTransition, Denial> {
    let (source_schema_id, source_schema_version_id) = source
        .authoritative_schema_basis()
        .map_err(|error| Denial::new(Kind::SourceBasisMismatch, error.detail))?
        .ok_or_else(|| {
            Denial::new(
                Kind::SourceBasisMismatch,
                "codec source has no authoritative schema basis",
            )
        })?;
    let (target_schema_id, target_schema_version_id) = target
        .authoritative_schema_basis()
        .map_err(|error| Denial::new(Kind::TargetBasisMismatch, error.detail))?
        .ok_or_else(|| {
            Denial::new(
                Kind::TargetBasisMismatch,
                "codec target has no authoritative schema basis",
            )
        })?;
    let mut diff_atoms = Vec::with_capacity(changes.len());
    for change in changes {
        let mut projection = String::with_capacity(
            change.aspect_key.as_str().len() + 1 + change.field_key.as_str().len(),
        );
        projection.push_str(change.aspect_key.as_str());
        projection.push('.');
        projection.push_str(change.field_key.as_str());
        let projection: std::sync::Arc<str> = projection.into();
        diff_atoms.push(SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                target_schema_id.clone(),
                target_schema_version_id,
                Some(change.kind_id),
                std::sync::Arc::clone(&projection),
            ),
            vec![
                SchemaStratum::ValueDomain,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::PatchEncodingChanged,
            SchemaSubscriberImpact::ContractUpgradeRequired,
            HistoricalInterpretationSensitivity::SensitiveToDerivedMeaning,
            SchemaDiffDetail::ProjectionContractChanged {
                projection_name: projection,
            },
        ));
    }
    Ok(ProposedSchemaTransition {
        source_schema_id,
        source_schema_version_id,
        target_schema_id,
        target_schema_version_id,
        diff_atoms,
    })
}
