use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::bridge_denial::{bridge_denial, bridge_denial_kind};
use super::progression::denial;
use super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};
use crate::domain_computation::primary_graph::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(super) fn evaluate_current_output_readiness(
        &self,
        producer_identity: &str,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        delivery: Option<&worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>,
    ) -> Result<
        crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence,
        WorthQueryOutputDemandDenial,
    >{
        self.evaluate_current_output_readiness_core(producer_identity, receipt, delivery, None)
    }

    /// The required Fresh continuation uses the actual newly published head.
    /// Query routing and projection share its original source admission; the
    /// Bridge/Signal conditional keeps its installed serial admission class.
    pub(super) fn evaluate_current_output_readiness_admitted(
        &self,
        producer_identity: &str,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        delivery: Option<&worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>,
        request: &WorthQueryRequestScope,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence,
        WorthQueryOutputDemandDenial,
    >{
        self.evaluate_current_output_readiness_core(
            producer_identity,
            receipt,
            delivery,
            Some((request, admission)),
        )
    }

    fn evaluate_current_output_readiness_core(
        &self,
        producer_identity: &str,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        delivery: Option<&worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>,
        mut admitted: Option<(&WorthQueryRequestScope, &mut InvalidationEditAdmission)>,
    ) -> Result<
        crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence,
        WorthQueryOutputDemandDenial,
    >{
        if let Some((request, admission)) = admitted.as_mut() {
            admission
                .charge_external_work(4)
                .map_err(readiness_resource_denial)?;
            admit_selected_readiness_entry(
                producer_identity,
                self.output_readiness_routes.len(),
                self.installed_producers.entries.len(),
                admission,
            )?;
            check_readiness_request(request, admission)?;
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
        let source_record = producer.executor.readiness_record(
            receipt,
            admitted.as_mut().map(|(_, admission)| &mut **admission),
        )?;
        #[cfg(feature = "test-primary-graph-faults")]
        let _held_world_observations = if self.primary_provider.take_readiness_snapshot_pressure() {
            Some(self.hold_world_snapshot_pressure_for_test(receipt))
        } else {
            None
        };
        let selected = self
            .on_branch(receipt.product_branch())
            .select()
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
        let truth = if let Some((request, admission)) = admitted.as_mut() {
            check_readiness_request(request, admission)?;
            crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalTruthBasis::from_selected_admitted(selected, admission)
                .map_err(readiness_resource_denial)?
        } else {
            crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalTruthBasis::from_selected(selected)
        };
        if let Some((_, admission)) = admitted.as_mut() {
            let result_work = std::mem::size_of::<crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence>()
                .checked_add(16)
                .ok_or_else(readiness_work_denial)?;
            admission
                .charge_external_work(
                    u64::try_from(result_work).map_err(|_| readiness_work_denial())?,
                )
                .map_err(readiness_resource_denial)?;
        }
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
            &bridge,
            &route.lowering,
            &truth,
            producer_identity,
            source_record,
            attempt,
            attempt,
        );
        if let Some((request, admission)) = admitted.as_mut() {
            check_readiness_request(request, admission)?;
        }
        let execution = execution.map_err(|error| {
            if admitted.is_some() {
                WorthQueryOutputDemandDenial::new(bridge_denial_kind(&error), "")
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

fn admit_selected_readiness_entry(
    producer: &str,
    route_count: usize,
    installed_count: usize,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    // Both B-tree searches inspect at most the selected name's initialized
    // bytes per comparison. Each node holds at most eleven selected keys.
    let selected_width = producer
        .len()
        .checked_add(1)
        .ok_or_else(readiness_work_denial)?;
    let mut work = 0usize;
    for count in [route_count, installed_count] {
        let levels = usize::BITS as usize - count.max(1).leading_zeros() as usize;
        work = count
            .min(11)
            .checked_mul(levels)
            .and_then(|comparisons| comparisons.checked_mul(selected_width))
            .and_then(|bound| work.checked_add(bound))
            .ok_or_else(readiness_work_denial)?;
    }
    // Selected semantic denials carry their original kind with an empty
    // subject, so no variable diagnostic String is allocated here.
    work = work.checked_add(24).ok_or_else(readiness_work_denial)?;
    admission
        .charge_external_work(u64::try_from(work).map_err(|_| readiness_work_denial())?)
        .map_err(readiness_resource_denial)
}

fn check_readiness_request(
    request: &WorthQueryRequestScope,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    admission
        .charge_external_work(1)
        .map_err(readiness_resource_denial)?;
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

fn readiness_work_denial() -> WorthQueryOutputDemandDenial {
    denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "")
}

fn readiness_resource_denial(stop: CompanionPreflightStop) -> WorthQueryOutputDemandDenial {
    let kind = match stop {
        CompanionPreflightStop::WorkExhausted { .. }
        | CompanionPreflightStop::WorkCounterOverflow => {
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        }
        _ => WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
    };
    denial(kind, "")
}
