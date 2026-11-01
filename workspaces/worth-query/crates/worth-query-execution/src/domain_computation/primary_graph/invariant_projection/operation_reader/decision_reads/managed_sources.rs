//! Original Source bodies stay with the admitted reader; keys cannot reissue them.
use super::*;
use crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity as Admission;
use crate::domain_computation::primary_graph::{
    application_attempt::{
        ManagedSourceRead, ManagedSourceReadKey, SourceReadAuthorization,
        SourceReadAuthorizationKey,
    },
    source_facts::SourceFactsAllocation,
};
use std::{collections::BTreeMap, sync::Arc};
use worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts as Contracts;
use worth_relational::facade::snapshots::SnapshotHandle;

struct Issued {
    body: Arc<ManagedSourceRead>,
    admission: Admission,
    contracts: Arc<Contracts>,
    source: SnapshotHandle,
}
struct IssuedAuthorization {
    body: Arc<SourceReadAuthorization>,
    admission: Admission,
    contracts: Arc<Contracts>,
    source: SnapshotHandle,
}
#[derive(Default)]
pub(super) struct IssuedManagedSourceValues {
    values: BTreeMap<ManagedSourceReadKey, Issued>,
    authorizations: BTreeMap<SourceReadAuthorizationKey, IssuedAuthorization>,
    allocation: Option<Arc<SourceFactsAllocation>>,
}
impl IssuedManagedSourceValues {
    pub(in crate::domain_computation::primary_graph) fn authorization(
        &self,
        key: &SourceReadAuthorizationKey,
        admission: Admission,
        contracts: &Arc<Contracts>,
        source: &SnapshotHandle,
    ) -> Option<&Arc<SourceReadAuthorization>> {
        let issued = self.authorizations.get(key)?;
        (issued.admission == admission
            && Arc::ptr_eq(&issued.contracts, contracts)
            && issued.source == *source)
            .then_some(&issued.body)
    }
    pub(in crate::domain_computation::primary_graph) fn get(
        &self,
        key: &ManagedSourceReadKey,
        admission: Admission,
        contracts: &Arc<Contracts>,
        source: &SnapshotHandle,
    ) -> Option<&Arc<ManagedSourceRead>> {
        let issued = self.values.get(key)?;
        (issued.admission == admission
            && Arc::ptr_eq(&issued.contracts, contracts)
            && issued.source == *source)
            .then_some(&issued.body)
    }
}
impl DecisionReads {
    pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader) fn fail_managed_source(
        &mut self,
    ) {
        self.failed_managed_source = true;
    }
    pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader) fn record_managed_source(
        &mut self,
        body: &Arc<ManagedSourceRead>,
        admission: Admission,
        contracts: Arc<Contracts>,
        source: &SnapshotHandle,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), worth_relational::facade::mvcc::CompanionPreflightStop> {
        use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
        execution
            .consult()
            .map_err(|cause| Stop::Execution(cause.into()))?;
        self.record_source_authorization(
            body.authorization(),
            admission,
            Arc::clone(&contracts),
            source,
            execution,
        )?;
        let values = &mut self.managed_sources;
        if let Some(prior) = values.values.get(body.key()) {
            if prior.admission != admission
                || !Arc::ptr_eq(&prior.contracts, &contracts)
                || prior.source != *source
                || !prior.body.same_truth(body, &mut |work| {
                    SourceFactsAllocation::work(execution, work).map_err(Stop::Execution)
                })?
            {
                return Err(Stop::ManagedSourceChanged);
            }
        } else {
            if values.allocation.is_none() {
                values.allocation =
                    Some(SourceFactsAllocation::open(execution).map_err(Stop::Execution)?);
            }
            values
                .allocation
                .as_ref()
                .expect("opened authoring custody")
                .claim_tree::<(ManagedSourceReadKey, Issued)>(execution, values.values.len())
                .map_err(Stop::Execution)?;
            values.values.insert(
                body.key().clone(),
                Issued {
                    body: Arc::clone(body),
                    admission,
                    contracts,
                    source: source.clone(),
                },
            );
        }
        self.record(
            WorthQueryApplicationFactKey::ManagedIndexSource(body.key().clone()),
            DecisionReadOutcome::Observed,
        );
        Ok(())
    }
    fn record_source_authorization(
        &mut self,
        body: &Arc<SourceReadAuthorization>,
        admission: Admission,
        contracts: Arc<Contracts>,
        source: &SnapshotHandle,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), worth_relational::facade::mvcc::CompanionPreflightStop> {
        use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
        let values = &mut self.managed_sources;
        if let Some(prior) = values.authorizations.get(body.key()) {
            if prior.admission != admission
                || !Arc::ptr_eq(&prior.contracts, &contracts)
                || prior.source != *source
                || !prior.body.same_truth(body, &mut |work| {
                    SourceFactsAllocation::work(execution, work).map_err(Stop::Execution)
                })?
            {
                return Err(Stop::ManagedSourceChanged);
            }
        } else {
            if values.allocation.is_none() {
                values.allocation =
                    Some(SourceFactsAllocation::open(execution).map_err(Stop::Execution)?);
            }
            values
                .allocation
                .as_ref()
                .expect("opened original reader custody")
                .claim_tree::<(SourceReadAuthorizationKey, IssuedAuthorization)>(
                    execution,
                    values.authorizations.len(),
                )
                .map_err(Stop::Execution)?;
            values.authorizations.insert(
                body.key().clone(),
                IssuedAuthorization {
                    body: Arc::clone(body),
                    admission,
                    contracts,
                    source: source.clone(),
                },
            );
        }
        self.record(
            WorthQueryApplicationFactKey::SourceReadAuthorization(body.key().clone()),
            DecisionReadOutcome::Observed,
        );
        Ok(())
    }
}
