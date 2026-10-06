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
    admission: &mut InvalidationEditAdmission,
) -> Result<OutputDependencySelection, WorthQueryOutputDemandDenial> {
    let facts = match reusable_facts(identity_current, facts) {
        Ok(facts) => facts,
        Err(fresh) => return Ok(fresh),
    };
    match first_moved_fact(runtime, snapshot, facts, admission) {
        Ok(None) => Ok(OutputDependencySelection::Reuse),
        Ok(Some(_)) => Ok(OutputDependencySelection::FreshRequired),
        Err(RetainedFactStop::Work) => Err(work_denial()),
        Err(RetainedFactStop::Unavailable(fact)) => Err(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
            fact.locator_identity(),
        )),
    }
}

/// Why a retained output's facts could not be compared.
pub(in crate::domain_computation::primary_graph) enum RetainedFactStop<'fact> {
    /// The request's meter, or the fact's own recorded bound, stopped it.
    Work,
    /// The snapshot cannot answer this fact.
    Unavailable(&'fact WorthQueryApplicationObservedFact),
}

/// The first of a retained output's facts that moved at `snapshot`, compared
/// in order and paid from the request's meter. `None` when every fact holds.
pub(in crate::domain_computation::primary_graph) fn first_moved_fact<'fact>(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    facts: impl IntoIterator<Item = &'fact WorthQueryApplicationObservedFact>,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<&'fact WorthQueryApplicationObservedFact>, RetainedFactStop<'fact>> {
    for fact in facts {
        match fact.source_currentness_in(runtime, snapshot, admission) {
            Ok(Ok(movement)) if movement.movement() == Movement::Moved => return Ok(Some(fact)),
            Ok(Ok(_)) => {}
            Ok(Err(WorthQuerySourceCurrentnessFailure::Unavailable)) => {
                return Err(RetainedFactStop::Unavailable(fact));
            }
            Ok(Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded)) | Err(_) => {
                return Err(RetainedFactStop::Work);
            }
        }
    }
    Ok(None)
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
    admission: &mut InvalidationEditAdmission,
) -> Result<OutputDependencySelection, WorthQueryOutputDemandDenial> {
    let Some(witness) = witness else {
        return Ok(OutputDependencySelection::Reuse);
    };
    let unchanged = owner_read(
        witness.unchanged_in(runtime, snapshot, admission),
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
    admission: &mut InvalidationEditAdmission,
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
    let current = owner_read(
        owner.currentness(&selected, settlement, admission),
        UNAVAILABLE,
    )?;
    Ok(!cutoff_declines(requirement, Some(&current)))
}

/// One owner read on the selection's meter: what it charged stays spent
/// whether or not it answered.
fn owner_read<Answer>(
    answer: Result<Answer, CompanionPreflightStop>,
    unavailable: &'static str,
) -> Result<Answer, WorthQueryOutputDemandDenial> {
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
