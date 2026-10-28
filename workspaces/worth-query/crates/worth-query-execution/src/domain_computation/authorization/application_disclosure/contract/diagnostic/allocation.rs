//! Quote the existing compiler's rule, mask, key and simultaneous tree copies.
use super::*;
use crate::domain_computation::primary_graph::admitted_capture::storage::tree_clone_bytes;
use std::{alloc::Layout, sync::atomic::AtomicUsize};
use worth_execution::MapKernelContext;
use worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence;

type Failure = MapKernelFailure<DiagnosticContractDenial>;

pub(super) fn backing_bytes<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<u64, Failure> {
    let overflow = || {
        Failure::Domain(DiagnosticContractDenial::Storage(
            RetainedStorageDenial::Overflow,
        ))
    };
    let contract = query.disclosure();
    let count = contract.rules().len();
    let result_trees =
        tree_clone_bytes::<(ApplicationQueryResultSlotKey, AdmittedDisclosureRule)>(count)
            .and_then(|bytes| bytes.checked_mul(2))
            .ok_or_else(overflow)?;
    let internal_tree =
        tree_clone_bytes::<((String, String, String), Vec<AdmittedDisclosureRule>)>(count)
            .ok_or_else(overflow)?;
    let mut bytes = result_trees
        .checked_add(internal_tree)
        .ok_or_else(overflow)?;
    let text = contract
        .classification()
        .len()
        .checked_add(contract.capability_name().map_or(0, str::len))
        .and_then(|n| n.checked_add(contract.capability_type().map_or(0, str::len)))
        .ok_or_else(overflow)?;
    bytes = add(bytes, text)?;
    bytes = add(bytes, query.name().len())?;
    // The shared compiler reserves the exact full internal roster and exact
    // field-group rosters. Each internal rule occupies one pointer in each.
    let mut internal_count = 0usize;
    for rule in contract.rules() {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        if matches!(
            rule.selector(),
            ApplicationQueryDisclosureSelector::InternalField { .. }
        ) {
            internal_count += 1;
        }
    }
    bytes = add(
        bytes,
        Layout::array::<AdmittedDisclosureRule>(
            internal_count.checked_mul(2).ok_or_else(overflow)?,
        )
        .map_err(|_| overflow())?
        .size(),
    )?;
    // The same compiler's single grouping pass borrows immutable declaration
    // names while its exact-capacity compiled groups are built.
    bytes = bytes
        .checked_add(
            tree_clone_bytes::<((&str, &str, &str), usize)>(internal_count).ok_or_else(overflow)?,
        )
        .ok_or_else(overflow)?;
    // Even a missing rule must pay for the real validator's temporary lookup
    // key before it reports the original contract failure.
    for field in query
        .read_graph()
        .predicates()
        .iter()
        .map(|field| field.field())
        .chain(
            query
                .read_graph()
                .ordering()
                .iter()
                .map(|field| field.field()),
        )
    {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        bytes = add(
            bytes,
            field
                .0
                .len()
                .checked_add(field.1.len())
                .and_then(|n| n.checked_add(field.2.len()))
                .ok_or_else(overflow)?,
        )?;
    }
    for rule in contract.rules() {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        let selector = rule.selector();
        let slot = match selector {
            ApplicationQueryDisclosureSelector::Field {
                query_type,
                slot_type,
                entity,
                aspect,
                field,
                output_name,
                value_type,
                ..
            } => strings([entity, aspect, field, output_name])?
                .checked_add(query_type.retained_heap_bytes())
                .and_then(|n| n.checked_add(slot_type.retained_heap_bytes()))
                .and_then(|n| n.checked_add(value_type.retained_heap_bytes()))
                .ok_or_else(overflow)?,
            ApplicationQueryDisclosureSelector::Relation {
                query_type,
                slot_type,
                relation,
                from,
                to,
                output_name,
                ..
            } => strings([relation, from, to, output_name])?
                .checked_add(query_type.retained_heap_bytes())
                .and_then(|n| n.checked_add(slot_type.retained_heap_bytes()))
                .ok_or_else(overflow)?,
            ApplicationQueryDisclosureSelector::InternalField { .. } => 0,
            _ => return Err(Failure::Domain(DiagnosticContractDenial::UnsupportedShape)),
        };
        bytes = add(bytes, slot)?;
        let header = Layout::new::<[AtomicUsize; 2]>()
            .extend(Layout::new::<WorthQueryAdmittedApplicationDisclosureRule>())
            .map_err(|_| overflow())?
            .0
            .pad_to_align()
            .size();
        bytes = add(bytes, header)?;
        bytes = add(
            bytes,
            rule.disclosure_value().owned_allocation_capacity_bytes(),
        )?;
        bytes = bytes
            .checked_add(
                tree_clone_bytes::<ApplicationQueryObservableInfluence>(
                    rule.influence().permitted().len(),
                )
                .ok_or_else(overflow)?,
            )
            .ok_or_else(overflow)?;
        if let Some((entity, aspect, field)) = selector.field_contract() {
            let names = entity
                .len()
                .checked_add(aspect.len())
                .and_then(|n| n.checked_add(field.len()))
                .ok_or_else(overflow)?;
            // The compiled field, internal map key and temporary validation key
            // have separate backing; the temporary aspect lookup also fits here.
            bytes = add(bytes, names.checked_mul(4).ok_or_else(overflow)?)?;
            bytes = add(
                bytes,
                selector
                    .projection_mask()
                    .ok_or_else(overflow)?
                    .owned_allocation_capacity_bytes(),
            )?;
            bytes = add(
                bytes,
                selector
                    .diagnostic_mask()
                    .ok_or_else(overflow)?
                    .owned_allocation_capacity_bytes(),
            )?;
        }
    }
    // The actual influence validator reserves a Vec of the closed vocabulary
    // and may hold its membership and ordering sets while producing surfaces.
    let vocabulary = ApplicationQueryObservableInfluence::ALL.len();
    let cone_bytes = tree_clone_bytes::<ApplicationQueryObservableInfluence>(vocabulary)
        .and_then(|bytes| bytes.checked_mul(2))
        .ok_or_else(overflow)?;
    bytes = bytes.checked_add(cone_bytes).ok_or_else(overflow)?;
    add(
        bytes,
        Layout::array::<ApplicationQueryObservableInfluence>(vocabulary)
            .map_err(|_| overflow())?
            .size(),
    )
}

fn strings<const N: usize>(values: [&String; N]) -> Result<usize, Failure> {
    values
        .into_iter()
        .try_fold(0usize, |sum, value| sum.checked_add(value.capacity()))
        .ok_or(Failure::Domain(DiagnosticContractDenial::Storage(
            RetainedStorageDenial::Overflow,
        )))
}

fn add(bytes: u64, extra: usize) -> Result<u64, Failure> {
    u64::try_from(extra)
        .ok()
        .and_then(|extra| bytes.checked_add(extra))
        .ok_or(Failure::Domain(DiagnosticContractDenial::Storage(
            RetainedStorageDenial::Overflow,
        )))
}
