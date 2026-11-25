//! Original Source bodies stay with the admitted reader; keys cannot reissue them.
use super::*;
use crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity as Admission;
use crate::domain_computation::primary_graph::{
    application_attempt::{
        retained_decision_facts::StorageControl, WorthQueryApplicationObservedFact as Fact,
    },
    application_query::observed_source::fact_conversion::source_fact_locator::SourceFactLocator,
    projection_read_origin::ProjectionReadOriginRef,
};
use crate::domain_computation::primary_graph::{
    application_attempt::{
        ManagedSourceRead, ManagedSourceReadKey, SourceReadAuthorization,
        SourceReadAuthorizationKey,
    },
    source_facts::SourceFactsAllocation,
};
use std::{collections::BTreeMap, sync::Arc};
use worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts as Contracts;
use worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification as Candidate;
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
struct IssuedApplied {
    body: Fact,
    admission: Admission,
    contracts: Arc<Contracts>,
    source: Candidate,
}

#[derive(Default)]
pub struct IssuedManagedSourceValues {
    values: BTreeMap<ManagedSourceReadKey, Issued>,
    applied: BTreeMap<Arc<SourceFactLocator>, IssuedApplied>,
    authorizations: BTreeMap<SourceReadAuthorizationKey, IssuedAuthorization>,
    allocation: Option<Arc<SourceFactsAllocation>>,
}
impl IssuedManagedSourceValues {
    /// Whether nothing was issued (PR C §1.7: a pending refusal's proofs).
    pub fn is_empty(&self) -> bool {
        self.values.is_empty() && self.applied.is_empty() && self.authorizations.is_empty()
    }
    pub(super) fn applied(
        &self,
        key: &SourceFactLocator,
        admission: Admission,
        contracts: &Arc<Contracts>,
        origin: ProjectionReadOriginRef<'_>,
    ) -> Option<&Fact> {
        let issued = self.applied.get(key)?;
        (issued.admission == admission
            && Arc::ptr_eq(&issued.contracts, contracts)
            && matches!(origin, ProjectionReadOriginRef::Applied(candidate) if candidate == issued.source))
            .then_some(&issued.body)
    }
    pub fn authorization(
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
    pub fn get(
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
    pub fn fail_managed_source(&mut self) {
        self.failed_managed_source = true;
    }
    pub fn record_managed_source(
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
                || !prior.body.same_truth(
                    body,
                    &mut |work| {
                        SourceFactsAllocation::work(execution, work).map_err(Stop::Execution)
                    },
                    &mut |bytes| {
                        execution
                            .reserve_retained_memory(bytes)
                            .map_err(|cause| Stop::Execution(cause.into()))
                    },
                    &mut |cause| Stop::Execution(cause.into_native()),
                    &mut |left, right| {
                        left.same_content(execution, right)
                            .map_err(|cause| Stop::Execution(cause.into_execution_cause()))
                    },
                )?
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
    pub fn record_applied_managed_source(
        &mut self,
        facts: Vec<Fact>,
        authorization: &Arc<SourceReadAuthorization>,
        admission: Admission,
        contracts: Arc<Contracts>,
        source: &SnapshotHandle,
        control: StorageControl<'_, '_>,
    ) -> Result<(), worth_relational::facade::mvcc::CompanionPreflightStop> {
        use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
        let execution = &control.execution().ok_or(Stop::SelectedSourceMismatch)?;
        let candidate = authorization
            .applied
            .as_ref()
            .ok_or(Stop::SelectedSourceMismatch)?;
        self.record_source_authorization(
            authorization,
            admission,
            Arc::clone(&contracts),
            source,
            execution,
        )?;
        for fact in facts {
            SourceFactsAllocation::work(execution, 1).map_err(Stop::Execution)?;
            let key = SourceFactLocator::from_fact(&fact);
            let values = &mut self.managed_sources;
            let key = if let Some((key, _)) = values.applied.get_key_value(&key) {
                let key = Arc::clone(key);
                let issued = values.applied.get_mut(key.as_ref()).expect("located fact");
                if issued.admission != admission
                    || !Arc::ptr_eq(&issued.contracts, &contracts)
                    || issued.source != candidate.qualification()
                    || !issued
                        .body
                        .merge_after_equal_source_key(fact, control)
                        .map_err(source_merge_stop)?
                {
                    return Err(Stop::ManagedSourceChanged);
                }
                key
            } else {
                let allocation = values.allocation.as_ref().expect("Source READ retained");
                allocation
                    .claim_arc::<SourceFactLocator>(execution)
                    .map_err(Stop::Execution)?;
                use worth_execution::ChargedBytes as _;
                allocation
                    .claim(execution, fact.additional_charged_bytes())
                    .map_err(Stop::Execution)?;
                allocation
                    .claim_tree::<(Arc<SourceFactLocator>, IssuedApplied)>(
                        execution,
                        values.applied.len(),
                    )
                    .map_err(Stop::Execution)?;
                let key = Arc::new(key);
                values.applied.insert(
                    Arc::clone(&key),
                    IssuedApplied {
                        body: fact,
                        admission,
                        contracts: Arc::clone(&contracts),
                        source: candidate.qualification(),
                    },
                );
                key
            };
            self.record(
                WorthQueryApplicationFactKey::AppliedSource(key),
                DecisionReadOutcome::Observed,
            );
        }
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

fn source_merge_stop(
    cause: crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial,
) -> worth_relational::facade::mvcc::CompanionPreflightStop {
    use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial;
    use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
    match cause {
        StoreDenial::ExecutionRequest(cause) => Stop::Execution(cause.into()),
        StoreDenial::Allocation(cause) => match cause.classify() {
            worth_execution::ExecutionAllocationDenialClass::Request(stop) => Stop::Execution(stop.into()),
            worth_execution::ExecutionAllocationDenialClass::Physical(_) => Stop::Execution(Cause::AllocationFailed),
        },
        StoreDenial::PhysicalAllocation => Stop::Execution(Cause::AllocationFailed),
        StoreDenial::RequestInterruption(stop) => Stop::Execution(match stop {
            worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled => Cause::Cancelled,
            worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded => Cause::DeadlineElapsed,
        }),
        StoreDenial::Content(cause) => Stop::Execution(cause.into_execution_cause()),
        _ => Stop::ManagedSourceChanged,
    }
}
