//! Mint only after joining the binding's original Native artifact to World.
use super::original_receipt::{
    MemoryOriginalReceiptInputs, OriginalReceiptInputs, OriginalReceiptRows,
    WorthQueryRecoveredAftermathCausality,
};
use super::*;
use crate::domain_computation::primary_graph::provider::OriginalPublication;
use worth_relational::facade::{history::RelationalCommitReceipt, identity::EntityId};
type Denial = WorthQueryProviderIdempotencyResolutionDenial;

pub(super) fn read_original(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    context: &WorthQueryIdempotencySnapshotContext<'_>,
    product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    entity: EntityId,
    commit: &RelationalCommitReceipt,
) -> Result<WorthQueryRecoveredApplicationCommitReceipt, Denial> {
    let publication = read_publication(execution, context.runtime, product, commit)?;
    let expected = product.observation();
    if !publication.matches_affinity(expected) {
        return Err(Denial::ForeignAffinity);
    }
    let owner = publication.owner_coordinate();
    let mut bytes = WorthQueryRecoveredApplicationCommitReceipt::retained_backing_bytes() as u64;
    // Each parser first copies projected values, then retains decoded meaning.
    // Both allocations fit within two complete creation-row backing copies.
    for created in publication.semantic_result().created_entities() {
        if [
            context.layout.entity_kind,
            context
                .provider
                .graph
                .layout
                .provider_dispatch_outbox()
                .entity_kind,
            context
                .provider
                .graph
                .layout
                .provider_aftermath_causality()
                .entity_kind,
        ]
        .contains(&created.key.kind)
        {
            let row = publication
                .created_entity(execution, created.assigned)
                .map_err(super::source_denial::native)?
                .ok_or(Denial::Unavailable)?;
            bytes = bytes
                .checked_add(
                    row.owned_allocation_capacity_bytes()
                        .checked_mul(2)
                        .ok_or(Denial::Unavailable)?,
                )
                .ok_or(Denial::Unavailable)?;
        }
    }
    bytes = bytes
        .checked_add(publication.branch_name_bytes() as u64)
        .and_then(|bytes| bytes.checked_add(2 * std::mem::size_of::<usize>() as u64))
        .ok_or(Denial::Unavailable)?;
    let funding = execution
        .reserve_retained_memory(bytes)
        .map_err(|cause| super::idempotency_read_stop(cause.into()))?;
    // Charge the admitted reconstructive copy bound before cloning payloads.
    let mut copy_charge = None;
    let charged = execution.run_native_with_checkpoint(|context| {
        copy_charge = Some(if context.has_accounting() {
            context.checkpoint(bytes)
        } else {
            context.poll()
        });
    });
    match (charged, copy_charge) {
        (Ok(_), Some(Ok(()))) => {}
        (_, Some(Err(cause))) => return Err(super::idempotency_read_stop(cause.into())),
        (Err(cause), _) => return Err(super::idempotency_read_stop(cause.into())),
        (Ok(_), None) => unreachable!("the original request charged its copy"),
    }
    let row = publication
        .created_entity(execution, entity)
        .map_err(super::source_denial::native)?
        .ok_or(Denial::Unavailable)?;
    let binding = super::original_binding::read(&row, context.layout)?;
    let outbox = context
        .provider
        .original_dispatch_outbox(execution, &publication)
        .map_err(|cause| {
            if let super::super::super::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxReadDenial::ReadExecution(cause) = cause {
                Denial::ExecutionDenied(cause)
            } else { Denial::Unavailable }
        })?;
    if outbox
        .as_ref()
        .is_some_and(|(_, row)| row.outcome_identity() != binding.outcome.get())
    {
        return Err(Denial::Unavailable);
    }
    match binding.settlement {
        worth_foundational::facade::DurableQueryOutputSettlement::Absent => {}
        worth_foundational::facade::DurableQueryOutputSettlement::OriginalOccurrenceSlot {
            slot,
        } => {
            if slot != 0 {
                return Err(Denial::Unavailable);
            }
        }
    }
    let causality = context
        .provider
        .original_aftermath_causality(execution, context.runtime, &publication, owner)
        .map_err(|cause| {
            if let super::super::super::aftermath_causality::WorthQueryAftermathCausalityReadDenial::ExecutionDenied(cause) = cause {
                Denial::ExecutionDenied(cause)
            } else { Denial::Unavailable }
        })?;
    if let Some(fact) = &causality {
        for coordinate in [
            Some(&fact.source),
            Some(&fact.root),
            fact.selected.as_ref(),
            fact.active.as_ref(),
            fact.prior.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            if coordinate.world_owner() != owner {
                return Err(Denial::ForeignAffinity);
            }
            let source =
                read_publication(execution, context.runtime, product, coordinate.relational())?;
            if source.world_commit() != coordinate.world_commit() {
                return Err(Denial::Unavailable);
            }
        }
    }
    let port = context.runtime.owner_component_services().basis_port();
    let (before_component, after_component) = publication
        .recorded_components(execution, &port)
        .map_err(super::source_denial::component)?;
    let rows = OriginalReceiptRows {
        binding,
        before_component,
        after_component,
        outbox,
        causality: causality.map(WorthQueryRecoveredAftermathCausality::from_source_fact),
        funding,
    };
    match publication {
        OriginalPublication::Home(publication) => {
            WorthQueryRecoveredApplicationCommitReceipt::from_readmitted(
                worth_proof::QueryReadmitted::from_owner(OriginalReceiptInputs {
                    publication,
                    rows,
                }),
            )
        }
        OriginalPublication::Memory(publication) => {
            WorthQueryRecoveredApplicationCommitReceipt::from_memory(
                worth_proof::QueryReadmitted::from_owner(MemoryOriginalReceiptInputs {
                    publication,
                    rows,
                }),
            )
        }
    }
}

fn read_publication(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    commit: &RelationalCommitReceipt,
) -> Result<OriginalPublication, Denial> {
    let authority = runtime.durability_authority();
    if product
        .observation()
        .branch_identity()
        .owner_identity()
        .durable()
        .is_none()
    {
        let original = authority
            .observe_memory_commit(execution, commit)
            .map_err(super::source_denial::native)?;
        return product
            .observation_port()
            .observe_memory_original_publication(execution, original)
            .map(OriginalPublication::Memory)
            .map_err(super::source_denial::world);
    }

    let mut exact = None;
    for subject in authority
        .discover_all_associated_effects(execution)
        .map_err(super::source_denial::native)?
    {
        if subject.receipt() == commit {
            if exact.is_some() {
                return Err(Denial::Unavailable);
            }
            exact = Some(subject);
        }
    }
    let effect = authority
        .readmit_associated_effect(execution, &exact.ok_or(Denial::Unavailable)?)
        .map_err(super::source_denial::native)?;
    product
        .observation_port()
        .readmitted_native_publication(execution, effect)
        .map(OriginalPublication::Home)
        .map_err(super::source_denial::world)
}
