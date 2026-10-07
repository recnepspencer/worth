use crate::product::FocusGroup;

pub(crate) struct FocusSelection {
    pub(crate) package: &'static str,
    pub(crate) target: &'static str,
    pub(crate) filter: String,
    pub(crate) process_binaries: bool,
}

impl FocusSelection {
    pub(crate) fn for_group(group: FocusGroup) -> Self {
        use FocusGroup::*;
        let filter = match group {
            EntryAdmission => concat!(
                "test(production_entry_) + test(before_first_checkpoint::) + ",
                "test(production_custody_serving::) + test(no_release_failed_ingest::) + ",
                "test(tier_epoch_reopen::)"
            ).into(),
            EntryRelease => concat!(
                "test(release_reopen::) + test(=completed_release_first_batch_survives_checkpoint_and_fresh_recovery) + ",
                "test(=terminal_release_survives_checkpoint_and_fresh_recovery) + ",
                "test(two_releases_) + test(two_distinct_released_generations_) + ",
                "test(release_candidate_) + test(=failed_ingest_then_independent_release_recovers_from_selected_controls)"
            ).into(),
            EntryCustody => format!(
                "(test(certified_release_serving::) - test({CAPACITY}) - test({POLICY})) + \
                 test(selected_custody_) + test(selected_empty_) + test(selected_tier_and_) + \
                 test(changed_selected_) + test(=genuine_release_custody_rejoins_store_media_before_serving)"
            ),
            EntryResidencyPolicy => format!("test({POLICY})"),
            EntryCapacity => format!("test({CAPACITY}) - test({POOL})"),
            EntryPool => format!("test({POOL})"),
            EntryPending => concat!(
                "test(pending_wal_world::) + test(pending_wal_fold::) + ",
                "test(selected_batch_multi_pending::) + test(three_batch_fold::) + ",
                "test(selected_tag7_pruned_wal::) + test(accumulator_only_pending_wal::) + ",
                "test(historical_release_only::) + test(mixed_retained_control::) + ",
                "test(retired_tail_append::) + test(maintenance_result::) + test(tier_release_pending_wal::)"
            ).into(),
            EntrySuccessor => "test(pending_successor_above_history::)".into(),
            EntryPublication => "test(published_above_checkpoint::) - test(published_above_checkpoint::damaged_history::) - test(published_above_checkpoint::limit_sweep::)".into(),
            EntryMediaDenials => "test(published_above_checkpoint::damaged_history::)".into(),
            EntryLimits => "test(published_above_checkpoint::limit_sweep::)".into(),
            PhaseOracle => "test(history::) + test(child_lifecycle::)".into(),
            PhaseCheckpoint => "test(checkpoint_crash::)".into(),
            PhaseAgreement => "(test(production::) - test(production::writer_crash::) - test(production::recovery_crash::)) + test(terminal_profiles::)".into(),
            PhaseRecovery => "test(production::recovery_crash::)".into(),
            PhaseMutation => MUTATION.into(),
            PhaseSuccessor => format!("test(production::writer_crash::) - ({MUTATION})"),
            PhaseObserver => "all()".into(),
        };
        let (package, target) = match group {
            PhaseObserver => ("worth-store-offline-verifier", "phase_eight_observer_cli"),
            PhaseOracle | PhaseCheckpoint | PhaseAgreement | PhaseRecovery | PhaseMutation
            | PhaseSuccessor => ("worth-store-recovery-runtime", "phase_eight_process"),
            _ => ("worth-store-recovery-runtime", "production_entry"),
        };
        Self {
            package,
            target,
            filter,
            process_binaries: target == "phase_eight_process",
        }
    }

    pub(crate) fn nextest_arguments(&self) -> Vec<String> {
        let mut arguments = [
            "nextest",
            "run",
            "--manifest-path",
            "Cargo.toml",
            "-p",
            self.package,
            "--test",
            self.target,
            "--no-tests=fail",
            "--no-fail-fast",
            "--test-threads",
            "4",
            "--filterset",
            &self.filter,
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        if self.package == "worth-store-recovery-runtime" {
            arguments.extend(["--features".into(), "certification-test-authority".into()]);
        }
        arguments
    }
}

const CAPACITY: &str = "certified_release_serving::release_capacity_budget::";
const POOL: &str = "certified_release_serving::release_capacity_budget::recovery_pool_handoff::";
const POLICY: &str = "certified_release_serving::ledger_preparation::";
const MUTATION: &str = concat!(
    "test(=production::writer_crash::killed_writer_recovers_after_each_mutation_effect_boundary) + ",
    "test(production::writer_crash::rewrite_recovery::) + ",
    "test(production::writer_crash::capacity_transition::)"
);
