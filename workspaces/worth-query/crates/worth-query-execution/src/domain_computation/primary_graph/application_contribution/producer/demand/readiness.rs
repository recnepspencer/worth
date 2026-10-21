use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::bridge_denial::bridge_denial;
use super::bridge_denial::bridge_denial_kind;
use super::progression::denial;
use super::WorthQueryOutputDemandDenial;
use super::WorthQueryOutputDemandDenialKind;

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(super) fn evaluate_current_output_readiness(
        &self,
execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        producer_identity: &str,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        delivery: Option<&worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>,
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
            None,
        )
    }

    /// The required Fresh continuation uses the actual newly published head.
    /// Query routing and projection share its original source admission; the
    /// Bridge/Signal conditional keeps its installed serial admission class.
    pub(super) fn evaluate_current_output_readiness_admitted(
        &self,
execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        producer_identity: &str,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        delivery: Option<&worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>,
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
            Some(request),
        )
    }

    fn evaluate_current_output_readiness_core(
        &self,
execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        producer_identity: &str,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        delivery: Option<&worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>,

        admitted: Option<&WorthQueryRequestScope>,
    ) -> Result<
        crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence,
        WorthQueryOutputDemandDenial,
    >{
        if let Some(request) = admitted.as_ref() {
            check_readiness_request(request)?;
        }
        if self
            .primary_provider
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
            .output_readiness_routes
            .get(producer_identity)
            .expect("installation requires one readiness route for every output producer");
        let producer = self
            .installed_producers
            .entries
            .get(producer_identity)
            .expect("readiness route must retain its installed producer");
        let source_record = producer.executor.readiness_record(receipt)?;
        let _held_world_observations = if self.primary_provider.take_readiness_snapshot_pressure() {
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

        let attempt = self
            .next_output_producer_attempt
            .fetch_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |current| current.checked_add(1),
            )
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
        let bridge = self.bridge.conditional();
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
        if decision != crate::domain_computation::primary_graph::WorthQueryConditionalSignalDecision::Eligible {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::SchedulingRejected,
                if admitted.is_some() {
                    String::new()
                } else {
                    format!("{producer_identity}: readiness returned {decision:?}")
                },
            ));
        }
        Ok(crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::from_execution(
            delivery,
            &execution,
        ))
    }
}

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
        CompanionPreflightStop::WorkExhausted { .. }
        | CompanionPreflightStop::WorkExhaustedWithoutRequirement { .. }
        | CompanionPreflightStop::WorkCounterOverflow => {
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        }
        _ => WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
    };
    denial(kind, "")
}
