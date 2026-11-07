use super::subscription_fixtures::{
    activation_for, activation_with_context, active_attachment, active_budget,
};
use super::{MilestoneNineTwoFailureClass, MilestoneNineTwoRejectionBundle};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::harness::certification::digest_parts;
use crate::live::LiveQueryFamily;
use crate::subscription::{
    admit_active_subscription_lane, build_active_delivery_work_packet,
    deny_preview_authoritative_sharing, deny_raw_bridge_invalidation_delivery,
    deny_raw_cdc_delivery_fallback, discard_preview_subscription,
    lower_query_subscription_maintenance_delta, measure_preview_subscription_residue,
    open_active_subscription_lane, ActiveAllocationScopeWidth,
    ActiveDeliveryAffectedAttachmentWidth, ActiveDeliveryAffectedLaneWidth,
    ActiveDeliveryContinuationWidth, ActiveDeliveryDensityPosture,
    ActiveDeliveryPreviewResidueWidth, ActiveSubscriptionAllocationPosture,
    ActiveSubscriptionRuntime, MaintenanceDeltaWidth, PatchGroupWidth, PreviewResidueWidth,
    QuerySubscriptionConstructionSource, QuerySubscriptionMaintenanceDelta,
    QuerySubscriptionMaintenanceDeltaKind,
};

pub(super) fn masked_sharing_rejection(
) -> Result<MilestoneNineTwoRejectionBundle, IdentityDerivationDenial> {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let source = activation_with_context("policy-unmasked")?;
    let foreign = activation_with_context("policy-masked")?;
    let admission = admit_active_subscription_lane(source, active_budget()).unwrap();
    let open_handle = open_active_subscription_lane(&mut runtime, admission).unwrap();
    let foreign_admission = admit_active_subscription_lane(foreign, active_budget()).unwrap();
    let error = crate::subscription::join_active_subscription_lane(
        &mut runtime,
        &open_handle,
        foreign_admission,
    )
    .unwrap_err();
    Ok(rejection(
        MilestoneNineTwoFailureClass::ActiveLifecycleDenied,
        error.denial_kind().as_str(),
        error.source_projection().label(),
        error.counters().counter_projection().label().to_string(),
    ))
}

pub(super) fn raw_cdc_rejection() -> MilestoneNineTwoRejectionBundle {
    let error = deny_raw_cdc_delivery_fallback("raw-cdc").unwrap_err();
    rejection(
        MilestoneNineTwoFailureClass::DeliveryDenied,
        error.denial_kind().as_str(),
        error
            .source_projection()
            .expect("fixture digest must fit")
            .label(),
        error.counters().counter_projection().label().to_string(),
    )
}

pub(super) fn raw_bridge_rejection() -> MilestoneNineTwoRejectionBundle {
    let error = deny_raw_bridge_invalidation_delivery("raw-bridge").unwrap_err();
    rejection(
        MilestoneNineTwoFailureClass::DeliveryDenied,
        error.denial_kind().as_str(),
        error
            .source_projection()
            .expect("fixture digest must fit")
            .label(),
        error.counters().counter_projection().label().to_string(),
    )
}

pub(super) fn preview_sharing_rejection(
) -> Result<MilestoneNineTwoRejectionBundle, IdentityDerivationDenial> {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let (handle, attachment) = active_attachment(&mut runtime)?;
    let isolation = crate::subscription::admit_preview_subscription_isolation(
        &attachment,
        "preview-epoch",
        PreviewResidueWidth::measured(1),
    )
    .unwrap();
    let error = deny_preview_authoritative_sharing(&isolation, &handle).unwrap_err();
    Ok(rejection(
        MilestoneNineTwoFailureClass::PreviewIsolationDenied,
        error.denial_kind().as_str(),
        error.source_projection().label(),
        error.counters().counter_projection().label().to_string(),
    ))
}

pub(super) fn preview_residue_rejection(
) -> Result<MilestoneNineTwoRejectionBundle, IdentityDerivationDenial> {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let (_, attachment) = active_attachment(&mut runtime)?;
    let isolation = crate::subscription::admit_preview_subscription_isolation(
        &attachment,
        "preview-epoch",
        PreviewResidueWidth::measured(2),
    )
    .unwrap();
    let residue = measure_preview_subscription_residue(
        PreviewResidueWidth::measured(1),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(1),
        PreviewResidueWidth::measured(0),
    );
    let error = discard_preview_subscription(isolation, residue).unwrap_err();
    Ok(rejection(
        MilestoneNineTwoFailureClass::PreviewIsolationDenied,
        error.denial_kind().as_str(),
        error.source_projection().label(),
        error.counters().counter_projection().label().to_string(),
    ))
}

pub(super) fn dense_refresh_rejection(
) -> Result<MilestoneNineTwoRejectionBundle, IdentityDerivationDenial> {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let (_, attachment) = active_attachment(&mut runtime)?;
    let delta = QuerySubscriptionMaintenanceDelta::admitted_with_scope_label(
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        attachment.lane_digest().clone(),
        "dense",
        MaintenanceDeltaWidth::measured(1),
    )?;
    let (delta, lowering_report, _) = lower_query_subscription_maintenance_delta(delta).unwrap();
    let error = build_active_delivery_work_packet(
        &mut runtime,
        &attachment,
        delta,
        lowering_report,
        ActiveDeliveryDensityPosture::DenseRefreshDenied,
        ActiveDeliveryAffectedLaneWidth::measured(1),
        ActiveDeliveryAffectedAttachmentWidth::measured(1),
        PatchGroupWidth::measured(1),
        ActiveDeliveryContinuationWidth::measured(0),
        ActiveDeliveryPreviewResidueWidth::measured(0),
        ActiveAllocationScopeWidth::measured(1),
        ActiveSubscriptionAllocationPosture::PatchScratch,
    )
    .unwrap_err();
    Ok(rejection(
        MilestoneNineTwoFailureClass::DeliveryDenied,
        error.denial_kind().as_str(),
        error.source_projection()?.label(),
        error.counters().counter_projection().label().to_string(),
    ))
}

pub(super) fn store_backed_restart_rejection(
) -> Result<MilestoneNineTwoRejectionBundle, IdentityDerivationDenial> {
    let activation = activation_for(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::FacadeLive,
    )?;
    let error = admit_active_subscription_lane(
        activation,
        active_budget().with_store_backed_restart_request(),
    )
    .unwrap_err();
    Ok(rejection(
        MilestoneNineTwoFailureClass::ActiveLifecycleDenied,
        error.denial_kind().as_str(),
        error.source_projection().label(),
        error.counters().counter_projection().label().to_string(),
    ))
}

fn rejection(
    failure_class: MilestoneNineTwoFailureClass,
    failure_kind: &str,
    source_digest: &str,
    counter_snapshot: String,
) -> MilestoneNineTwoRejectionBundle {
    let failure_digest = digest_parts(&[
        format!("failure_class:{failure_class:?}"),
        format!("failure_kind:{failure_kind}"),
        format!("source:{source_digest}"),
        format!("counters:{counter_snapshot}"),
    ]);
    MilestoneNineTwoRejectionBundle {
        failure_class,
        failure_kind: failure_kind.to_string(),
        lifecycle_denial_digest: digest_parts(&[
            source_digest.to_string(),
            failure_kind.to_string(),
            counter_snapshot.clone(),
        ]),
        failure_digest,
        counter_snapshot,
    }
}
