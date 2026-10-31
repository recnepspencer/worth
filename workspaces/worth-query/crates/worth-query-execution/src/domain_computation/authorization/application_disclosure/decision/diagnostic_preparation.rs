//! Pay the existing policy preparation pass from the diagnostic request's payer.
use super::*;
use crate::domain_computation::primary_graph::admitted_capture::{
    checkpoint, run_retained_stage, CaptureStop,
};
use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, MapKernelContext, MapKernelFailure, MapKernelStop,
};
use worth_query_declaration::facade::application_query::ApplicationQueryDisclosureSelector;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

pub(in crate::domain_computation) enum DiagnosticPreparationDenial {
    Governance(WorthQueryApplicationQueryGovernanceDenialKind),
}

pub(in crate::domain_computation) fn prepare_application_diagnostic_governance<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    contract: WorthQueryAdmittedApplicationDisclosureContract,
    pending: WorthQueryPendingApplicationQueryGovernance,
    execution: &ExecutionRequest<'_, '_>,
    memory: &mut Option<ExecutionMemoryReservation>,
) -> Result<WorthQueryPreparedApplicationGovernance, CaptureStop<DiagnosticPreparationDenial>> {
    run_retained_stage(execution, memory, |context, _| {
        checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
        if let WorthQueryAdmittedApplicationDisclosureContract::Governed {
            capability_name,
            capability_type,
            result_rules,
            internal_rules,
            ..
        } = &contract
        {
            for (left, right) in [
                (capability_name, &pending.capability_name),
                (capability_type, &pending.capability_type),
            ] {
                pay_widths(context, &[left.len(), right.len()])?;
            }
            let mut slot_width = std::mem::size_of::<ApplicationQueryResultSlotKey>();
            for rule in query.disclosure().rules() {
                checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
                if !matches!(
                    rule.selector(),
                    ApplicationQueryDisclosureSelector::Field { .. }
                        | ApplicationQueryDisclosureSelector::Relation { .. }
                ) {
                    continue;
                }
                let parts = super::super::contract::selector_parts(rule.selector());
                let width = parts
                    .iter()
                    .try_fold(
                        std::mem::size_of::<ApplicationQueryResultSlotKey>(),
                        |width, text| width.checked_add(text.len())?.checked_add(1),
                    )
                    .ok_or_else(overflow)?;
                slot_width = slot_width.max(width);
            }
            let insertion = u64::try_from(result_rules.len())
                .ok()
                .and_then(|n| n.checked_add(1))
                .and_then(|n| n.checked_mul(u64::try_from(slot_width).ok()?))
                .ok_or_else(overflow)?;
            let pending_width = pending.disclosure_value.semantic_byte_width();
            for rule in internal_rules {
                pay_widths(
                    context,
                    &[rule.disclosure_value().semantic_byte_width(), pending_width],
                )?;
            }
            for rule in result_rules.values() {
                pay_widths(
                    context,
                    &[rule.disclosure_value().semantic_byte_width(), pending_width],
                )?;
                checkpoint(context, insertion).map_err(MapKernelFailure::Stop)?;
            }
        }
        // The compiler already paid simultaneous input/output result trees;
        // funded pending capture paid both actual pending/bound value copies.
        // This is the sole policy pass, with its original denial preserved.
        preparation::prepare(contract, Some(pending)).map_err(|cause| {
            MapKernelFailure::Domain(DiagnosticPreparationDenial::Governance(cause))
        })
    })
}

fn pay_widths(
    context: &mut MapKernelContext<'_, '_>,
    widths: &[usize],
) -> Result<(), MapKernelFailure<DiagnosticPreparationDenial>> {
    let work = widths
        .iter()
        .try_fold(1u64, |sum, width| {
            sum.checked_add(u64::try_from(*width).ok()?)
        })
        .ok_or_else(overflow)?;
    checkpoint(context, work).map_err(MapKernelFailure::Stop)
}

fn overflow() -> MapKernelFailure<DiagnosticPreparationDenial> {
    MapKernelFailure::Stop(MapKernelStop::WorkCounterOverflow)
}
