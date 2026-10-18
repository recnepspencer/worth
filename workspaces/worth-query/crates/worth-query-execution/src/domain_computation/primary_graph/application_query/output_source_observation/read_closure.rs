//! Original Query session and authorization custody through publication.

use super::*;
use crate::domain_computation::authorization::{
    WorthQueryInstalledAuthorizationRegistry, WorthQueryRetainedAuthorizationDecisionFacts,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::{
    arc_bytes, arc_slice_bytes, InvalidationEditAdmission,
};
use std::sync::Arc;

pub(in crate::domain_computation::primary_graph) struct SourceQueryReadClosure {
    control:crate::domain_computation::primary_graph::application_runtime::WorthQueryApplicationAttemptOperationControl,
    receipt: WorthQueryApplicationQueryAccessReceipt,
    authorization: WorthQueryRetainedAuthorizationDecisionFacts,
    registry: Arc<WorthQueryInstalledAuthorizationRegistry>,
    principal_facts: Arc<[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact]>,
    facts: Arc<[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact]>,
    _capacity: crate::domain_computation::primary_graph::output_lineage::invalidation::RetainedConsumedOutputCapacity,
}

impl SourceQueryReadClosure {
    pub(super) fn retain<Schema: ApplicationSchema>(
        application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        receipt: WorthQueryApplicationQueryAccessReceipt,
        authorization: WorthQueryRetainedAuthorizationDecisionFacts,
        selected: &worth_relational::facade::runtime::PositionedRelationalSnapshot,
        preparation_bytes: u64,
        facts: Vec<crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Arc<Self>, CompanionPreflightStop> {
        let principal_start = admission.charged_bytes();
        let principal_facts = authorization.source_query_principal_facts(admission)?;
        let principal_bytes = admission
            .charged_bytes()
            .checked_sub(principal_start)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        // Both Vec backings remain live while their Arc slices are allocated.
        // Retain the complete original preparation and these new destinations.
        type Fact = crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact;
        let slices = arc_slice_bytes::<Fact>(facts.len())
            .and_then(|bytes| bytes.checked_add(arc_slice_bytes::<Fact>(principal_facts.len())?))
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let bytes = arc_bytes::<Self>()
            .and_then(|bytes| {
                bytes
                    .checked_add(preparation_bytes)
                    .and_then(|bytes| bytes.checked_add(principal_bytes))
                    .and_then(|bytes| bytes.checked_add(slices))
            })
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        admission.admit_read_scratch(bytes)?;
        let copies = facts
            .len()
            .checked_add(principal_facts.len())
            .and_then(|n| n.checked_mul(2))
            .and_then(|n| u64::try_from(n).ok())
            .and_then(|n| n.checked_add(3))
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        admission.charge_external_work(copies)?;
        let owner = &application
            .primary_provider
            .graph
            .source_owner
            .invalidation_owner;
        // Conservative custody: retain every byte admitted by this source read,
        // including temporary scratch. The receipt and permission cannot outlive
        // the owner funding that admitted their complete preparation.
        let capacity = owner.retain_consumed_output(&facts, selected, bytes, admission)?;
        Ok(Arc::new(Self {
            control: application
                .primary_provider
                .application_attempt_operation_control
                .clone(),
            receipt,
            principal_facts: principal_facts.into(),
            authorization,
            facts: facts.into(),
            registry: Arc::clone(&application.authorization),
            _capacity: capacity,
        }))
    }

    pub(in crate::domain_computation::primary_graph) fn admits_source<Query>(
        &self,
        source: &WorthQueryObservedSource<Query>,
    ) -> bool {
        self.receipt.query_identity() == &source.query_identity
            && self.receipt.parameter_binding_identity() == &source.parameter_binding_identity
            && self.receipt.basis_identity().selection() == &source.selection
            && self
                .authorization
                .belongs_to_session(self.receipt.read_completion().session_identity())
    }

    pub(in crate::domain_computation::primary_graph) fn facts(
        &self,
    ) -> &[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact] {
        &self.facts
    }

    pub(in crate::domain_computation::primary_graph) fn source_facts_current(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure>{
        use crate::domain_computation::primary_graph::application_attempt::{
            Movement, WorthQuerySourceCurrentnessFailure as Stop,
        };
        for fact in self.facts.iter() {
            self.control.before_source_fact_comparison();
            let movement = fact
                .source_currentness_in(runtime, execution, snapshot, admission)
                .map_err(|stop| Stop::OutputObservation(
                    crate::domain_computation::primary_graph::invariant_projection::consumed_output::map_admission_stop(stop)
                ))??;
            if movement.movement() != Movement::Unmoved {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(in crate::domain_computation::primary_graph) fn authorization_facts(
        &self,
    ) -> &[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact] {
        &self.principal_facts
    }

    pub(in crate::domain_computation::primary_graph) fn validate_for_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop>{
        use crate::domain_computation::primary_graph::invariant_projection::consumed_output::map_admission_stop;
        use crate::domain_computation::{
            authorization::{
                AuthorizationCurrentnessStop,
                WorthQueryOperationAuthorizationDenialKind as Authorization,
                WorthQueryOperationAuthorizationRefusalKind as Refusal,
            },
            primary_graph::{
                application_attempt::Movement,
                invariant_projection::ConsumedOutputVerificationStop as Stop,
            },
        };
        for fact in self.principal_facts.iter() {
            let current = fact.source_currentness_in(runtime, execution, snapshot, admission)
                .map_err(map_admission_stop)?.map_err(|stop| match stop {
                    crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::OutputObservation(stop) => stop,
                    crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::Execution(stop) => Stop::Execution(stop),
                    crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded => Stop::WorkExhausted,
                    crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::Unavailable => Stop::Unavailable,
                })?;
            if current.movement() != Movement::Unmoved {
                return Err(Stop::SourceQueryAuthorization(Authorization::Refused(
                    Refusal::StalePrincipal,
                )));
            }
        }
        let current = self
            .authorization
            .conventional_decisions_current_admitted(
                execution,
                runtime,
                snapshot,
                self.registry.bridge(),
                admission,
            )
            .map_err(|stop| match stop {
                AuthorizationCurrentnessStop::Admission(stop) => map_admission_stop(stop),
                AuthorizationCurrentnessStop::Execution(stop) => Stop::Execution(stop),
            })?;
        current
            .then_some(())
            .ok_or(Stop::SourceQueryAuthorization(Authorization::Refused(
                Refusal::StaleAuthorization,
            )))
    }
}
