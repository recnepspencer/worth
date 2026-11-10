use crate::application::{
    checked_route_plan_from_progressed_with_profile, receipt_materialized_profile_for_tier,
    worth_query_checked_declaration_envelope,
    worth_query_checked_declaration_receipt_with_materialized_profile,
    WorthQueryAdmittedDeclarationProgression,
    WorthQueryDeclarationEntryOrchestrationArtifactPolicy,
    WorthQueryDeclarationEntryOrchestrationExposureLevel,
    WorthQueryDeclarationEntryOrchestrationInput, WorthQueryDeclarationEntryOrchestrationPlan,
    WorthQueryDeclarationEntryOrchestrationProduct, WorthQueryDeclarationEntryOrchestrationStage,
    WorthQueryDeclarationEntryOrchestrationStageRecord, WorthQueryDeclarationEnvelopeChecked,
    WorthQueryDeclarationEnvelopeInput, WorthQueryDeclarationInput,
    WorthQueryDeclarationReceiptChecked, WorthQueryDeclarationReceiptInput,
    WorthQueryDeclarationRouteIntent, WorthQueryDeclarationRoutePlanChecked,
    WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
};
use crate::target_binding::{resolve_admitted_progression_target, WorthQueryBindingTargetWitness};

use super::super::artifacts::canonical_digest_token;
use super::super::materialization::foundational_materialization_tier;

pub(crate) enum WorthQueryDeclarationEntryProductChecked<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    RoutePlan(WorthQueryDeclarationRoutePlanChecked<D, I>),
    Receipt(WorthQueryDeclarationReceiptChecked<D, I>),
    Envelope(WorthQueryDeclarationEnvelopeChecked<D, I>),
}

pub(crate) struct WorthQueryLoweredDeclarationEntryProductOrchestration<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    pub(crate) plan: WorthQueryDeclarationEntryOrchestrationPlan<D, I>,
    pub(crate) checked: WorthQueryDeclarationEntryProductChecked<D, I>,
    pub(crate) step_records: Vec<WorthQueryDeclarationEntryOrchestrationStageRecord>,
}

pub(crate) fn worth_query_lower_declaration_entry_product_orchestration_from_progressed_on_handle<
    D: WorthQueryDomainEntryMarker,
    C: WorthQueryDomainOperatingContext<D>,
    I: WorthQueryDeclarationInput<D>,
