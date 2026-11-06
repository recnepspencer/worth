use std::sync::Arc;

pub(crate) mod comparison;
pub(in crate::domain_computation) use comparison::PrimaryDecisionReadSetComparison;

use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::{
    WorthQueryDecisionFactAdmission, WorthQueryDecisionFactComparisonAdmission,
    WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionFactEvidence,
    WorthQueryDecisionFactEvidenceView, WorthQueryDecisionFactProvider,
    WorthQueryDecisionFactRequestView, WorthQueryDecisionReadSetFailure,
    WorthQueryProviderSessionView,
};

impl WorthQueryDecisionFactProvider for Arc<WorthQueryPrimaryGraphProvider> {
    fn observe_decision_fact(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        request: WorthQueryDecisionFactRequestView<'_>,
        admission: WorthQueryDecisionFactAdmission,
    ) -> Result<WorthQueryDecisionFactEvidence, WorthQueryDecisionReadSetFailure> {
        let observed = self
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains_observed_fact(session, request.locator().identity());
        if !observed {
            return Err(provider_rejected());
        }
        admission.observe(format!(
            "application-observed:{}",
            request.locator().identity()
        ))
    }

    fn compare_decision_fact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
        evidence: WorthQueryDecisionFactEvidenceView<'_>,
        admission: WorthQueryDecisionFactComparisonAdmission,
    ) -> Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure> {
        let fact_basis = {
            self.attempts
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .observed_fact_and_product(session, evidence.locator().identity())
                .ok_or_else(provider_rejected)?
        };
        let fresh = self.graph.with_runtime_mut(execution, |runtime| {
            fact_basis.remains_equal_in(execution, runtime)
        })?;
        admission.observe_current_version(if fresh {
            evidence.physical_version_evidence().to_owned()
        } else {
            format!("application-stale:{}", evidence.locator().identity())
        })
    }
}

pub(super) fn snapshot_read_set_failure(
    denial: crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial,
) -> WorthQueryDecisionReadSetFailure {
    let kind = match denial {
        crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => crate::domain_computation::WorthQueryDecisionReadSetDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial::RetentionCapacityExhausted => {
            crate::domain_computation::WorthQueryDecisionReadSetDenialKind::RetentionCapacityExhausted
        }
        crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial::RetentionIdentityExhausted => {
            crate::domain_computation::WorthQueryDecisionReadSetDenialKind::RetentionIdentityExhausted
        }
        crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial::SnapshotIdentityExhausted => {
            crate::domain_computation::WorthQueryDecisionReadSetDenialKind::SnapshotIdentityExhausted
        }
        _ => crate::domain_computation::WorthQueryDecisionReadSetDenialKind::ProviderRejected,
    };
    WorthQueryDecisionReadSetFailure::new(
        kind,
        "primary provider could not open the exact decision-fact basis",
    )
}

pub(in crate::domain_computation::primary_graph) fn storage_read_set_failure(
    failure: crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::StoreDenial,
) -> WorthQueryDecisionReadSetFailure {
    use crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::StoreDenial as Store;
    use crate::domain_computation::WorthQueryDecisionReadSetDenialKind as Kind;
    match failure {
        Store::ExecutionRequest(stop) => {
            use crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind as Control;
            use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Interruption;
            let cause = stop.into();
            let kind =
                match crate::domain_computation::primary_graph::vocabulary::provider::relational_execution_denial::relational_execution_kind(cause, None) {
                    Ok(kind) => Kind::ExecutionRequest(kind),
                    Err(Control::Cancelled) => Kind::RequestInterrupted(Interruption::Cancelled),
                    Err(Control::TimedOut) => {
                        Kind::RequestInterrupted(Interruption::DeadlineExceeded)
                    }
                };
            WorthQueryDecisionReadSetFailure::new(kind, "original read storage request refused")
                .with_native_query_read_stop(
                    worth_relational::facade::runtime::QueryReadExecutionStop::ReadRequest(cause),
                )
        }
        Store::Allocation(denial) => WorthQueryDecisionReadSetFailure::allocation_denied(denial),
        Store::RequestInterruption(stop) => WorthQueryDecisionReadSetFailure::new(
            Kind::RequestInterrupted(stop),
            "original read interrupted",
        ),
        _ => WorthQueryDecisionReadSetFailure::new(
            Kind::EvidenceSubstitution,
            "invalid original read custody",
        ),
    }
}

fn provider_rejected() -> WorthQueryDecisionReadSetFailure {
    WorthQueryDecisionReadSetFailure::new(
        crate::domain_computation::WorthQueryDecisionReadSetDenialKind::ProviderRejected,
        "primary provider has no exact session-owned application fact",
    )
}

pub fn fact_read_set_failure(
    denial: crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial,
) -> WorthQueryDecisionReadSetFailure {
    denial.into_decision_read_set_failure()
}
