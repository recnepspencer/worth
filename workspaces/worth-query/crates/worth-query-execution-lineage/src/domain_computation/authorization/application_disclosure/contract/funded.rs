//! Funded disclosure contract compilation for governed one-shot admission.
//! Every compiler visit, copy, lookup, group intersection, and backing buffer
//! is admitted against the caller's `ExecutionRequest` before allocation.
use super::*;
use crate::domain_computation::primary_graph::admitted_capture::{
    checkpoint, run_retained_stage, storage::tree_clone_bytes, CaptureStop, RetainedStorageDenial,
};
use std::{alloc::Layout, sync::atomic::AtomicUsize};
use worth_execution::{
    ExecutionRequest, LeaseDenial, MapKernelContext, MapKernelFailure, MapKernelStop, ScopeStop,
    WorkCeilingDenial,
};
use worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FundedDisclosureCompilationDenial {
    Contract(WorthQueryApplicationDisclosureContractDenial),
    Cancelled,
    DeadlineExceeded,
    WorkExhausted,
    MemoryExhausted,
}

enum StageError {
    Contract(WorthQueryApplicationDisclosureContractDenial),
    Memory,
}

type Failure = MapKernelFailure<StageError>;

/// Compiles a query's disclosure contract under the caller's `ExecutionRequest`,
/// charging work and reserving temporary/output compilation memory before
/// allocation. Returns the live `ExecutionMemoryReservation` alongside the
/// compiled contract so the caller keeps the backing accounted until drop.
pub fn compile_funded_disclosure_contract<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    layout: &WorthQueryPrimaryGraphLayout,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<
    (
        WorthQueryAdmittedApplicationDisclosureContract,
        Option<worth_execution::ExecutionMemoryReservation>,
    ),
    FundedDisclosureCompilationDenial,
> {
    let mut memory = None;
    let compiled = run_retained_stage(execution, &mut memory, |context, memory| {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        match query.disclosure().posture() {
            ApplicationQueryDisclosurePosture::Public if query.disclosure().rules().is_empty() => {
                return Ok(WorthQueryAdmittedApplicationDisclosureContract::Public);
            }
            ApplicationQueryDisclosurePosture::Governed => {}
            _ => {
                return Err(MapKernelFailure::Domain(StageError::Contract(denial(
                    query.name(),
                ))));
            }
        }
        let bytes = backing_bytes(query, context)?;
        checkpoint(context, bytes).map_err(MapKernelFailure::Stop)?;
        if let Some(memory) = memory {
            memory
                .resize(bytes)
                .map_err(|_: RetainedStorageDenial| MapKernelFailure::Domain(StageError::Memory))?;
        }
        super::compilation::compile_with_admission(query, layout, &mut |operation| {
            admit_work(context, operation)
        })
        .map_err(|cause| match cause {
            super::admission::CompilationStop::Admission(cause) => cause,
            super::admission::CompilationStop::Contract(cause) => {
                MapKernelFailure::Domain(StageError::Contract(cause))
            }
        })
    });
    compiled
        .map(|contract| (contract, memory))
        .map_err(map_stop)
}

fn map_stop(stop: CaptureStop<StageError>) -> FundedDisclosureCompilationDenial {
    match stop {
        CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Domain(
            StageError::Contract(cause),
        ))) => FundedDisclosureCompilationDenial::Contract(cause),
        CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Domain(StageError::Memory)))
        | CaptureStop::Request(WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(_)))
        | CaptureStop::Kernel(ScopeStop::Admission(LeaseDenial::MemoryExhausted(_))) => {
            FundedDisclosureCompilationDenial::MemoryExhausted
        }
        CaptureStop::Request(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
        | CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Stop(
            MapKernelStop::Cancelled,
        ))) => FundedDisclosureCompilationDenial::Cancelled,
        CaptureStop::Request(WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed))
        | CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Stop(
            MapKernelStop::DeadlineElapsed,
        ))) => FundedDisclosureCompilationDenial::DeadlineExceeded,
        _ => FundedDisclosureCompilationDenial::WorkExhausted,
    }
}

