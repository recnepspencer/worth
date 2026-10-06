use worth_relational::facade::mvcc::CompanionPreflightStop;

use crate::domain_computation::primary_graph::{
    application_attempt::{
        Movement, WorthQueryApplicationObservedFact, WorthQuerySourceCurrentnessFailure,
    },
    output_lineage::{
        cutoff_declines,
        invalidation::{FullVerificationReason, InvalidationEditAdmission},
        RecordedSettlementIdentity, SealedNativeOutputWitness,
    },
    SourceInvalidationOwner, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

/// Internal choice; callers can only request an output through demand
/// admission. FreshRequired never grants authority to preserve an output.
pub(in crate::domain_computation::primary_graph) enum OutputDependencySelection {
    Reuse,
    FreshRequired,
}

pub(in crate::domain_computation::primary_graph) fn compare_retained_output_dependencies(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    identity_current: bool,
    facts: Option<&[WorthQueryApplicationObservedFact]>,
    remaining_work: &mut usize,
) -> Result<OutputDependencySelection, WorthQueryOutputDemandDenial> {
    let facts = match reusable_facts(identity_current, facts) {
        Ok(facts) => facts,
        Err(fresh) => return Ok(fresh),
    };
    for fact in facts {
        let available = *remaining_work;
        let prepaid = fact
            .exact_probe_work()
            .map_err(|_| work_denial())?
            .unwrap_or(0);
        if prepaid > *remaining_work {
            return Err(work_denial());
        }
        *remaining_work -= prepaid;
        let (movement, work) = fact
            .source_currentness_in(runtime, snapshot, available)
            .map_err(|failure| match failure {
                WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded => {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                        "output dependency comparison exceeded the admitted work budget",
                    )
                }
                WorthQuerySourceCurrentnessFailure::Unavailable => {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                        fact.locator_identity(),
                    )
                }
            })?;
        *remaining_work -= work.saturating_sub(prepaid);
        if movement.movement() == Movement::Moved {
            return Ok(OutputDependencySelection::FreshRequired);
        }
    }
    Ok(OutputDependencySelection::Reuse)
}

/// The other half of a retained output's canonical fact set: the performed
/// output itself. A role entity or aspect revision that moved since the
/// producer published selects producer execution over the live output, the
/// same verdict marking and consumed-output checks reach from this witness.
/// A row no sealed witness covers is decided by its source facts and by the
/// full verification its lineage row requires.
pub(in crate::domain_computation::primary_graph) fn compare_retained_output_witness(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    witness: Option<&SealedNativeOutputWitness>,
    owner: &SourceInvalidationOwner,
    remaining_work: &mut usize,
) -> Result<OutputDependencySelection, WorthQueryOutputDemandDenial> {
    let Some(witness) = witness else {
        return Ok(OutputDependencySelection::Reuse);
    };
    let mut admission = owner.read_admission(*remaining_work);
    let unchanged = witness.unchanged_in(runtime, snapshot, &mut admission);
    let unchanged = owner_read(
        unchanged,
        &admission,
        remaining_work,
        "retained output witness could not be compared",
    )?;
    Ok(if unchanged {
        OutputDependencySelection::Reuse
    } else {
        OutputDependencySelection::FreshRequired
    })
}

/// Whether the input cutoff verifies this row's settlement under the selected
/// source. A row it declines cannot be an exact selection: the cutoff would
/// run its producer again, so the live output selects the Preserve posture.
pub(in crate::domain_computation::primary_graph) fn retained_output_settlement_is_verified(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    requirement: Option<FullVerificationReason>,
    settlement: &RecordedSettlementIdentity,
    owner: &SourceInvalidationOwner,
    remaining_work: &mut usize,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    const UNAVAILABLE: &str = "retained output settlement could not be read";
    if cutoff_declines(requirement, None) {
        return Ok(false);
    }
    let selected = runtime
        .read_truth()
        .positioned_snapshot(snapshot)
        .map_err(|_| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                UNAVAILABLE,
            )
        })?;
    let mut admission = owner.read_admission(*remaining_work);
    let current = owner.currentness(&selected, settlement, &mut admission);
    let current = owner_read(current, &admission, remaining_work, UNAVAILABLE)?;
    Ok(!cutoff_declines(requirement, Some(&current)))
}

/// Settle one owner read against the selection's work: what the read charged
/// is spent whether or not it answered.
fn owner_read<Answer>(
    answer: Result<Answer, CompanionPreflightStop>,
    admission: &InvalidationEditAdmission,
    remaining_work: &mut usize,
    unavailable: &'static str,
) -> Result<Answer, WorthQueryOutputDemandDenial> {
    let charged = usize::try_from(admission.charged_work()).map_err(|_| work_denial())?;
    *remaining_work = remaining_work
        .checked_sub(charged)
        .ok_or_else(work_denial)?;
    answer.map_err(|stop| match stop {
        CompanionPreflightStop::WorkExhausted { .. }
        | CompanionPreflightStop::WorkCounterOverflow => work_denial(),
        _ => WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
            unavailable,
        ),
    })
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "output dependency comparison exceeded the admitted work budget",
    )
}

fn reusable_facts(
    identity_current: bool,
    facts: Option<&[WorthQueryApplicationObservedFact]>,
) -> Result<&[WorthQueryApplicationObservedFact], OutputDependencySelection> {
    if !identity_current {
        return Err(OutputDependencySelection::FreshRequired);
    }
    facts
        .filter(|facts| !facts.is_empty())
        .ok_or(OutputDependencySelection::FreshRequired)
}

#[cfg(test)]
mod tests {
    use super::{reusable_facts, OutputDependencySelection};

    #[test]
    fn missing_dependency_facts_never_authorize_retained_output() {
        assert!(matches!(
            reusable_facts(true, None),
            Err(OutputDependencySelection::FreshRequired)
        ));
        assert!(matches!(
            reusable_facts(true, Some(&[])),
            Err(OutputDependencySelection::FreshRequired)
        ));
        assert!(matches!(
            reusable_facts(false, None),
            Err(OutputDependencySelection::FreshRequired)
        ));
    }
}
