use crate::declarative_live::{DeclarativeLiveQueryRequest, DeclarativeLiveViewShape};
use crate::memory_workspace::{WorthQueryMutationKind, WorthQueryMutationReceipt};
use crate::subscription::{
    advance_subscription_acknowledgement, build_active_delivery_work_packet,
    emit_query_delivery_batch, lower_query_subscription_maintenance_delta,
    open_query_delivery_window, ActiveAllocationScopeWidth, ActiveDeliveryAffectedAttachmentWidth,
    ActiveDeliveryAffectedLaneWidth, ActiveDeliveryContinuationWidth, ActiveDeliveryDensityPosture,
    ActiveDeliveryPreviewResidueWidth, ActiveSubscriptionAllocationPosture,
    ActiveSubscriptionRuntime, DeliveryBackpressurePolicy, DeliveryWindowWidth,
    MaintenanceDeltaWidth, PatchGroupWidth, QueryDeliveryWindowBudget,
    QuerySubscriptionMaintenanceDelta, QuerySubscriptionMaintenanceDeltaKind,
};
use worth_foundational::facade::CanonicalFieldPath;

mod target_selection;

pub(crate) use target_selection::route_live_subscription_delivery;

use super::delivery::{
    WorthQueryLiveMutationRoutingWork, WorthQueryRuntimeDeliveryBatch,
    WorthQueryRuntimeLiveSubscriptionState, WorthQueryRuntimeRetainedDelivery,
};
use super::{
    WorthQueryAspectTouch, WorthQueryLiveArtifactTarget, WorthQueryLiveGraphReadAccessPlan,
    WorthQueryLiveGraphReadMaintenanceBudget, WorthQueryLiveGraphReadMaintenanceReceipt,
    WorthQueryRuntimeError,
};

struct RelevantLiveSubscriptionDeltaRoute<'a> {
    active_subscriptions: &'a mut ActiveSubscriptionRuntime,
    state: &'a mut WorthQueryRuntimeLiveSubscriptionState,
    target: &'a WorthQueryLiveArtifactTarget,
    receipt: &'a WorthQueryMutationReceipt,
    delta: &'a crate::memory_workspace::WorthQueryMutationDelta,
    delta_kind: QuerySubscriptionMaintenanceDeltaKind,
    preclassified_installed_impact:
        Option<crate::domain_installation::WorthQueryPreclassifiedInstalledLiveImpact>,
    routing_work: WorthQueryLiveMutationRoutingWork,
    affected: &'a mut Vec<WorthQueryLiveArtifactTarget>,
}

impl RelevantLiveSubscriptionDeltaRoute<'_> {
    fn route(mut self) -> Result<(), WorthQueryRuntimeError> {
        let view_name = self.target.view_name().to_string();
        let patch_width = self.delta.admitted_touched_aspects().len().max(1) as u64;
        let maintenance_delta = admitted_subscription_maintenance_delta(
            self.receipt,
            self.delta,
            self.delta_kind,
            self.state,
            patch_width,
        );
        let live_graph_read_maintenance = live_graph_read_maintenance_receipt(
            &view_name,
            self.state,
            &maintenance_delta,
            patch_width,
        )?;
        let (maintenance_delta, lowering_report, _) =
            lower_query_subscription_maintenance_delta(maintenance_delta).map_err(|error| {
                live_delivery_stage_error(&view_name, "delivery-delta-lowering", error.message())
            })?;
        let window = open_query_delivery_window(
            self.active_subscriptions,
            &self.state.consumer_attachment,
            runtime_delivery_window_budget(patch_width),
        )
        .map_err(|error| {
            live_delivery_stage_error(&view_name, "delivery-window", error.message())
        })?;
        let work_packet = build_active_delivery_work_packet(
            self.active_subscriptions,
            &self.state.consumer_attachment,
            maintenance_delta,
            lowering_report,
            ActiveDeliveryDensityPosture::SparseDelta,
            ActiveDeliveryAffectedLaneWidth::measured(1),
            ActiveDeliveryAffectedAttachmentWidth::measured(1),
            PatchGroupWidth::measured(patch_width),
            ActiveDeliveryContinuationWidth::measured(0),
            ActiveDeliveryPreviewResidueWidth::measured(0),
            ActiveAllocationScopeWidth::measured(1),
            ActiveSubscriptionAllocationPosture::PatchScratch,
        )
        .map_err(|error| {
            live_delivery_stage_error(&view_name, "delivery-work-packet", error.message())
        })?;
        let batch = emit_query_delivery_batch(self.active_subscriptions, window, work_packet)
            .map_err(|error| {
                live_delivery_stage_error(&view_name, "delivery-batch", error.message())
            })?;
        self.retain_emitted_batch(&view_name, &batch, live_graph_read_maintenance)?;
        self.affected.push(self.target.clone());
        Ok(())
    }

    fn retain_emitted_batch(
        &mut self,
        view_name: &str,
        batch: &crate::subscription::QueryDeliveryBatch,
        live_graph_read_maintenance: WorthQueryLiveGraphReadMaintenanceReceipt,
    ) -> Result<(), WorthQueryRuntimeError> {
        let mut runtime_batch = WorthQueryRuntimeDeliveryBatch::from_query_delivery(
            view_name,
            batch,
            self.receipt.commit_identity.bridge_identity().cloned(),
            Some(self.delta.clone()),
            Some(live_graph_read_maintenance),
        );
        if let Some(impact) = self.preclassified_installed_impact.take() {
            runtime_batch = runtime_batch.with_preclassified_installed_impact(impact);
        }
        runtime_batch = runtime_batch.with_routing_work(self.routing_work);
        self.state.last_delivery = Some(WorthQueryRuntimeRetainedDelivery::from_batch(
            &runtime_batch,
        ));
        self.state.delivery_batches.push(runtime_batch);
        self.state.consumer_attachment = advance_subscription_acknowledgement(
            self.active_subscriptions,
            self.state.consumer_attachment.clone(),
            batch.receipt().clone(),
        )
        .map_err(|error| {
            live_delivery_stage_error(view_name, "delivery-acknowledgement", &format!("{error:?}"))
        })?;
        Ok(())
    }
}