fn admit_work(
    context: &mut MapKernelContext<'_, '_>,
    work: super::admission::CompilationWork<'_>,
) -> Result<(), Failure> {
    let overflow = || MapKernelFailure::Stop(MapKernelStop::WorkCounterOverflow);
    let units = match work {
        super::admission::CompilationWork::Visit => 1,
        super::admission::CompilationWork::Copy(bytes) => {
            u64::try_from(bytes).map_err(|_| overflow())?.max(1)
        }
        super::admission::CompilationWork::Scan { entries, parts }
        | super::admission::CompilationWork::Lookup { entries, parts } => {
            let width = parts
                .iter()
                .try_fold(1u64, |sum, text| {
                    sum.checked_add(u64::try_from(text.len()).ok()?.checked_add(1)?)
                })
                .ok_or_else(overflow)?;
            u64::try_from(entries)
                .ok()
                .and_then(|n| n.checked_add(1))
                .and_then(|n| n.checked_mul(width))
                .ok_or_else(overflow)?
        }
    };
    checkpoint(context, units).map_err(MapKernelFailure::Stop)
}

fn backing_bytes<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<u64, Failure> {
    let ovf = || MapKernelFailure::Domain(StageError::Memory);
    let contract = query.disclosure();
    let count = contract.rules().len();
    let mut bytes =
        tree_clone_bytes::<(ApplicationQueryResultSlotKey, AdmittedDisclosureRule)>(count)
            .and_then(|b| b.checked_mul(2))
            .ok_or_else(ovf)?;
    bytes = bytes
        .checked_add(
            tree_clone_bytes::<((String, String, String), Vec<AdmittedDisclosureRule>)>(count)
                .ok_or_else(ovf)?,
        )
        .and_then(|b| {
            b.checked_add(tree_clone_bytes::<(
                (String, String, String, RelationDirection),
                Vec<AdmittedDisclosureRule>,
            )>(count)?)
        })
        .ok_or_else(ovf)?;
    let mut internal_fields = 0usize;
    let mut internal_rels = 0usize;
    for rule in contract.rules() {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        match rule.selector() {
            ApplicationQueryDisclosureSelector::InternalField { .. } => internal_fields += 1,
            ApplicationQueryDisclosureSelector::InternalRelation { .. } => internal_rels += 1,
            _ => {}
        }
    }
    let total_internal = internal_fields.checked_add(internal_rels).ok_or_else(ovf)?;
    let vec_tables =
        Layout::array::<AdmittedDisclosureRule>(total_internal.checked_mul(2).ok_or_else(ovf)?)
            .map_err(|_| ovf())?
            .size()
            .checked_add(
                Layout::array::<((String, String, String), GovernedInternalRuleGroup)>(
                    internal_fields,
                )
                .map_err(|_| ovf())?
                .size(),
            )
            .and_then(|n| {
                n.checked_add(
                    Layout::array::<(
                        (String, String, String, RelationDirection),
                        GovernedInternalRuleGroup,
                    )>(internal_rels)
                    .ok()?
                    .size(),
                )
            })
            .ok_or_else(ovf)?;
    bytes = add(bytes, vec_tables)?;
    bytes = bytes
        .checked_add(
            tree_clone_bytes::<((&str, &str, &str), usize)>(internal_fields).ok_or_else(ovf)?,
        )
        .and_then(|b| {
            b.checked_add(tree_clone_bytes::<(
                (&str, &str, &str, RelationDirection),
                usize,
            )>(internal_rels)?)
        })
        .ok_or_else(ovf)?;
    let text = contract
        .classification()
        .len()
        .checked_add(contract.capability_name().map_or(0, str::len))
        .and_then(|n| n.checked_add(contract.capability_type().map_or(0, str::len)))
        .and_then(|n| n.checked_add(query.name().len()))
        .ok_or_else(ovf)?;
    bytes = add(bytes, text)?;
    let header = Layout::new::<[AtomicUsize; 2]>()
        .extend(Layout::new::<WorthQueryAdmittedApplicationDisclosureRule>())
        .map_err(|_| ovf())?
        .0
        .pad_to_align()
        .size();
    for rule in contract.rules() {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        bytes = add(bytes, result_slot_key_bytes(rule.selector())?)?;
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
                .ok_or_else(ovf)?,
            )
            .ok_or_else(ovf)?;
        if let Some((entity, aspect, field)) = rule.selector().field_contract() {
            let names = entity
                .len()
                .checked_add(aspect.len())
                .and_then(|n| n.checked_add(field.len()))
                .and_then(|n| n.checked_mul(4))
                .ok_or_else(ovf)?;
            bytes = add(bytes, names)?;
            if let Some(mask) = rule.selector().projection_mask() {
                bytes = add(bytes, mask.owned_allocation_capacity_bytes())?;
            }
            if let Some(mask) = rule.selector().diagnostic_mask() {
                bytes = add(bytes, mask.owned_allocation_capacity_bytes())?;
            }
        }
        if let ApplicationQueryDisclosureSelector::InternalRelation {
            relation, from, to, ..
        } = rule.selector()
        {
            let names = relation
                .len()
                .checked_add(from.len())
                .and_then(|n| n.checked_add(to.len()))
                .and_then(|n| n.checked_mul(2))
                .ok_or_else(ovf)?;
            bytes = add(bytes, names)?;
        }
    }
    bytes = add(bytes, validation_lookup_key_bytes(query, context)?)?;
    let vocabulary = ApplicationQueryObservableInfluence::ALL.len();
    let cone_bytes = tree_clone_bytes::<ApplicationQueryObservableInfluence>(vocabulary)
        .and_then(|b| b.checked_mul(2))
        .ok_or_else(ovf)?;
    bytes = bytes.checked_add(cone_bytes).ok_or_else(ovf)?;
    add(
        bytes,
        Layout::array::<ApplicationQueryObservableInfluence>(vocabulary)
            .map_err(|_| ovf())?
            .size(),
    )
}