>(
    handle: &crate::application::WorthQueryInstalledDomainDeclarationContext<D, C>,
    progressed: WorthQueryAdmittedDeclarationProgression<D, I>,
    exposure_level: WorthQueryDeclarationEntryOrchestrationExposureLevel,
    artifact_policy: WorthQueryDeclarationEntryOrchestrationArtifactPolicy,
    product: WorthQueryDeclarationEntryOrchestrationProduct,
    route_intent: Option<WorthQueryDeclarationRouteIntent>,
) -> WorthQueryLoweredDeclarationEntryProductOrchestration<D, I> {
    let resolved_progression = resolve_admitted_progression_target(progressed);
    let (_, _, _, _, _, aspect_contract, reviewed_aspect_coverage) = resolved_progression
        .target()
        .semantics()
        .admitted_declaration_progression()
        .expect("resolved progression target should retain admitted progression semantics");
    let orchestration_input = WorthQueryDeclarationEntryOrchestrationInput::new(
        handle.retained_world_basis(),
        aspect_contract.clone(),
        reviewed_aspect_coverage.clone(),
        crate::application::WorthQueryDeclarationAspectCoverageBasis::ReviewedRetainedCoverage,
        exposure_level,
        artifact_policy,
    );
    let plan = WorthQueryDeclarationEntryOrchestrationPlan::from_progressed(
        orchestration_input,
        product,
        route_intent,
    );
    let mut step_records = vec![
        WorthQueryDeclarationEntryOrchestrationStageRecord::admitted(
            WorthQueryDeclarationEntryOrchestrationStage::ProgressionAdmitted,
            Some(resolved_progression.target().binding_digest().to_string()),
        ),
    ];
    let route_checked = checked_route_plan_from_progressed_with_profile(
        handle,
        resolved_progression.into_progressed(),
        route_intent,
        plan.foundational_evidence_profile(),
    );
    step_records.push(foundational_record(&plan, &route_checked));
    let checked = match product {
        WorthQueryDeclarationEntryOrchestrationProduct::RoutePlan => {
            append_route_stop(&mut step_records, &plan, &route_checked);
            WorthQueryDeclarationEntryProductChecked::RoutePlan(route_checked)
        }
        WorthQueryDeclarationEntryOrchestrationProduct::Receipt => {
            append_route_progress(&mut step_records, &plan, &route_checked);
            let receipt_checked = worth_query_checked_declaration_receipt_with_materialized_profile(
                WorthQueryDeclarationReceiptInput::route_checked(route_checked),
                &receipt_materialized_profile_for_tier(plan.receipt_materialization_tier()),
                plan.receipt_materialization_tier(),
            );
            append_receipt_stop(&mut step_records, &plan, &receipt_checked);
            WorthQueryDeclarationEntryProductChecked::Receipt(receipt_checked)
        }
        WorthQueryDeclarationEntryOrchestrationProduct::Envelope => {
            let receipt_checked = lower_receipt_checked(&mut step_records, &plan, route_checked);
            let envelope_checked =
                lower_envelope_checked(&mut step_records, &plan, receipt_checked);
            WorthQueryDeclarationEntryProductChecked::Envelope(envelope_checked)
        }
    };
    WorthQueryLoweredDeclarationEntryProductOrchestration {
        plan,
        checked,
        step_records,
    }
}

fn foundational_record<D, I>(
    plan: &WorthQueryDeclarationEntryOrchestrationPlan<D, I>,
    checked: &WorthQueryDeclarationRoutePlanChecked<D, I>,
) -> WorthQueryDeclarationEntryOrchestrationStageRecord
where
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
{
    let evidence = route_foundational_evidence(checked);
    WorthQueryDeclarationEntryOrchestrationStageRecord::automated(
        WorthQueryDeclarationEntryOrchestrationStage::FoundationalDescribed,
        Some(canonical_digest_token(evidence.attachment_bundle_digest())),
    )
    .with_materialization_tier(foundational_materialization_tier(
        plan.foundational_evidence_profile(),
    ))
}

fn route_foundational_evidence<D, I>(
    checked: &WorthQueryDeclarationRoutePlanChecked<D, I>,
) -> &crate::application::WorthQueryDeclarationFoundationalEvidence<D, I>
where
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
{
    match checked {
        WorthQueryDeclarationRoutePlanChecked::Planned(value) => value.foundational_evidence(),
        WorthQueryDeclarationRoutePlanChecked::Deferred(value) => value.foundational_evidence(),
        WorthQueryDeclarationRoutePlanChecked::Denied(value) => value.foundational_evidence(),
        WorthQueryDeclarationRoutePlanChecked::Failed(value) => value.foundational_evidence(),
    }
}

fn lower_receipt_checked<D, I>(
    step_records: &mut Vec<WorthQueryDeclarationEntryOrchestrationStageRecord>,
    plan: &WorthQueryDeclarationEntryOrchestrationPlan<D, I>,
    route_checked: WorthQueryDeclarationRoutePlanChecked<D, I>,
) -> WorthQueryDeclarationReceiptChecked<D, I>
where
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
{
    append_route_progress(step_records, plan, &route_checked);
    let receipt_checked = worth_query_checked_declaration_receipt_with_materialized_profile(
        WorthQueryDeclarationReceiptInput::route_checked(route_checked),
        &receipt_materialized_profile_for_tier(plan.receipt_materialization_tier()),
        plan.receipt_materialization_tier(),
    );
    append_receipt_progress_or_stop(step_records, plan, &receipt_checked);
    receipt_checked
}

