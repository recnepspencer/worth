use std::collections::BTreeSet;

use crate::domain_installation::{
    WorthQueryAdmittedConsumerInvalidation, WorthQueryCollectionDeliveryCounters,
    WorthQueryCollectionDeliveryDenialKind, WorthQueryCollectionDeliveryOutcome,
    WorthQueryCollectionPatch, WorthQueryCollectionPatchOperation, WorthQueryCollectionResetCost,
    WorthQueryCollectionResetReason, WorthQueryConsumerInvalidationDelta, WorthQueryImpactClass,
    WorthQueryOperationWindowPolicy,
};
use crate::memory_workspace::WorthQueryEntityIdentity;

use super::state::{denial, WorthQueryCollectionConsumerWindow};

#[path = "planning/operations.rs"]
mod operations;
use operations::{operations_for, OperationDiff};
#[path = "planning/window_projection.rs"]
mod window_projection;
use window_projection::{next_window, reset_for};

pub(super) fn prepare_granular<D, O, F, L: crate::basis_lifecycle::BasisOperationLane>(
    state: &WorthQueryCollectionConsumerWindow,
    current: &crate::domain_installation::WorthQuerySettledDomainProjection<D, O, F, L>,
    fresh_rows: &[crate::memory_workspace::WorthQueryEntity],
    affected: &BTreeSet<WorthQueryEntityIdentity>,
    keys: &[crate::domain_installation::WorthQueryNativeAccessKey],
    replacement_targets: &[super::index::WorthQueryCollectionMaintenanceTarget],
    impact: WorthQueryImpactClass,
) -> Result<
    (
        super::WorthQueryPerformedCollectionStateMutation,
        super::WorthQueryPendingCollectionStateMutation,
    ),
    WorthQueryCollectionDeliveryDenialKind,
> {
    if state.reset_pending || state.pending_maintenance_ordinal.is_some() {
        return Err(WorthQueryCollectionDeliveryDenialKind::ResetPending);
    }
    if state.window.source_identity != current.identity()
        || state.window.binding_identity != current.bound_operation().binding_identity()
        || state.window.result_shape_identity
            != current
                .consumer_contract()
                .canonical_projection()
                .result_shape()
                .digest()
                .as_str()
    {
        return Err(WorthQueryCollectionDeliveryDenialKind::SourceMismatch);
    }
    let mut counters = WorthQueryCollectionDeliveryCounters {
        semantic_contract_checks: 3,
        ..WorthQueryCollectionDeliveryCounters::default()
    };
    let preview = state.index.preview_fresh_rows(
        &state.window,
        affected,
        keys,
        replacement_targets,
        fresh_rows,
        &mut counters,
    );
    let next = next_window(state, preview.rows, preview.has_more, counters);
    let mut operations = operations_for(
        OperationDiff {
            prior: &state.window,
            next: &next,
            impact,
            affected: &preview.consumer_affected,
        },
        &mut counters,
    );
    operations.extend(preview.delta.group_transitions().iter().map(|transition| {
        WorthQueryCollectionPatchOperation::Regroup {
            entity: transition.entity.clone(),
            from: transition.from.clone(),
            to: transition.to.clone(),
        }
    }));
    counters.operations_materialized = operations.len();
    let facts = preview.facts;
    let next_maintenance_ordinal = state
        .last_maintenance_ordinal
        .unwrap_or_default()
        .saturating_add(1);
    let performed = super::WorthQueryPerformedCollectionStateMutation {
        operations,
        facts,
        counters,
        rows: next.rows().to_vec(),
    };
    let pending = super::WorthQueryPendingCollectionStateMutation {
        delta: preview.delta,
        next,
        next_maintenance_ordinal,
    };
    Ok((performed, pending))
}