fn live_delivery_stage_error(
    view_name: &str,
    stage: &'static str,
    message: &str,
) -> WorthQueryRuntimeError {
    WorthQueryRuntimeError::LiveSubscriptionInstallation {
        view_name: view_name.to_string(),
        stage,
        message: message.to_string(),
    }
}

fn admitted_subscription_maintenance_delta(
    receipt: &WorthQueryMutationReceipt,
    delta: &crate::memory_workspace::WorthQueryMutationDelta,
    delta_kind: QuerySubscriptionMaintenanceDeltaKind,
    state: &WorthQueryRuntimeLiveSubscriptionState,
    patch_width: u64,
) -> QuerySubscriptionMaintenanceDelta {
    let commit_evidence = receipt.commit_identity.evidence_identity();
    let entity_evidence = delta.entity_identity.evidence_identity();
    QuerySubscriptionMaintenanceDelta::admitted_with_typed_scope(
        delta_kind,
        state.active_lane_handle.lane_digest().clone(),
        &commit_evidence,
        delta.target_collection_identity().evidence_identity(),
        &entity_evidence,
        MaintenanceDeltaWidth::measured(patch_width),
    )
}

fn live_graph_read_maintenance_receipt(
    view_name: &str,
    state: &WorthQueryRuntimeLiveSubscriptionState,
    maintenance_delta: &QuerySubscriptionMaintenanceDelta,
    patch_width: u64,
) -> Result<WorthQueryLiveGraphReadMaintenanceReceipt, WorthQueryRuntimeError> {
    let live_graph_access_plan = WorthQueryLiveGraphReadAccessPlan::from_live_installation(
        &state.installation,
        WorthQueryLiveGraphReadMaintenanceBudget::bounded_with_snapshot_refresh(),
    )
    .map_err(
        |error| WorthQueryRuntimeError::LiveSubscriptionInstallation {
            view_name: view_name.to_string(),
            stage: "live-graph-access-maintenance-plan",
            message: error.message().to_string(),
        },
    )?;
    Ok(
        WorthQueryLiveGraphReadMaintenanceReceipt::from_maintenance_delta(
            &live_graph_access_plan,
            maintenance_delta,
            patch_width as usize,
        ),
    )
}

fn runtime_delivery_window_budget(patch_width: u64) -> QueryDeliveryWindowBudget {
    let bounded_patch_width = patch_width.max(1);
    QueryDeliveryWindowBudget::admitted(
        DeliveryWindowWidth::measured(1),
        PatchGroupWidth::measured(bounded_patch_width),
        MaintenanceDeltaWidth::measured(bounded_patch_width),
        ActiveAllocationScopeWidth::measured(1),
        ActiveSubscriptionAllocationPosture::DeliveryWindowArena,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    )
}

