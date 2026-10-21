//! A freshly performed consumer restores missing derived upstream rows.
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerification;
use crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceInvalidationOwner;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceSettlementCurrentness;
use std::sync::Arc;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

pub(in crate::domain_computation::primary_graph) enum PublicationRecoveryStop {
    Reason(FullVerificationReason),
    Closure(super::ConsumedOutputVerificationStop),
    Registration(crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementRegistrationStop),
    Admission(worth_relational::facade::mvcc::CompanionPreflightStop),
}

impl PublicationRecoveryStop {
    fn reason(self) -> FullVerificationReason {
        match self {
            Self::Reason(reason) => reason,
            Self::Closure(_) | Self::Registration(_) | Self::Admission(_) => {
                FullVerificationReason::RegistrationIncomplete
            }
        }
    }
}

enum Visit<'a> {
    Enter(&'a ConsumedOutputEvidence),
    Established(&'a ConsumedOutputEvidence),
}

impl ConsumedOutputEvidence {
    pub(in crate::domain_computation::primary_graph) fn establish_missing(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        owner: &SourceInvalidationOwner,
        roots: &[ConsumedOutputEvidence],
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<(), FullVerificationReason> {
        Self::recover_missing(execution, owner, roots, runtime, snapshot, selected)
            .map_err(PublicationRecoveryStop::reason)
    }

    pub(in crate::domain_computation::primary_graph) fn recover_missing(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        owner: &SourceInvalidationOwner,
        roots: &[ConsumedOutputEvidence],
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<(), PublicationRecoveryStop> {
        let incomplete =
            PublicationRecoveryStop::Reason(FullVerificationReason::RegistrationIncomplete);
        let mut pending = Vec::new();
        reserve(&mut pending, roots.len())?;
        pending.extend(roots.iter().map(Visit::Enter));
        let mut checked = im::OrdSet::new();
        while let Some(visit) = pending.pop() {
            let evidence = match visit {
                Visit::Enter(evidence) => {
                    if checked.contains(evidence.identity()) {
                        continue;
                    }
                    checked.insert(Arc::clone(evidence.identity()));
                    let state = owner
                        .currentness(selected, evidence.identity())
                        .map_err(|stop| PublicationRecoveryStop::Registration(crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementRegistrationStop::Admission(stop)))?;
                    if !state.no_row_answers() {
                        continue;
                    }
                    if matches!(state, SourceSettlementCurrentness::Foreign) {
                        return Err(incomplete);
                    }
                    // Postconditions cannot prove the original computation. This
                    // compares its original source facts and sealed output here.
                    if ConsumedOutputEvidence::verify_many_with_admission(
                        execution,
                        std::slice::from_ref(evidence),
                        owner,
                        runtime,
                        snapshot,
                        selected,
                    )
                    .map_err(PublicationRecoveryStop::Closure)?
                        != ConsumedOutputVerification::Current
                    {
                        return Err(PublicationRecoveryStop::Reason(
                            FullVerificationReason::MissingSettlement,
                        ));
                    }
                    reserve(&mut pending, evidence.upstream.len().saturating_add(1))?;
                    pending.push(Visit::Established(evidence));
                    pending.extend(evidence.upstream.iter().map(Visit::Enter));
                    continue;
                }
                Visit::Established(evidence) => evidence,
            };
            let witness = evidence
                .native_output_witness
                .as_ref()
                .and_then(|w| w.get())
                .ok_or(PublicationRecoveryStop::Reason(
                    FullVerificationReason::NativeRevisionUnavailable,
                ))?;
            let upstream = crate::domain_computation::primary_graph::output_lineage::invalidation::collect_consumed_output_upstream(&*evidence.upstream)
            .map_err(PublicationRecoveryStop::Admission)?;
            owner
                .establish_verified_consumed(
                    selected,
                    evidence.identity(),
                    &evidence.source_facts,
                    witness,
                    upstream,
                )
                .map_err(PublicationRecoveryStop::Registration)?;
        }
        Ok(())
    }
}

fn reserve<'a>(
    pending: &mut Vec<Visit<'a>>,
    additional: usize,
) -> Result<(), PublicationRecoveryStop> {
    let incomplete =
        || PublicationRecoveryStop::Reason(FullVerificationReason::RegistrationIncomplete);

    pending
        .try_reserve_exact(additional)
        .map_err(|_| incomplete())
}