pub(super) fn plan(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorthQueryCollectionConsumerWindow,
    admitted: &WorthQueryAdmittedConsumerInvalidation<'_>,
    workspace: &crate::runtime::WorthQueryWorkspace,
) -> WorthQueryCollectionDeliveryOutcome {
    let mut counters = WorthQueryCollectionDeliveryCounters::default();
    let delta = match validate_plan(state, admitted, workspace, &mut counters) {
        Ok(delta) => delta,
        Err(kind) => return no_delivery(kind, counters),
    };
    if state
        .pending_maintenance_ordinal
        .is_some_and(|pending| pending != delta.maintenance_ordinal())
    {
        counters.operations_materialized = 1;
        return reset_patch(
            state,
            delta,
            ResetPatchSpec {
                impact: WorthQueryImpactClass::UnsupportedEscalation,
                operation: WorthQueryCollectionPatchOperation::ResetRequired {
                    reason: WorthQueryCollectionResetReason::UnappliedPriorPatch,
                    cost: WorthQueryCollectionResetCost {
                        fresh_execution_required: true,
                        maximum_replacement_rows: state.window.rows().len(),
                    },
                },
                counters,
            },
        );
    }
    let impact = delta.impact().class();
    if let Some(reset) = reset_for(impact, state.window.rows().len()) {
        counters.operations_materialized = 1;
        return reset_patch(
            state,
            delta,
            ResetPatchSpec {
                impact,
                operation: reset,
                counters,
            },
        );
    }
    plan_incremental(
        execution,
        state,
        IncrementalPlan {
            delta,
            workspace,
            impact,
            counters,
        },
    )
}

struct IncrementalPlan<'a> {
    delta: &'a WorthQueryConsumerInvalidationDelta,
    workspace: &'a crate::runtime::WorthQueryWorkspace,
    impact: WorthQueryImpactClass,
    counters: WorthQueryCollectionDeliveryCounters,
}

fn plan_incremental(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorthQueryCollectionConsumerWindow,
    request: IncrementalPlan<'_>,
) -> WorthQueryCollectionDeliveryOutcome {
    let IncrementalPlan {
        delta,
        workspace,
        impact,
        mut counters,
    } = request;
    let affected = delta
        .affected_entity_identities()
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let preview = match preview_or_reset(
        execution,
        state,
        PreviewContext {
            affected: &affected,
            delta,
            workspace,
        },
        &mut counters,
    ) {
        Ok(preview) => preview,
        Err(outcome) => return outcome,
    };
    let next = next_window(state, preview.rows, preview.has_more, counters);
    let mut operations = operations_for(
        OperationDiff {
            prior: &state.window,
            next: &next,
            impact,
            affected: &affected,
        },
        &mut counters,
    );
    operations.extend(preview.delta.group_transitions().iter().map(|transition| {
        WorthQueryCollectionPatchOperation::Regroup {
            entity: transition.entity.clone(),
            from: transition.from.clone(),
            to: transition.to.clone(),
        }
    }));
    if operations.is_empty() {
        state.index.apply(preview.delta);
        state.last_maintenance_ordinal = Some(delta.maintenance_ordinal());
        return no_delivery(
            WorthQueryCollectionDeliveryDenialKind::NoSemanticWindowEffect,
            counters,
        );
    }
    counters.operations_materialized = operations.len();
    state.pending_maintenance_ordinal = Some(delta.maintenance_ordinal());
    WorthQueryCollectionDeliveryOutcome::Patch(WorthQueryCollectionPatch {
        authority: delta.authority().clone(),
        maintenance_ordinal: delta.maintenance_ordinal(),
        impact,
        prior_cursor: state.window.cursor().clone(),
        next,
        operations: std::mem::take(&mut operations),
        facts: preview.facts,
        foundational_invalidation: delta.foundational_projection(),
        counters,
        index_delta: Some(preview.delta),
    })
}

fn validate_plan<'a>(
    state: &WorthQueryCollectionConsumerWindow,
    admitted: &'a WorthQueryAdmittedConsumerInvalidation<'_>,
    workspace: &crate::runtime::WorthQueryWorkspace,
    counters: &mut WorthQueryCollectionDeliveryCounters,
) -> Result<&'a WorthQueryConsumerInvalidationDelta, WorthQueryCollectionDeliveryDenialKind> {
    if state.reset_pending {
        return Err(WorthQueryCollectionDeliveryDenialKind::ResetPending);
    }
    counters.semantic_contract_checks += 1;
    if !state.index.delivery_supported() {
        return Err(WorthQueryCollectionDeliveryDenialKind::UnsupportedCollectionDelivery);
    }
    counters.invalidation_authority_checks += 1;
    if !admitted.remains_current(workspace) {
        return Err(WorthQueryCollectionDeliveryDenialKind::ForeignOrStaleInvalidation);
    }
    let delta = admitted.delta();
    let Some(authority) = &state.authority else {
        return Err(WorthQueryCollectionDeliveryDenialKind::WrongLease);
    };
    counters.lease_checks += 1;
    if !authority.is_same_current_authority_as(delta.authority()) {
        return Err(WorthQueryCollectionDeliveryDenialKind::WrongLease);
    }
    counters.cursor_checks += 1;
    if state
        .last_maintenance_ordinal
        .is_some_and(|last| delta.maintenance_ordinal() <= last)
    {
        return Err(WorthQueryCollectionDeliveryDenialKind::DuplicateOrReorderedDelivery);
    }
    Ok(delta)
}

