use super::super::schedule_output_producer;
use super::super::schedule_output_producer_on_selected;
use super::super::WorthQueryOutputDemandDenial;
use super::super::WorthQueryOutputDemandDenialKind;
use super::super::WorthQuerySelectedApplicationProducer;
use super::bridge_denial::bridge_denial;
use super::bridge_denial::is_temporarily_deferred;
use super::progression::denial;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputSchedulingResult;
use crate::domain_computation::primary_graph::conditional_operation::WorthQuerySelectedSignalProjections;
use crate::domain_computation::primary_graph::product_operation::SharedSelectedProductOperation;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use crate::domain_computation::primary_graph::WorthQueryObservedSource;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use std::sync::Arc;
use worth_runtime_bridge::facade::BridgeConditionalDenial;
use worth_runtime_bridge::facade::BridgeInstalledConditionalLowering;
use worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly;

const OUTPUT_SCHEDULE_DOMAIN: &[u8] = b"worth-query:output-schedule:v1";

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema + 'static,
{
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn schedule_selected_output_producer<
        Query,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        selected: &WorthQuerySelectedApplicationProducer,
        branch: crate::basis::WorthQueryProductBranch,
        observed_source: &WorthQueryObservedSource<Query>,
        performed_source: Option<
            &crate::domain_computation::primary_graph::application_output_demand::WorthQueryPerformedOutputDemandSource,
        >,
    ) -> Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial> {
        let route = self
            .output_producer_routes
            .get(&selected.identity)
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                    format!("{}: Signal route was not installed", selected.identity),
                )
            })?;
        let selected_branch = match self.on_branch(branch).select_with_request(execution) {
            Ok(selected) => selected,
            Err(error @ crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(_)) => {
                return Err(WorthQueryOutputDemandDenial::product_selection(
                    error,
                    format!("{}: scheduling product selection", selected.identity),
                ))
            }
            Err(crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(kind))
                if kind.is_transient() =>
            {
                return Ok(WorthQueryOutputSchedulingResult::Deferred)
            }
            Err(error @ crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(_)) => {
                return Err(WorthQueryOutputDemandDenial::product_selection(
                    error,
                    format!("{}: scheduling product selection", selected.identity),
                ))
            }
        };
        if performed_source.is_some_and(|source| {
            selected_branch
                .product()
                .observation()
                .lifecycle_incarnation()
                != source.observation.lifecycle_incarnation()
        }) {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "performed source and current scheduling branch belong to different occurrences",
            ));
        }
        let truth = crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalTruthBasis::from_selected(selected_branch);
        self.schedule_selected_output_producer_with_truth(
            selected,
            observed_source,
            performed_source,
            route,
            truth.signal_basis(),
            |bridge, route, signal_basis, query_identity, attempt| {
                schedule_output_producer(
                    phase,
                    bridge,
                    route,
                    &truth,
                    signal_basis,
                    &observed_source.query_identifier,
                    query_identity,
                    &selected.identity,
                    attempt,
                )
            },
        )
    }

    /// Use the already selected Product and its owner-issued bridge identity
    /// projections for a required wave. The ordinary and selected entries
    /// share route, attempt, performed-source and Signal classification.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn schedule_selected_output_producer_on_selected<
        Query,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        selected: &WorthQuerySelectedApplicationProducer,
        branch: crate::basis::WorthQueryProductBranch,
        observed_source: &WorthQueryObservedSource<Query>,
        performed_source: Option<
            &crate::domain_computation::primary_graph::application_output_demand::WorthQueryPerformedOutputDemandSource,
        >,
        shared: &SharedSelectedProductOperation<'_, Schema>,
    ) -> Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial> {
        // Inspecting the selected Product, route cardinality and borrowed
        // field widths is itself request work; claim it before those reads.

        let selected_product = shared.selected();
        let observation = selected_product.product().observation();

        if !std::ptr::eq(self, selected_product.application())
            || observation.lifecycle_incarnation() != branch.occurrence()
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "selected scheduling Product belongs to another wave",
            ));
        }
        let route = self
            .output_producer_routes
            .get(&selected.identity)
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                    format!("{}: Signal route was not installed", selected.identity),
                )
            })?;
        if performed_source.is_some_and(|source| {
            observation.lifecycle_incarnation() != source.observation.lifecycle_incarnation()
        }) {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "performed source and selected scheduling Product belong to different occurrences",
            ));
        }
        let truth = WorthQuerySelectedSignalProjections::prepare(shared)
            .map_err(schedule_admission_denial)?;
        self.schedule_selected_output_producer_with_truth(
            selected,
            observed_source,
            performed_source,
            route,
            selected_product.product().signal_basis(),
            |bridge, route, signal_basis, query_identity, attempt| {
                schedule_output_producer_on_selected(
                    phase,
                    bridge,
                    route,
                    &truth,
                    signal_basis,
                    &observed_source.query_identifier,
                    query_identity,
                    &selected.identity,
                    attempt,
                )
            },
        )
    }

    fn schedule_selected_output_producer_with_truth<Query>(
        &self,
        selected: &WorthQuerySelectedApplicationProducer,
        observed_source: &WorthQueryObservedSource<Query>,
        performed_source: Option<
            &crate::domain_computation::primary_graph::application_output_demand::WorthQueryPerformedOutputDemandSource,
        >,
        route: &Arc<BridgeInstalledConditionalLowering>,
        current_signal_basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
        execute_signal: impl FnOnce(
            &BridgeSealedRuntimeAssembly,
            &Arc<BridgeInstalledConditionalLowering>,
            &worth_signal::facade::branch::AdmittedSignalBranchBasis,
            u64,
            u64,
        ) -> Result<
            crate::domain_computation::primary_graph::WorthQueryConditionalSignalDecision,
            BridgeConditionalDenial,
        >,
    ) -> Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial> {
        if performed_source.is_some_and(|source| {
            let publication = source.receipt.committed_product_publication();
            publication.product_branch() != source.change.product_branch_identity()
                || publication.composite_commit() != source.change.product_commit()
        }) {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "performed source custody drifted from its committed publication",
            ));
        }
        let signal_basis = performed_source.map_or_else(
            || current_signal_basis,
            |source| source.change.signal_basis(),
        );
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
                    format!("{}: Signal attempt identity exhausted", selected.identity),
                )
            })?;
        let mut identity = sha2::Sha256::new();
        use sha2::Digest;
        identity.update(OUTPUT_SCHEDULE_DOMAIN);
        identity.update(observed_source.query_identity.as_bytes());
        identity.update(observed_source.parameter_binding_identity.bytes());
        let identity = identity.finalize();
        let mut identity_bytes = [0_u8; 8];
        identity_bytes.copy_from_slice(&identity[..8]);
        let query_identity = u64::from_le_bytes(identity_bytes);
        let bridge = self.bridge.conditional();
        let decision = match execute_signal(&bridge, route, signal_basis, query_identity, attempt) {
            Ok(decision) => decision,
            Err(error) => {
                return scheduling_refusal(&selected.identity, error);
            }
        };
        use crate::domain_computation::primary_graph::WorthQueryConditionalSignalDecision as Decision;
        match decision {
            Decision::Eligible => Ok(WorthQueryOutputSchedulingResult::Scheduled),
            Decision::Deferred => Ok(WorthQueryOutputSchedulingResult::Deferred),
            Decision::DependencyUnchanged | Decision::RevertedClean | Decision::Suppressed => {
                Ok(WorthQueryOutputSchedulingResult::NoEffect(denial(
                    WorthQueryOutputDemandDenialKind::NoEffect,
                    format!("{}: Signal returned {decision:?}", selected.identity),
                )))
            }
        }
    }
}

fn schedule_admission_denial(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    crate::domain_computation::primary_graph::output_reuse::retained_fact_denial(
        crate::domain_computation::primary_graph::output_reuse::preflight_stop(stop),
        "selected demand projection unavailable",
    )
}

fn scheduling_refusal(
    producer_identity: &str,
    error: BridgeConditionalDenial,
) -> Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial> {
    if is_temporarily_deferred(&error.kind()) {
        return Ok(WorthQueryOutputSchedulingResult::Deferred);
    }
    Err(bridge_denial(producer_identity, error))
}

#[cfg(test)]
mod tests;
