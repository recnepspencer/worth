mod conditional_evaluation;
pub(crate) use conditional_evaluation::WorthQueryExecutedConditional;

use worth_query_execution::facade::integration::WorthQueryProductRuntime;
use worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly;

/// The sealed conditional service and the World assembled from its owner services.
pub(super) struct WorthQueryInstalledProduct {
    pub(super) world: WorthQueryProductRuntime,
    pub(super) conditional: BridgeSealedRuntimeAssembly,
    conditional_evaluations: conditional_evaluation::WorthQueryConditionalEvaluationRegistry,
}

impl WorthQueryInstalledProduct {
    pub(super) fn conditional_resource_observation(
        &self,
    ) -> Option<super::WorthQueryConditionalEvaluationResourceObservation> {
        let query = self.conditional_evaluations.observe();
        let signal = self
            .conditional
            .conditional_lifecycle_probe()
            .signal_conditional_retention()?;
        Some(super::WorthQueryConditionalEvaluationResourceObservation {
            query_retained_entries: query.retained_entries,
            query_retained_bytes: query.retained_bytes,
            query_cache_hits: query.hits,
            query_cache_misses: query.misses,
            query_cache_evictions: query.evictions,
            signal_retained_slots: signal.retained_slots(),
            signal_retained_bytes: signal.retained_bytes(),
        })
    }

    pub(super) fn validate_selected_source(
        &self,
        selected: &worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
        snapshot: Option<&worth_runtime_bridge::facade::TruthSnapshotIdentity>,
    ) -> Result<
        (),
        (
            worth_runtime_bridge::facade::BridgeConditionalDenialKind,
            &'static str,
        ),
    > {
        if !self
            .world
            .integration_owns_product_branch(selected.product_branch())
        {
            return Err((
                worth_runtime_bridge::facade::BridgeConditionalDenialKind::GraphAuthorityMismatch,
                "selected product belongs to another World owner",
            ));
        }
        if snapshot.is_some_and(|snapshot| snapshot != selected.bridge_snapshot_identity()) {
            return Err((
                worth_runtime_bridge::facade::BridgeConditionalDenialKind::SnapshotMismatch,
                "conditional source differs from the selected product observation",
            ));
        }
        Ok(())
    }

    pub(super) fn install(
        backend: &dyn super::WorthQueryRuntimeBackend,
        mut conditional: BridgeSealedRuntimeAssembly,
        cache_budget: super::WorthQueryConditionalEvaluationCacheBudget,
        product_world_resources: worth_query_execution::facade::integration::WorthQueryProductWorldResources,
    ) -> Result<Self, super::WorthQueryRuntimeError> {
        let source = backend.prepare_product_source().map_err(|denial| {
            super::WorthQueryRuntimeError::InvariantRegistration {
                stage: "product_source_installation",
                message: denial.to_string(),
            }
        })?;
        let world =
            WorthQueryProductRuntime::install(source, &mut conditional, product_world_resources)
                .map_err(
                    |denial| super::WorthQueryRuntimeError::InvariantRegistration {
                        stage: "product_world_installation",
                        message: denial.detail().to_string(),
                    },
                )?;
        let conditional_evaluations =
            conditional_evaluation::WorthQueryConditionalEvaluationRegistry::new(cache_budget)
                .map_err(
                    |denial| super::WorthQueryRuntimeError::InvariantRegistration {
                        stage: "conditional_evaluation_resource_installation",
                        message: denial.to_string(),
                    },
                )?;
        Ok(Self {
            world,
            conditional,
            conditional_evaluations,
        })
    }
}