fn lower_envelope_checked<D, I>(
    step_records: &mut Vec<WorthQueryDeclarationEntryOrchestrationStageRecord>,
    plan: &WorthQueryDeclarationEntryOrchestrationPlan<D, I>,
    receipt_checked: WorthQueryDeclarationReceiptChecked<D, I>,
) -> WorthQueryDeclarationEnvelopeChecked<D, I>
where
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
{
    let envelope_checked = worth_query_checked_declaration_envelope(
        WorthQueryDeclarationEnvelopeInput::receipt_checked(receipt_checked),
    );
    append_envelope_stop(step_records, plan, &envelope_checked);
    envelope_checked
}

fn append_route_stop<D, I>(
    step_records: &mut Vec<WorthQueryDeclarationEntryOrchestrationStageRecord>,
    plan: &WorthQueryDeclarationEntryOrchestrationPlan<D, I>,
    checked: &WorthQueryDeclarationRoutePlanChecked<D, I>,
) where
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
{
    let record = match checked {
        WorthQueryDeclarationRoutePlanChecked::Planned(value) => {
            WorthQueryDeclarationEntryOrchestrationStageRecord::terminal_success(
                WorthQueryDeclarationEntryOrchestrationStage::RoutePlanned,
                Some(value.route_plan_digest().to_string()),
            )
        }
        WorthQueryDeclarationRoutePlanChecked::Deferred(value) => {
            WorthQueryDeclarationEntryOrchestrationStageRecord::deferred(
                WorthQueryDeclarationEntryOrchestrationStage::RoutePlanned,
                None,
                value.reason(),
            )
        }
        WorthQueryDeclarationRoutePlanChecked::Denied(value) => {
            WorthQueryDeclarationEntryOrchestrationStageRecord::denied(
                WorthQueryDeclarationEntryOrchestrationStage::RoutePlanned,
                None,
                value.reason(),
            )
        }
        WorthQueryDeclarationRoutePlanChecked::Failed(value) => {
            WorthQueryDeclarationEntryOrchestrationStageRecord::failed(
                WorthQueryDeclarationEntryOrchestrationStage::RoutePlanned,
                None,
                value.reason(),
            )
        }
    };
    step_records.push(record.with_materialization_tier(plan.materialization_tier()));
}

fn append_route_progress<D, I>(
    step_records: &mut Vec<WorthQueryDeclarationEntryOrchestrationStageRecord>,
    plan: &WorthQueryDeclarationEntryOrchestrationPlan<D, I>,
    checked: &WorthQueryDeclarationRoutePlanChecked<D, I>,
) where
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
{
    let digest = match checked {
        WorthQueryDeclarationRoutePlanChecked::Planned(value) => {
            Some(value.route_plan_digest().to_string())
        }
        _ => None,
    };
    step_records.push(
        WorthQueryDeclarationEntryOrchestrationStageRecord::automated(
            WorthQueryDeclarationEntryOrchestrationStage::RoutePlanned,
            digest,
        )
        .with_materialization_tier(plan.materialization_tier()),
    );
}

