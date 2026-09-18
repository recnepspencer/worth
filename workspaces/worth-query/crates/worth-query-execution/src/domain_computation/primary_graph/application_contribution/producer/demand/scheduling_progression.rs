use super::super::{
    schedule_output_producer, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQuerySelectedApplicationProducer,
};
use super::bridge_denial::bridge_denial;
use super::progression::denial;
use crate::domain_computation::primary_graph::{
    application_output_demand::WorthQueryOutputSchedulingResult, WorthQueryObservedSource,
    WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema + 'static,
{
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn schedule_selected_output_producer<
        Query,
    >(
        &self,
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
            || truth.signal_basis(),
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
        let mut identity_bytes = [0_u8; 8];
        identity_bytes.copy_from_slice(&observed_source.query_identity.as_bytes()[..8]);
        let query_identity = u64::from_le_bytes(identity_bytes);
        let bridge = self.bridge.conditional();
        let decision = match schedule_output_producer(
            &bridge,
            route,
            &truth,
            signal_basis,
            &observed_source.query_identifier,
            query_identity,
            &selected.identity,
            attempt,
        ) {
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
