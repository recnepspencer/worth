//! The single authoritative Relational commit transition.

mod precommit_snapshot;
mod product_publication;
mod publication;
pub(in crate::domain_computation::primary_graph) use publication::WorthQueryPrimaryGraphCommittedApplication;

use super::super::WorthQueryPreparedApplicationCommit;
use crate::domain_computation::primary_graph::provider::{
    mutation_work::WorthQueryPrimaryMutationWorkCounters,
    session_commit::{
        provider_failure, snapshot_admission_failure, WorthQueryPreImageRetentionWork,
    },
    WorthQueryPrimaryGraphApplicationAttempt, WorthQueryPrimaryGraphProvider,
};
use crate::domain_computation::{
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolStage,
};

pub(super) struct WorthQueryCommittedApplicationSession {
    attempt: WorthQueryPrimaryGraphApplicationAttempt,
    work: WorthQueryPrimaryMutationWorkCounters,
    retained_preimage:
        Option<crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage>,
    preimage_retention_work: WorthQueryPreImageRetentionWork,
    branch: worth_relational::facade::history::BranchId,
    before: worth_relational::facade::snapshots::SnapshotHandle,
    next_basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    committed: worth_relational::facade::transactions::CommitResult,
    product_publication: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt,
}

pub(super) fn commit(
    provider: &WorthQueryPrimaryGraphProvider,
    prepared: WorthQueryPreparedApplicationCommit,
    mint: super::WorthQueryCommitProgressionMint,
) -> Result<
    WorthQueryCommittedApplicationSession,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let WorthQueryPreparedApplicationCommit {
        mut attempt,
        candidate,
        work,
        branch,
        retained_preimage,
        preimage_retention_work,
    } = prepared;
    let _ = mint;
    let product = attempt.affinity().product_publication().clone();
    let before = precommit_snapshot::WorthQueryPrecommitSnapshot::acquire(
        provider.graph.clone(),
        product.observation().basis().relational_basis(),
    )
    .map_err(|denial| {
        snapshot_admission_failure(
            WorthQueryProviderSessionProtocolStage::Commit,
            denial.into(),
            "application publication could not retain its exact pre-commit basis",
        )
    })
    .map_err(crate::domain_computation::WorthQueryProviderSessionCommitStop::from)?;
    let candidate = provider
        .graph
        .with_runtime_mut(|runtime| runtime.prepare_validated_proposal(candidate))
        .map_err(transaction_commit_stop)?;
    let performed = product_publication::publish(provider, &mut attempt, candidate)?;
    let performed = performed.publication;
    let next_basis = performed
        .publication()
        .commit()
        .basis()
        .relational_basis()
        .clone();
    let committed = performed
        .publication()
        .component_results()
        .relational_commit_result()
        .expect("World performed a prepared Relational application candidate")
        .clone();
    Ok(WorthQueryCommittedApplicationSession {
        attempt,
        work,
        retained_preimage,
        preimage_retention_work,
        branch,
        before: before.into_publication(),
        next_basis,
        committed,
        product_publication: performed,
    })
}

fn world_no_effect(
    no_effect: worth_runtime_world::facade::NoEffectCompositePublication,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use crate::domain_computation::{
        WorthQueryProviderSessionCommitControlStopped, WorthQueryProviderSessionCommitStop as Stop,
        WorthQueryProviderSessionControlStopKind,
    };
    use worth_runtime_world::facade::NoEffectCause;
    match no_effect.cause() {
        NoEffectCause::StaleExpectedProductHead => Stop::ProductStale(
            crate::domain_computation::WorthQueryProductStaleApplication::new(no_effect),
        ),
        NoEffectCause::CancelledBeforeEffect => {
            Stop::ControlStopped(WorthQueryProviderSessionCommitControlStopped::new(
                WorthQueryProviderSessionControlStopKind::Cancelled,
                "World publication cancelled before effect",
            ))
        }
        NoEffectCause::DeadlineBeforeEffect => {
            Stop::ControlStopped(WorthQueryProviderSessionCommitControlStopped::new(
                WorthQueryProviderSessionControlStopKind::TimedOut,
                "World publication deadline elapsed before effect",
            ))
        }
        _ => Stop::NoEffect(no_effect),
    }
}
impl WorthQueryCommittedApplicationSession {
    pub(super) const fn attempt(&self) -> &WorthQueryPrimaryGraphApplicationAttempt {
        &self.attempt
    }

    pub(super) const fn work(&self) -> WorthQueryPrimaryMutationWorkCounters {
        self.work
    }

