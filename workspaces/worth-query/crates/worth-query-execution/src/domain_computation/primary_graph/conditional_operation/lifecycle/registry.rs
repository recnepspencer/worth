use std::{collections::BTreeMap, sync::Arc};

use worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly;

use super::{WorthQueryConditionalOperationCell, WorthQueryInstalledConditionalOperation};
use crate::domain_computation::primary_graph::conditional_operation::installation::{
    ConditionalClockLease, WorthQueryConditionalRuntimeInstallationDenial,
};

pub(in crate::domain_computation::primary_graph) struct WorthQueryConditionalOperationRegistry<
    Schema,
> {
    installed: Arc<BTreeMap<String, WorthQueryConditionalOperationCell<Schema>>>,
    marker: std::marker::PhantomData<fn() -> Schema>,
}

impl<Schema> Default for WorthQueryConditionalOperationRegistry<Schema> {
    fn default() -> Self {
        Self {
            installed: Arc::new(BTreeMap::new()),
            marker: std::marker::PhantomData,
        }
    }
}

impl<Schema> WorthQueryConditionalOperationRegistry<Schema> {
    /// Snapshot only routing and managed ownership; no operation callback runs
    /// while the application registry is borrowed.
    pub(in crate::domain_computation::primary_graph) fn snapshot(&self) -> Self {
        Self {
            installed: Arc::clone(&self.installed),
            marker: std::marker::PhantomData,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn lifecycle_probe(
        &self,
        bridge: worth_runtime_bridge::facade::BridgeConditionalRuntimeLifecycleProbe,
    ) -> super::super::WorthQueryConditionalRuntimeLifecycleProbe {
        super::super::WorthQueryConditionalRuntimeLifecycleProbe::from_resources(
            self.installed
                .values()
                .map(|operation| operation.lock_operation().lifecycle_resources()),
            bridge,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn len(&self) -> usize {
        self.installed.len()
    }

    pub(in crate::domain_computation::primary_graph) fn retained_resource_counts(
        &self,
    ) -> super::WorthQueryConditionalRetainedResourceCounts {
        self.installed.values().fold(
            super::WorthQueryConditionalRetainedResourceCounts::default(),
            |mut total, operation| {
                let counts = operation.lock_operation().retained_resource_counts();
                total.wakes = total.wakes.saturating_add(counts.wakes);
                total.intents = total.intents.saturating_add(counts.intents);
                total.attempts = total.attempts.saturating_add(counts.attempts);
                total.direct_deliveries = total
                    .direct_deliveries
                    .saturating_add(counts.direct_deliveries);
                total
            },
        )
    }

    pub(in crate::domain_computation::primary_graph) fn installation_canonical_work(
        &self,
    ) -> worth_query_installation::facade::WorthQueryCanonicalWorkEvidence {
        self.installed.values().fold(
            worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
            |total, operation| {
                total.combine(operation.lock_operation().installation_canonical_work())
            },
        )
    }

    pub(in crate::domain_computation::primary_graph) fn reconstruction_work(
        &self,
    ) -> crate::domain_computation::primary_graph::conditional_operation::temporal_reconstruction::WorthQueryTemporalReconstructionWork{
        self.installed
            .values()
            .fold(Default::default(), |mut total, operation| {
                let work = operation.lock_operation().reconstruction_work();
                total.examined_candidates = total
                    .examined_candidates
                    .saturating_add(work.examined_candidates);
                total.projected_records = total
                    .projected_records
                    .saturating_add(work.projected_records);
                total.projected_fields =
                    total.projected_fields.saturating_add(work.projected_fields);
                total.total_work_units =
                    total.total_work_units.saturating_add(work.total_work_units);
                total
            })
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_derived_runtime_reinstallation(
        &self,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        bridge: &mut worth_runtime_bridge::facade::BridgePreparedConditionalReconstitution,
        product: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<
        BTreeMap<String, super::WorthQueryPreparedConditionalRuntimeBinding>,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        self.installed
            .iter()
            .map(|(identity, operation)| {
                operation
                    .lock_operation()
                    .prepare_derived_runtime_reinstallation(runtime, bridge, product)
                    .map(|prepared| (identity.clone(), prepared))
            })
            .collect()
    }

    pub(in crate::domain_computation::primary_graph) fn apply_derived_runtime_reinstallation(
        &self,
        mut prepared: BTreeMap<String, super::WorthQueryPreparedConditionalRuntimeBinding>,
    ) {
        for (identity, operation) in self.installed.iter() {
            operation
                .lock_operation()
                .apply_derived_runtime_reinstallation(
                    prepared
                        .remove(identity)
                        .expect("prepared conditional inventory matches installed registry"),
                );
        }
        assert!(prepared.is_empty());
    }

    pub(in crate::domain_computation::primary_graph) fn reconcile_prepared_runtime_reinstallation(
        &mut self,
        bridge: &mut worth_runtime_bridge::facade::BridgePreparedConditionalReconstitution,
        prepared: &mut BTreeMap<String, super::WorthQueryPreparedConditionalRuntimeBinding>,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        for (identity, operation) in self.installed.iter() {
            operation
                .lock_operation()
                .reconcile_prepared_runtime_reinstallation(
                    bridge,
                    prepared
                        .get_mut(identity)
                        .expect("prepared conditional inventory matches installed registry"),
                )?;
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::conditional_operation) fn install(
        &mut self,
        operation: Box<dyn WorthQueryInstalledConditionalOperation<Schema>>,
    ) -> Result<(), ()> {
        let identity = operation.binding_identity().to_string();
        if self.installed.contains_key(&identity) {
            return Err(());
        }
        Arc::make_mut(&mut self.installed)
            .insert(identity, WorthQueryConditionalOperationCell::new(operation));
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn admit_clock(
        &self,
        identity: &str,
        lease: &Arc<ConditionalClockLease>,
    ) -> Option<WorthQueryConditionalOperationCell<Schema>> {
        self.installed
            .get(identity)
            .filter(|operation| operation.matches_clock_lease(lease))
            .cloned()
    }

    pub(in crate::domain_computation::primary_graph) fn reconstruct_all(
        &mut self,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
            Schema,
        >,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        for cell in self.installed.values() {
            cell.lock_operation().reconstruct(runtime)?;
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn reconcile_all(
        &mut self,
        bridge: &mut BridgeSealedRuntimeAssembly,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        for operation in self.installed.values() {
            operation
                .lock_operation()
                .reconcile_reconstruction(bridge)?;
        }
        Ok(())
    }
}
