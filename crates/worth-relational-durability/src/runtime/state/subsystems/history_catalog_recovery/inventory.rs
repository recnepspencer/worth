//! Prepare the recovered source inventory before its single ledger application.
use super::*;
use worth_relational_contracts::internal_port as contracts;
pub fn install_recovered_canonical_inventory(
    history: &HistorySubsystem,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    commits: &[Arc<worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit>],
    symbols: &contracts::symbols::data::string_interner::StringInterner,
) -> Result<(), RelationalHistoryRecoveryDenial> {
    let mut prepared =
        worth_execution::ExecutionRequestArrayBuilder::allocate(*execution, commits.len())
            .map_err(array_stop)?;
    let mut seen = worth_execution::ExecutionOrderedMap::default();
    for commit in commits {
        if seen.get(&commit.commit.commit_id).is_some() {
            return Err("duplicate recovered source commit".to_owned().into());
        }
        seen.insert_cloned_native_for_request(&commit.commit.commit_id, &(), *execution)
            .map_err(|cause| {
                RelationalHistoryRecoveryDenial::Source(
                    contracts::durability::data::recovery_stop::map_stop(cause),
                )
            })?;
        prepared
            .push(source_binding::prepare_recovered_source_catalog(
                history, execution, commit,
            )?)
            .map_err(array_stop)?;
    }
    let prepared = prepared.seal().map_err(array_stop)?;
    history.with_ledger_mut(|ledger| {
        for plan in &prepared {
            if !ledger
                .commit_catalog
                .source_replacement_matches(plan.artifact.commit_id(), plan.expected.as_ref())
            {
                return Err(RelationalHistoryRecoveryDenial::Catalog(
                    worth_relational_branch::internal_port::history::commit::catalog::RelationalCommitCatalogAppendDenial::ChangedDuringPreparation,
                ));
            }
        }
        for (commit, plan) in commits.iter().zip(&prepared) {
            ledger
                .commit_catalog
                .install_source_recovery(plan.expected.as_ref(), Arc::clone(&plan.artifact))
                .expect("same locked catalog validated every expected source before effects");
            ledger
                .commit_envelopes
                .insert(commit.commit.commit_id, plan.artifact.envelope_residency());
            ledger
                .patch_stream_index
                .insert(commit.position(), commit.commit.commit_id);
        }
        Ok(())
    })?;
    let admitted = commits
        .iter()
        .map(|commit| (commit.commit.commit_id, Arc::clone(commit)))
        .collect();
    rebuild_catalog_with_live_roots_and_descriptors(
        history,
        execution,
        &std::collections::BTreeMap::new(),
        &std::collections::BTreeMap::new(),
        &admitted,
        symbols,
    )?;
    Ok(())
}
fn array_stop(
    cause: worth_execution::ExecutionRequestArrayDenial,
) -> RelationalHistoryRecoveryDenial {
    RelationalHistoryRecoveryDenial::Source(match cause {
        worth_execution::ExecutionRequestArrayDenial::Request(cause) => {
            contracts::durability::data::recovery_stop::RelationalRecoveryStop::Request(
                cause.into(),
            )
        }
        worth_execution::ExecutionRequestArrayDenial::Allocation(cause) => {
            contracts::durability::data::recovery_stop::RelationalRecoveryStop::Allocation(cause)
        }
    })
}