fn maintenance_delta_kind_for_live_change(
    request: &DeclarativeLiveQueryRequest,
    mutation_kind: &WorthQueryMutationKind,
    aspect_touches: &[WorthQueryAspectTouch],
) -> Option<QuerySubscriptionMaintenanceDeltaKind> {
    if !live_change_is_relevant(request, mutation_kind, aspect_touches) {
        return None;
    }
    match request.view_shape() {
        DeclarativeLiveViewShape::InspectorObserved
        | DeclarativeLiveViewShape::InspectorFocused { .. }
        | DeclarativeLiveViewShape::IdentityAwareInspectorFocused { .. } => {
            Some(QuerySubscriptionMaintenanceDeltaKind::InspectorFocusDelta)
        }
        DeclarativeLiveViewShape::KanbanGrouped { grouping_aspect } => {
            if is_membership_change(mutation_kind)
                || aspect_touches
                    .iter()
                    .any(|touch| touch.native_aspect_key() == grouping_aspect)
            {
                Some(QuerySubscriptionMaintenanceDeltaKind::GroupedMembershipDelta)
            } else {
                Some(QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta)
            }
        }
        DeclarativeLiveViewShape::ListSplice | DeclarativeLiveViewShape::Table => {
            if is_membership_change(mutation_kind) {
                Some(QuerySubscriptionMaintenanceDeltaKind::CollectionMembershipDelta)
            } else {
                Some(QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta)
            }
        }
        DeclarativeLiveViewShape::Detail => {
            if matches!(mutation_kind, WorthQueryMutationKind::Deleted) {
                Some(QuerySubscriptionMaintenanceDeltaKind::CollectionMembershipDelta)
            } else {
                Some(QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta)
            }
        }
    }
}

fn maintenance_delta_kind_for_classified_impact(
    impact: crate::domain_installation::WorthQueryImpactClass,
    request: &DeclarativeLiveQueryRequest,
) -> Option<QuerySubscriptionMaintenanceDeltaKind> {
    use crate::domain_installation::WorthQueryImpactClass as Impact;
    match impact {
        Impact::UnaffectedOrSuppressed => None,
        Impact::ValuePatch => Some(QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta),
        Impact::MembershipSplice => {
            Some(QuerySubscriptionMaintenanceDeltaKind::CollectionMembershipDelta)
        }
        Impact::ReorderOrRegroup => Some(match request.view_shape() {
            DeclarativeLiveViewShape::KanbanGrouped { .. } => {
                QuerySubscriptionMaintenanceDeltaKind::GroupedMembershipDelta
            }
            _ => QuerySubscriptionMaintenanceDeltaKind::CollectionOrderDelta,
        }),
        Impact::WindowShift => {
            Some(QuerySubscriptionMaintenanceDeltaKind::BoundedMaterializationScopeDelta)
        }
        Impact::Reexecute
        | Impact::ExplicitRebind
        | Impact::Replacement
        | Impact::Retirement
        | Impact::UnsupportedEscalation => {
            Some(QuerySubscriptionMaintenanceDeltaKind::GapNoticeDelta)
        }
    }
}

fn live_change_is_relevant(
    request: &DeclarativeLiveQueryRequest,
    mutation_kind: &WorthQueryMutationKind,
    aspect_touches: &[WorthQueryAspectTouch],
) -> bool {
    if is_membership_change(mutation_kind) || aspect_touches.is_empty() {
        return true;
    }
    aspect_touches.iter().any(|changed| {
        request_dependency_fields(request)
            .any(|field| changed.matches_or_contains(&live_request_field_touch(field)))
            || match request.view_shape() {
                DeclarativeLiveViewShape::InspectorFocused { focused_aspect }
                | DeclarativeLiveViewShape::IdentityAwareInspectorFocused {
                    focused_aspect, ..
                } => changed.native_aspect_key() == focused_aspect,
                DeclarativeLiveViewShape::KanbanGrouped { grouping_aspect } => {
                    changed.native_aspect_key() == grouping_aspect
                }
                _ => false,
            }
    })
}

fn request_dependency_fields(
    request: &DeclarativeLiveQueryRequest,
) -> impl Iterator<Item = &crate::authoring::AspectFieldKey> {
    request
        .query_projection()
        .iter()
        .map(|field| field.source_field_key())
        .chain(
            request
                .predicate_filters()
                .iter()
                .map(|filter| filter.source_field_key()),
        )
        .chain(
            request
                .ordering()
                .iter()
                .map(|ordering| ordering.source_field_key()),
        )
}

fn is_membership_change(mutation_kind: &WorthQueryMutationKind) -> bool {
    matches!(
        mutation_kind,
        WorthQueryMutationKind::Created | WorthQueryMutationKind::Deleted
    )
}

fn live_request_field_touch(field: &crate::authoring::AspectFieldKey) -> WorthQueryAspectTouch {
    WorthQueryAspectTouch::from_native_parts(
        field.native_aspect_key(),
        Some(CanonicalFieldPath::single(field.native_field_key())),
    )
}