struct PreviewContext<'a> {
    affected: &'a BTreeSet<WorthQueryEntityIdentity>,
    delta: &'a WorthQueryConsumerInvalidationDelta,
    workspace: &'a crate::runtime::WorthQueryWorkspace,
}

fn preview_or_reset(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    state: &mut WorthQueryCollectionConsumerWindow,
    context: PreviewContext<'_>,
    counters: &mut WorthQueryCollectionDeliveryCounters,
) -> Result<super::index::WorthQueryCollectionIndexPreview, WorthQueryCollectionDeliveryOutcome> {
    let preview = state
        .index
        .preview(
            execution,
            super::index::WorthQueryCollectionPreviewRequest {
                window: &state.window,
                affected: context.affected,
                keys: context.delta.affected_native_keys(),
                workspace: context.workspace,
            },
            counters,
        )
        .map_err(|()| {
            counters.operations_materialized = 1;
            let impact = WorthQueryImpactClass::UnsupportedEscalation;
            let reset = reset_for(impact, state.window.rows().len())
                .expect("unsupported escalation always requires reset");
            reset_patch(
                state,
                context.delta,
                ResetPatchSpec {
                    impact,
                    operation: reset,
                    counters: *counters,
                },
            )
        })?;
    if preview.has_more
        && state.index.window_policy() == WorthQueryOperationWindowPolicy::CompleteCollection
    {
        counters.operations_materialized = 1;
        let reset = WorthQueryCollectionPatchOperation::ResetRequired {
            reason: WorthQueryCollectionResetReason::UnsupportedIncrementalMeaning,
            cost: WorthQueryCollectionResetCost {
                fresh_execution_required: true,
                maximum_replacement_rows: state.window.admitted_width(),
            },
        };
        return Err(reset_patch(
            state,
            context.delta,
            ResetPatchSpec {
                impact: WorthQueryImpactClass::UnsupportedEscalation,
                operation: reset,
                counters: *counters,
            },
        ));
    }
    Ok(preview)
}

struct ResetPatchSpec {
    impact: WorthQueryImpactClass,
    operation: WorthQueryCollectionPatchOperation,
    counters: WorthQueryCollectionDeliveryCounters,
}

fn reset_patch(
    state: &mut WorthQueryCollectionConsumerWindow,
    delta: &WorthQueryConsumerInvalidationDelta,
    spec: ResetPatchSpec,
) -> WorthQueryCollectionDeliveryOutcome {
    state.pending_maintenance_ordinal = Some(delta.maintenance_ordinal());
    WorthQueryCollectionDeliveryOutcome::Patch(WorthQueryCollectionPatch {
        authority: delta.authority().clone(),
        maintenance_ordinal: delta.maintenance_ordinal(),
        impact: spec.impact,
        prior_cursor: state.window.cursor().clone(),
        next: state.window.targetized(
            state.window.capability_identity,
            state.window.capability_generation,
        ),
        operations: vec![spec.operation],
        facts: Vec::new(),
        foundational_invalidation: delta.foundational_projection(),
        counters: spec.counters,
        index_delta: None,
    })
}

fn no_delivery(
    kind: WorthQueryCollectionDeliveryDenialKind,
    counters: WorthQueryCollectionDeliveryCounters,
) -> WorthQueryCollectionDeliveryOutcome {
    WorthQueryCollectionDeliveryOutcome::NoDelivery(denial(kind, counters))
}
