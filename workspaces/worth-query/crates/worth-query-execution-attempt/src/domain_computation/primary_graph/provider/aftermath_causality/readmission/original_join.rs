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
        match original_for_commit(originals, fact.child.commit_id, &fact.record) {
            Ok(child) => {
                match scope {
                    JoinScope::CurrentNativeHead(_, _, _) => {}
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
                join_coordinates(execution, originals, &fact, &child, false)?;
            }
            Err(Denial::MissingWorldOriginal { .. }) => {
                let JoinScope::CurrentNativeHead(_, continuable, _) = scope else {
                    return Err(Denial::MissingWorldOriginal {
                        record: fact.record,
                    });
                };
                let attempt = continuable.get(&fact.child.commit_id).ok_or_else(|| {
                    Denial::MissingWorldOriginal {
                        record: fact.record.clone(),
                    }
                })?;
                let state = originals.original_state(*attempt).ok_or_else(|| {
                    Denial::MissingWorldOriginal {
                        record: fact.record.clone(),
                    }
                })?;
                if !matches!(
                    state.fate(),
                    worth_foundational::facade::WorldOriginalFate::Open
                        | worth_foundational::facade::WorldOriginalFate::Continued
                ) {
                    return Err(Denial::MissingWorldOriginal {
                        record: fact.record,
                    });
                }
                let predecessor = originals
                    .original_observations()
                    .find(|obs| obs.identity() == state.predecessor())
                    .ok_or_else(|| Denial::MissingWorldOriginal {
                        record: fact.record.clone(),
                    })?;
                join_coordinates(execution, originals, &fact, &predecessor, true)?;
            }
            Err(cause) => return Err(cause),
        }
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
    anchor: &WorldOriginalObservation<'_>,
    allow_equal_anchor: bool,
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
                original.identity().owner() == anchor.identity().owner()
                    && original.identity().ordinal() == coordinate.world_commit()
            })
            .ok_or_else(|| Denial::MissingWorldOriginal {
                record: fact.record.clone(),
            })?;
        if original.relational_commit() != Some(coordinate.relational().commit_id) {
            return Err(Denial::WorldOriginalMismatch {
                record: fact.record.clone(),
            });
        }
        let valid_ancestry = (allow_equal_anchor && original.identity() == anchor.identity())
            || originals
                .original_precedes_original(&original, anchor, execution)
                .map_err(Denial::World)?;
        if !valid_ancestry {
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
        .filter(|original| original.relational_commit() == Some(commit));
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
