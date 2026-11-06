use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use crate::domain_computation::primary_graph::application_output_demand::denial::bridge_conditional::bridge_denial;
use crate::domain_computation::primary_graph::application_output_demand::denial::bridge_conditional::bridge_denial_kind;
use super::progression::denial;
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind;

pub(super) trait ReadinessParentOps<Schema>: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> where
    Schema: ApplicationSchema + 'static, {
fn evaluate_current_output_readiness(
        &self,
execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        producer_identity: &str,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        delivery: Option<&worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>,
        ready: &crate::domain_computation::primary_graph::application_output_demand::PreparedReadyBacking,
    ) -> Result<
        crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence,
        WorthQueryOutputDemandDenial,
    >{
        self.evaluate_current_output_readiness_core(
            execution,
            phase,
            producer_identity,
            receipt,
            delivery,
            ready,
            None,
        )
    }

fn evaluate_current_output_readiness_admitted(
        &self,
execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        producer_identity: &str,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        delivery: Option<&worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>,
        ready: &crate::domain_computation::primary_graph::application_output_demand::PreparedReadyBacking,
        request: &WorthQueryRequestScope,

    ) -> Result<
        crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence,
        WorthQueryOutputDemandDenial,
    >{
        self.evaluate_current_output_readiness_core(
            execution,
            phase,
            producer_identity,
            receipt,
            delivery,
            ready,
            Some(request),
        )
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> + ?Sized> ReadinessParentOps<Schema> for HostRuntime where
    Schema: ApplicationSchema + 'static, {}

trait ReadinessOwnOps<Schema>: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> where
    Schema: ApplicationSchema + 'static, {
fn evaluate_current_output_readiness_core(
        &self,
execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        producer_identity: &str,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        delivery: Option<&worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>,
        ready: &crate::domain_computation::primary_graph::application_output_demand::PreparedReadyBacking,

        admitted: Option<&WorthQueryRequestScope>,
    ) -> Result<
        crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence,
        WorthQueryOutputDemandDenial,
    >{
        if let Some(request) = admitted.as_ref() {
            check_readiness_request(request)?;
        }
        if self
            .primary_provider()
            .take_failed_output_readiness_evaluation()
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::SchedulingDeferred,
                if admitted.is_some() {
                    String::new()
                } else {
                    format!("{producer_identity}: injected transient readiness evaluation failure")
                },
            ));
        }
        let route = self
            .output_production()
            .readiness_routes()
            .get(producer_identity)
            .expect("installation requires one readiness route for every output producer");
        let producer = self
            .output_production()
            .producers()
            .entries
            .get(producer_identity)
            .expect("readiness route must retain its installed producer");
        let source_record = producer.executor.readiness_record(receipt)?;
        let _held_world_observations = if self
            .primary_provider()
            .take_readiness_snapshot_pressure()
        {
            Some(self.hold_world_snapshot_pressure_for_test(receipt.product_branch()))
        } else {
            None
        };
        let selected = self
            .on_branch(receipt.product_branch())
            .select_with_request(execution)
            .map_err(|error| {
                WorthQueryOutputDemandDenial::product_selection(
                    error,
                    if admitted.is_some() {
                        String::new()
                    } else {
                        format!("{producer_identity}: readiness product selection")
                    },
                )
            })?;
        let truth = if let Some(request) = admitted.as_ref() {
            check_readiness_request(request)?;
            crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalTruthBasis::from_selected_admitted(selected)
                .map_err(readiness_resource_denial)?
        } else {
            crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalTruthBasis::from_selected(selected)
        };

        let attempt = {
            let update_counter = self.output_production().next_producer_attempt();
            let mut current = update_counter.load(std::sync::atomic::Ordering::Relaxed);
            loop {
                let Some(updated_value) = current.checked_add(1) else {
                    break Err(current);
                };
                match update_counter.compare_exchange_weak(
                    current,
                    updated_value,
                    std::sync::atomic::Ordering::Relaxed,
                    std::sync::atomic::Ordering::Relaxed,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => current = reloaded_value,
                }
            }
        }
        .map_err(|_| {
            denial(
                WorthQueryOutputDemandDenialKind::SchedulingRejected,
                if admitted.is_some() {
                    String::new()
                } else {
                    format!("{producer_identity}: readiness attempt identity exhausted")
                },
            )
        })?;
        let bridge = self.bridge().conditional();
        let execution = super::super::evaluate_output_readiness(
            phase
                .request_for_owner(truth.owner_identity())
                .expect("readiness uses its admitted runtime"),
            &bridge,
            &route.lowering,
            &truth,
            producer_identity,
            source_record,
            attempt,
            attempt,
        );
        if let Some(request) = admitted.as_ref() {
            check_readiness_request(request)?;
        }
        let execution = execution.map_err(|error| {
            if admitted.is_some() {
                WorthQueryOutputDemandDenial::new(bridge_denial_kind(error.kind()), "")
            } else {
                bridge_denial(producer_identity, error)
            }
        })?;
        let decision =
            crate::domain_computation::primary_graph::conditional_operation::classify_bridge_signal(
                &execution,
            );
        if decision != crate::domain_computation::primary_graph::vocabulary::conditional_operation::classification::WorthQueryConditionalSignalDecision::Eligible {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::SchedulingRejected,
                if admitted.is_some() {
                    String::new()
                } else {
                    format!("{producer_identity}: readiness returned {decision:?}")
                },
            ));
        }
        let evaluation = match self
            .output_production()
            .producer_routes()
            .get(producer_identity)
            .and_then(|route| route.output_comparison.as_ref())
        {
            None => None,
            Some(comparison) => {
                let visit = comparison.evaluate(
                    phase
                        .request_for_owner(truth.owner_identity())
                        .expect("output evaluation uses its admitted runtime"),
                    &bridge,
                    &truth,
                    super::super::output_evaluation::NativeOutputBasis {
                        receipt,
                        readiness: &execution,
                        lowering: &route.lowering,
                        ordinal: route.delivery_dependency_ordinal,
                        layout: &self.provider_graph().layout,
                    },
                    ready,
                    self.output_production().next_producer_attempt(),
                )?;
                if visit.compared_now {
                    self.primary_provider().output_comparison_completed();
                }
                Some(visit.evaluation)
            }
        };
        // A newly completed comparison is already memoized. Any late request
        // interruption must return its Delivered checkpoint, including ordinary demand.
        super::super::output_evaluation::consult(
            &phase
                .request_for_owner(truth.owner_identity())
                .expect("readiness completion uses its admitted runtime"),
        )?;
        if let Some(request) = admitted.as_ref() {
            check_readiness_request(request)?;
        }
        Ok(crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::from_execution(
            delivery,
            &execution,
        ).with_output_evaluation(evaluation))
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> + ?Sized> ReadinessOwnOps<Schema> for HostRuntime where
    Schema: ApplicationSchema + 'static, {}

fn check_readiness_request(
    request: &WorthQueryRequestScope,
) -> Result<(), WorthQueryOutputDemandDenial> {
    match request.interruption() {
        Some(WorthQueryRequestInterruption::Cancelled) => {
            Err(denial(WorthQueryOutputDemandDenialKind::Cancelled, ""))
        }
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => {
            Err(denial(WorthQueryOutputDemandDenialKind::TimedOut, ""))
        }
        None => Ok(()),
    }
}

fn readiness_resource_denial(stop: CompanionPreflightStop) -> WorthQueryOutputDemandDenial {
    let kind = match stop {
        CompanionPreflightStop::WorkCounterOverflow => {
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        }
        _ => WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
    };
    denial(kind, "")
}