fn append_receipt_stop<D, I>(
    step_records: &mut Vec<WorthQueryDeclarationEntryOrchestrationStageRecord>,
    plan: &WorthQueryDeclarationEntryOrchestrationPlan<D, I>,
    checked: &WorthQueryDeclarationReceiptChecked<D, I>,
) where
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
{
    let record = match checked {
        WorthQueryDeclarationReceiptChecked::Issued(value) => {
            WorthQueryDeclarationEntryOrchestrationStageRecord::terminal_success(
                WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued,
                Some(format!("{:?}", value.receipt_digest())),
            )
        }
        WorthQueryDeclarationReceiptChecked::Deferred(value) => {
            WorthQueryDeclarationEntryOrchestrationStageRecord::deferred(
                WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued,
                Some(format!("{:?}", value.receipt().receipt_digest())),
                value.reason(),
            )
        }
        WorthQueryDeclarationReceiptChecked::Denied(value) => {
            WorthQueryDeclarationEntryOrchestrationStageRecord::denied(
                WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued,
                Some(format!("{:?}", value.receipt().receipt_digest())),
                value.reason(),
            )
        }
        WorthQueryDeclarationReceiptChecked::Failed(value) => {
            WorthQueryDeclarationEntryOrchestrationStageRecord::failed(
                WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued,
                Some(format!("{:?}", value.receipt().receipt_digest())),
                value.reason(),
            )
        }
    };
    step_records.push(record.with_materialization_tier(plan.receipt_materialization_tier()));
}

fn append_receipt_progress_or_stop<D, I>(
    step_records: &mut Vec<WorthQueryDeclarationEntryOrchestrationStageRecord>,
    plan: &WorthQueryDeclarationEntryOrchestrationPlan<D, I>,
    checked: &WorthQueryDeclarationReceiptChecked<D, I>,
) where
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
{
    let digest = match checked {
        WorthQueryDeclarationReceiptChecked::Issued(value) => {
            Some(format!("{:?}", value.receipt_digest()))
        }
        WorthQueryDeclarationReceiptChecked::Deferred(value) => {
            Some(format!("{:?}", value.receipt().receipt_digest()))
        }
        WorthQueryDeclarationReceiptChecked::Denied(value) => {
            Some(format!("{:?}", value.receipt().receipt_digest()))
        }
        WorthQueryDeclarationReceiptChecked::Failed(value) => {
            Some(format!("{:?}", value.receipt().receipt_digest()))
        }
    };
    step_records.push(
        WorthQueryDeclarationEntryOrchestrationStageRecord::automated(
            WorthQueryDeclarationEntryOrchestrationStage::ReceiptIssued,
            digest,
        )
        .with_materialization_tier(plan.receipt_materialization_tier()),
    );
}

fn append_envelope_stop<D, I>(
    step_records: &mut Vec<WorthQueryDeclarationEntryOrchestrationStageRecord>,
    plan: &WorthQueryDeclarationEntryOrchestrationPlan<D, I>,
    checked: &WorthQueryDeclarationEnvelopeChecked<D, I>,
) where
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
{
    let record = match checked {
        WorthQueryDeclarationEnvelopeChecked::Enveloped(value) => {
            WorthQueryDeclarationEntryOrchestrationStageRecord::terminal_success(
                WorthQueryDeclarationEntryOrchestrationStage::EnvelopeConstructed,
                Some(format!("{:?}", value.envelope_digest())),
            )
        }
        WorthQueryDeclarationEnvelopeChecked::Deferred(value) => {
            WorthQueryDeclarationEntryOrchestrationStageRecord::deferred(
                WorthQueryDeclarationEntryOrchestrationStage::EnvelopeConstructed,
                Some(format!("{:?}", value.envelope().envelope_digest())),
                value.reason(),
            )
        }
        WorthQueryDeclarationEnvelopeChecked::Denied(value) => {
            WorthQueryDeclarationEntryOrchestrationStageRecord::denied(
                WorthQueryDeclarationEntryOrchestrationStage::EnvelopeConstructed,
                Some(format!("{:?}", value.envelope().envelope_digest())),
                value.reason(),
            )
        }
        WorthQueryDeclarationEnvelopeChecked::Failed(value) => {
            WorthQueryDeclarationEntryOrchestrationStageRecord::failed(
                WorthQueryDeclarationEntryOrchestrationStage::EnvelopeConstructed,
                Some(format!("{:?}", value.envelope().envelope_digest())),
                value.reason(),
            )
        }
    };
    step_records.push(record.with_materialization_tier(plan.envelope_materialization_tier()));
}
