//! Every recorded coordinate joins the same protected original and Native receipt.
use super::*;
pub(super) fn readmit(
    execution: &ExecutionRequest<'_, '_>,
    originals: &WorldProgramActivationHistory,
    facts: Vec<WorthQueryCanonicalCorrectionFact>,
    scope: &JoinScope<'_>,
) -> Result<BTreeMap<CommitId, WorthQueryCanonicalCorrectionFact>, Denial> {
    let mut joined = BTreeMap::new();
    for fact in facts {
        let child = original_for_commit(originals, fact.child.commit_id, &fact.record)?;
        match scope {
            JoinScope::CurrentNativeHead(_) => {}
            JoinScope::SelectedWorldBranch(branch) => {
                if !originals
                    .original_is_ancestor_of(&child, branch.selected_commit(), execution)
                    .map_err(Denial::World)?
                {
                    return Err(Denial::OutsideOriginalAncestry {
                        record: fact.record,
                    });
                }
            }
        }
        join_coordinates(execution, originals, &fact, &child)?;
        let record = fact.record.clone();
        if joined.insert(fact.child.commit_id, fact).is_some() {
            return Err(Denial::InconsistentRelationships { record });
        }
    }
    Ok(joined)
}
fn join_coordinates(
    execution: &ExecutionRequest<'_, '_>,
    originals: &WorldProgramActivationHistory,
    fact: &WorthQueryCanonicalCorrectionFact,
    child: &WorldOriginalObservation<'_>,
) -> Result<(), Denial> {
    for coordinate in [
        Some(&fact.source),
        Some(&fact.root),
        fact.selected.as_ref(),
        fact.active.as_ref(),
        fact.prior.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        let original = originals
            .original_observations()
            .find(|original| {
                original.identity().owner() == child.identity().owner()
                    && original.identity().ordinal() == coordinate.world_commit()
            })
            .ok_or_else(|| Denial::MissingWorldOriginal {
                record: fact.record.clone(),
            })?;
        if original.relational_commit() != coordinate.relational().commit_id {
            return Err(Denial::WorldOriginalMismatch {
                record: fact.record.clone(),
            });
        }
        if !originals
            .original_precedes_original(&original, child, execution)
            .map_err(Denial::World)?
        {
            return Err(Denial::OutsideOriginalAncestry {
                record: fact.record.clone(),
            });
        }
    }
    Ok(())
}
fn original_for_commit<'a>(
    originals: &'a WorldProgramActivationHistory,
    commit: CommitId,
    record: &RecordRef,
) -> Result<WorldOriginalObservation<'a>, Denial> {
    let mut matches = originals
        .original_observations()
        .filter(|original| original.relational_commit() == commit);
    let original = matches.next().ok_or_else(|| Denial::MissingWorldOriginal {
        record: record.clone(),
    })?;
    if matches.next().is_some() {
        return Err(Denial::AmbiguousWorldOriginal {
            record: record.clone(),
        });
    }
    Ok(original)
}
