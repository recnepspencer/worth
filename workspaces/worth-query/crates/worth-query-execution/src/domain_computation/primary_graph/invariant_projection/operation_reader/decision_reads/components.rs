//! Original aggregate bodies bound to this projection's admission and contracts.
use super::structural_walks::WalkRecordDenial as Denial;
use super::*;
use crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity as Admission;
use crate::domain_computation::primary_graph::component_selection::{
    ComponentSelection, ComponentSelectionKey,
};
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::MapKernelContext;
use worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts as Contracts;
use worth_relational::facade::snapshots::SnapshotHandle;

struct Issued {
    body: Arc<ComponentSelection>,
    admission: Admission,
    contracts: Arc<Contracts>,
    cookie: u64,
}
#[derive(Default)]
pub(super) struct IssuedComponentValues {
    values: BTreeMap<ComponentSelectionKey, Issued>,
}
impl IssuedComponentValues {
    pub(super) fn get(
        &self,
        key: &ComponentSelectionKey,
        admission: Admission,
        contracts: &Arc<Contracts>,
        source: &SnapshotHandle,
    ) -> Option<&Arc<ComponentSelection>> {
        let issued = self.values.get(key)?;
        (issued.admission == admission
            && Arc::ptr_eq(&issued.contracts, contracts)
            && issued.body.source() == source)
            .then_some(&issued.body)
    }
}
impl DecisionReads {
    pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader) fn fail_component(
        &mut self,
    ) {
        self.failed_component = true;
    }
    pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader) fn record_component(
        &mut self,
        body: &Arc<ComponentSelection>,
        admission: Admission,
        contracts: Arc<Contracts>,
        cookie: u64,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Denial> {
        crate::domain_computation::primary_graph::aggregate_observation::AggregateObservation::checkpoint(context)?;
        if let Some(prior) = self.components.values.get(body.key()) {
            if prior.admission != admission
                || !Arc::ptr_eq(&prior.contracts, &contracts)
                || prior.cookie != cookie
                || prior.body.source() != body.source()
                || !prior
                    .body
                    .same_truth_with_poll(body, || crate::domain_computation::primary_graph::aggregate_observation::AggregateObservation::checkpoint(context))?
            {
                return Err(Denial::Conflict);
            }
        } else {
            self.components.values.insert(
                body.key().clone(),
                Issued {
                    body: Arc::clone(body),
                    admission,
                    contracts,
                    cookie,
                },
            );
        }
        self.record(
            WorthQueryApplicationFactKey::ComponentSelection(body.key().clone()),
            DecisionReadOutcome::Observed,
        );
        Ok(())
    }
}
