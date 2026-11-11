//! Prepare the recovered source inventory before its single ledger application.
use super::*;
impl HistorySubsystem {
    pub(crate) fn install_recovered_canonical_inventory(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commits: Vec<Arc<crate::history::data::PositionedCanonicalCommit>>,
        symbols: &crate::symbols::data::StringInterner,
    ) -> Result<(), RelationalHistoryRecoveryDenial> {
        let mut prepared =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*execution, commits.len())
                .map_err(array_stop)?;
        let mut seen = worth_execution::ExecutionOrderedMap::default();
        for commit in &commits {
            if seen.get(&commit.commit.commit_id).is_some() {
                return Err("duplicate recovered source commit".to_owned().into());
            }
            seen.insert_cloned_native_for_request(&commit.commit.commit_id, &(), *execution)
                .map_err(|cause| {
                    RelationalHistoryRecoveryDenial::Source(
                        crate::durability::checkpoint_history::capture::map_stop(cause),
                    )
                })?;
            prepared
                .push(self.prepare_recovered_source_catalog(execution, commit)?)
                .map_err(array_stop)?;
        }
        let prepared = prepared.seal().map_err(array_stop)?;
        self.with_ledger_mut(|ledger| {
            for plan in &prepared {
                if !ledger.commit_catalog.source_replacement_matches(plan.artifact.commit_id(), plan.expected.as_ref()) {
                    return Err(RelationalHistoryRecoveryDenial::Catalog(crate::history::RelationalCommitCatalogAppendDenial::ChangedDuringPreparation));
                }
            }
            for (commit, plan) in commits.iter().zip(&prepared) {
                ledger.commit_catalog.install_source_recovery(plan.expected.as_ref(), Arc::clone(&plan.artifact))
                    .expect("same locked catalog validated every expected source before effects");
                ledger.commit_envelopes.insert(commit.commit.commit_id, plan.artifact.envelope_residency());
                ledger.patch_stream_index.insert(commit.position(), commit.commit.commit_id);
            }
            Ok(())
        })?;
        let admitted = commits
            .iter()
            .map(|commit| (commit.commit.commit_id, Arc::clone(commit)))
            .collect();
        self.rebuild_catalog_with_live_roots_and_descriptors(
            execution,
            &std::collections::BTreeMap::new(),
            &std::collections::BTreeMap::new(),
            &admitted,
            symbols,
        )?;
        self.rebuild_recovered_canonical_routes(commits)
            .map_err(|detail| RelationalHistoryRecoveryDenial::Invalid(detail.to_owned()))
    }
}
fn array_stop(
    cause: worth_execution::ExecutionRequestArrayDenial,
) -> RelationalHistoryRecoveryDenial {
    RelationalHistoryRecoveryDenial::Source(match cause {
        worth_execution::ExecutionRequestArrayDenial::Request(cause) => {
            crate::durability::data::RelationalRecoveryStop::Request(cause.into())
        }
        worth_execution::ExecutionRequestArrayDenial::Allocation(cause) => {
            crate::durability::data::RelationalRecoveryStop::Allocation(cause)
        }
    })
}