    pub(super) const fn retained_preimage(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage> {
        self.retained_preimage.as_ref()
    }

    pub(super) const fn preimage_retention_work(&self) -> WorthQueryPreImageRetentionWork {
        self.preimage_retention_work
    }

    pub(super) const fn committed(&self) -> &worth_relational::facade::transactions::CommitResult {
        &self.committed
    }

    pub(super) fn publish_and_encode(
        self,
        provider: &WorthQueryPrimaryGraphProvider,
        runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
        evidence: super::WorthQueryPrimaryGraphCommitEvidence,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        WorthQueryProviderSessionFailure,
    > {
        let published = publication::publish(provider, runtime, self, evidence)?;
        publication::encode(provider, published)
    }
}

fn failure(detail: &'static str) -> WorthQueryProviderSessionFailure {
    provider_failure(WorthQueryProviderSessionProtocolStage::Commit, detail)
}

fn transaction_commit_stop(
    error: worth_relational::facade::mvcc::TransactionCommitError,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use worth_relational::facade::mvcc::TransactionCommitError as Error;
    match error {
        Error::Interrupted { interruption, .. } => {
            crate::domain_computation::WorthQueryProviderSessionCommitStop::ControlStopped(
                interruption_control_stopped(interruption),
            )
        }
        Error::PublicationDeferred { deferred, .. } => {
            crate::domain_computation::WorthQueryProviderSessionCommitStop::Deferred(
                publication_deferred(deferred),
            )
        }
        Error::PublicationFailed { failure, .. } => {
            crate::domain_computation::WorthQueryProviderSessionCommitStop::Denied(
                publication_failure(failure),
            )
        }
        Error::PerformedButDurabilityDeferred {
            settlement, error, ..
        } => crate::domain_computation::WorthQueryProviderSessionCommitStop::SettlementDeferred(
            crate::domain_computation::WorthQueryProviderSessionSettlementDeferred::new(
                error.detail,
                settlement,
            ),
        ),
        _ => crate::domain_computation::WorthQueryProviderSessionCommitStop::Denied(failure(
            "Relational rejected application commit preparation",
        )),
    }
}

fn interruption_control_stopped(
    event: worth_relational::facade::mvcc::RelationalInterruptionEvent,
) -> crate::domain_computation::WorthQueryProviderSessionCommitControlStopped {
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Kind;
    let kind = match event.interruption() {
        worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled => {
            Kind::Cancelled
        }
        worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut => Kind::TimedOut,
    };
    crate::domain_computation::WorthQueryProviderSessionCommitControlStopped::new(
        kind,
        format!("{event:?}"),
    )
}

fn publication_deferred(
    deferred: worth_relational::facade::mvcc::RelationalPublicationDeferred,
) -> crate::domain_computation::WorthQueryProviderSessionCommitDeferred {
    use crate::domain_computation::WorthQueryProviderSessionCommitDeferredKind as Kind;
    use worth_relational::facade::mvcc::RelationalPublicationDeferred as Deferred;
    let kind = match deferred {
        Deferred::PatchPositionReservationContended => Kind::PatchPositionReservationContended,
        Deferred::RetentionBackpressure => Kind::RetentionCapacityExhausted,
        Deferred::CandidateLifetimeExpired {
            maximum_lifetime_millis,
        } => Kind::CandidateLifetimeExpired {
            maximum_lifetime_millis,
        },
        Deferred::CandidateCapacityExhausted { maximum_candidates } => {
            Kind::CandidateCapacityExhausted { maximum_candidates }
        }
        Deferred::PublishedSnapshotCapacityExhausted { maximum_handles } => {
            Kind::PublishedSnapshotCapacityExhausted { maximum_handles }
        }
    };
    crate::domain_computation::WorthQueryProviderSessionCommitDeferred::new(
        kind,
        format!("{deferred:?}"),
    )
}

fn publication_failure(
    failure: worth_relational::facade::mvcc::RelationalPublicationFailure,
) -> WorthQueryProviderSessionFailure {
    use worth_relational::facade::mvcc::RelationalPublicationFailureKind as Failure;
    let kind = match failure.kind() {
        Failure::SnapshotIdentityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::SnapshotIdentityExhausted
        }
        Failure::CandidateIdentityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::CandidateIdentityExhausted
        }
        Failure::RetentionIdentityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionIdentityExhausted
        }
        Failure::PreparedRootBudgetExhausted {
            maximum_bytes,
            required_bytes,
        } => crate::domain_computation::WorthQueryProviderSessionDenialKind::PreparedRootBudgetExhausted {
            maximum_bytes: *maximum_bytes,
            required_bytes: *required_bytes,
        },
        _ => crate::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected,
    };
    WorthQueryProviderSessionFailure::new(
        kind,
        WorthQueryProviderSessionProtocolStage::Commit,
        failure.detail(),
        crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
    )
}
