use super::super::{
    schedule_output_producer, schedule_output_producer_on_selected, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind, WorthQuerySelectedApplicationProducer,
};
use super::bridge_denial::bridge_denial;
use super::progression::denial;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use crate::domain_computation::primary_graph::{
    application_output_demand::WorthQueryOutputSchedulingResult,
    conditional_operation::WorthQuerySelectedSignalProjections,
    output_lineage::invalidation::InvalidationEditAdmission,
    product_operation::SharedSelectedProductOperation, WorthQueryObservedSource,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use std::sync::Arc;
use worth_runtime_bridge::facade::{
    BridgeConditionalDenial, BridgeInstalledConditionalLowering, BridgeSealedRuntimeAssembly,
};

const OUTPUT_SCHEDULE_DOMAIN: &[u8] = b"worth-query:output-schedule:v1";

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema + 'static,
{
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn schedule_selected_output_producer<
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
        let selected_branch = match self.on_branch(branch).select() {
            Ok(selected) => selected,
            Err(error) if error.is_transient() => {
                return Ok(WorthQueryOutputSchedulingResult::Deferred)
            }
            Err(error) => {
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial> {
        // Inspecting the selected Product, route cardinality and borrowed
        // field widths is itself request work; claim it before those reads.
        admission
            .charge_external_work(8)
            .map_err(|_| schedule_work_denial())?;
        let selected_product = shared.selected();
        let observation = selected_product.product().observation();
        let name_bytes = observation.branch_identity().name().as_str().len();
        let performed_name_bytes = performed_source.map_or(0, |source| {
            source
                .change
                .product_branch_identity()
                .name()
                .as_str()
                .len()
        });
        let route_count = self.output_producer_routes.len();
        let levels = usize::BITS as usize - route_count.max(1).leading_zeros() as usize;
        let comparisons = route_count
            .min(11)
            .checked_mul(levels)
            .ok_or_else(schedule_work_denial)?;
        let hash_input = OUTPUT_SCHEDULE_DOMAIN
            .len()
            .checked_add(observed_source.query_identity.as_bytes().len())
            .and_then(|bytes| {
                bytes.checked_add(observed_source.parameter_binding_identity.bytes().len())
            })
            .ok_or_else(schedule_work_denial)?;
        let hash_blocks = hash_input
            .checked_add(9)
            .and_then(|bytes| bytes.checked_add(63))
            .map(|bytes| bytes / 64)
            .ok_or_else(schedule_work_denial)?;
        let hash_work = hash_input
            .checked_add(hash_blocks)
            // SHA finalization writes 32 bytes. The attempt key initializes
            // eight bytes and then copies eight digest bytes into them.
            // One finalization and one checked-attempt visit are separate.
            .and_then(|work| work.checked_add(32 + 8 + 8 + 2))
            .ok_or_else(schedule_work_denial)?;
        let route_work = selected
            .identity
            .len()
            .checked_add(3)
            .and_then(|width| comparisons.checked_mul(width))
            .and_then(|work| work.checked_add(name_bytes))
            .and_then(|work| work.checked_add(performed_name_bytes))
            .and_then(|work| work.checked_add(hash_work))
            // Runtime/occurrence, performed publication and route/Signal
            // owner visits after the selected tree search.
            .and_then(|work| work.checked_add(12))
            .ok_or_else(schedule_work_denial)?;
        admission
            .charge_external_work(u64::try_from(route_work).map_err(|_| schedule_work_denial())?)
            .map_err(|_| schedule_work_denial())?;
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
        let truth = WorthQuerySelectedSignalProjections::prepare(shared, admission)
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
                let denial = bridge_denial(&selected.identity, error);
                if denial.kind() == WorthQueryOutputDemandDenialKind::SchedulingDeferred {
                    return Ok(WorthQueryOutputSchedulingResult::Deferred);
                }
                return Err(denial);
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

fn schedule_work_denial() -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "selected required scheduling exceeds request work",
    )
}

fn schedule_admission_denial(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    use worth_relational::facade::mvcc::CompanionPreflightStop;
    match stop {
        CompanionPreflightStop::WorkExhausted { .. }
        | CompanionPreflightStop::WorkCounterOverflow => schedule_work_denial(),
        _ => denial(
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
            "selected required scheduling projection exceeds preparation memory",
        ),
    }
}