/// Accounts heap backing for every `ApplicationQueryResultSlotKey` variant
/// that the one-shot disclosure compiler allocates into `result_rules`.
fn result_slot_key_bytes(selector: &ApplicationQueryDisclosureSelector) -> Result<usize, Failure> {
    let ovf = || MapKernelFailure::Domain(StageError::Memory);
    let sum = |lens: &[usize]| -> Result<usize, Failure> {
        lens.iter()
            .try_fold(0usize, |acc, &n| acc.checked_add(n))
            .ok_or_else(ovf)
    };
    match selector {
        ApplicationQueryDisclosureSelector::Field {
            query_type,
            slot_type,
            entity,
            aspect,
            field,
            output_name,
            value_type,
            ..
        } => sum(&[
            entity.capacity(),
            aspect.capacity(),
            field.capacity(),
            output_name.capacity(),
            query_type.retained_heap_bytes(),
            slot_type.retained_heap_bytes(),
            value_type.retained_heap_bytes(),
        ]),
        ApplicationQueryDisclosureSelector::Relation {
            query_type,
            slot_type,
            relation,
            from,
            to,
            output_name,
            ..
        } => sum(&[
            relation.capacity(),
            from.capacity(),
            to.capacity(),
            output_name.capacity(),
            query_type.retained_heap_bytes(),
            slot_type.retained_heap_bytes(),
        ]),
        ApplicationQueryDisclosureSelector::Orbit(o) => sum(&[
            o.query_type().len(),
            o.slot_type().len(),
            o.relation().len(),
            o.from().len(),
            o.to().len(),
            o.output_name().len(),
        ]),
        ApplicationQueryDisclosureSelector::Components(c) => sum(&[
            c.query_type().len(),
            c.slot_type().len(),
            c.set().len(),
            c.member().len(),
            c.output_name().len(),
        ]),
        _ => Ok(0),
    }
}

/// Accounts temporary lookup keys built by `validate_influence` even when a
/// required internal rule is absent.
fn validation_lookup_key_bytes<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<usize, Failure> {
    let ovf = || MapKernelFailure::Domain(StageError::Memory);
    let mut total = 0usize;
    let mut add3 = |a: &str, b: &str, c: &str| -> Result<(), Failure> {
        total = total
            .checked_add(a.len())
            .and_then(|n| n.checked_add(b.len()))
            .and_then(|n| n.checked_add(c.len()))
            .ok_or_else(ovf)?;
        Ok(())
    };
    let graph = query.read_graph();
    for f in graph
        .predicates()
        .iter()
        .map(|p| p.field())
        .chain(
            graph
                .where_filter()
                .map_or(&[][..], |w| w.fields())
                .iter()
                .map(|f| f.field()),
        )
        .chain(graph.ordering().iter().map(|o| o.field()))
    {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        add3(f.0, f.1, f.2)?;
    }
    for related in graph.related_predicates() {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        add3(related.relation(), related.from(), related.to())?;
        for t in related.target_predicates() {
            let f = t.field();
            add3(f.0, f.1, f.2)?;
        }
        for f in related
            .target_where_filter()
            .map_or(&[][..], |w| w.fields())
        {
            let f = f.field();
            add3(f.0, f.1, f.2)?;
        }
    }
    for guard in graph.root_paths().iter().flat_map(|p| p.guards()) {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        add3(
            guard.entity(),
            guard.aspect().as_str(),
            guard.field().as_str(),
        )?;
    }
    Ok(total)
}

fn add(bytes: u64, extra: usize) -> Result<u64, Failure> {
    u64::try_from(extra)
        .ok()
        .and_then(|extra| bytes.checked_add(extra))
        .ok_or(MapKernelFailure::Domain(StageError::Memory))
}
