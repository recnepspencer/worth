//! A bounded history operation demands its selected journal envelope only.
use std::sync::atomic::Ordering;
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;
use worth_relational::facade::storage::{RelationalStorageSubject, RelationalStorageUnavailable};
use worth_relational::facade::transactions::RelationalExecutionDenialCause;
use worth_relational::facade::{
    config::RelationalRuntimeProfile, durability::RecoveryVerificationMode,
    runtime::RelationalRuntimeApi,
};
use worth_testkit_relational::{
    cold_recovery::source_with_profile, durable_medium::TestDurableMedium,
};

#[derive(Clone, Copy, Debug)]
pub(super) enum EnvelopeSurface {
    VersionSummary,
    CommitInspection,
    RecentInspection,
    RecentBranchInspection,
    CompiledArtifact,
}

impl EnvelopeSurface {
    pub(super) const ALL: [Self; 5] = [
        Self::VersionSummary,
        Self::CommitInspection,
        Self::RecentInspection,
        Self::RecentBranchInspection,
        Self::CompiledArtifact,
    ];
    pub(super) fn read(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        receipt: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Result<usize, RelationalExecutionDenialCause> {
        Ok(match self {
            Self::CompiledArtifact => {
                runtime
                    .compiled_artifacts_authority()
                    .compile_execution_artifact(
                        execution,
                        receipt.commit_id,
                        vec![worth_relational::facade::identity::PartitionId::main()],
                    )
                    .map_err(|failure| match failure.authority_status {
                        worth_relational::facade::runtime::CompiledArtifactAuthorityStatus::ReadRequest(
                            cause,
                        ) => cause,
                        other => {
                            panic!("enabled fixture compile source must be readable: {other:?}")
                        }
                    })?
                    .compiled_record_count
            }
            Self::VersionSummary => runtime
                .history()
                .historical_committed_version(execution, receipt.version_id)?
                .expect("fixture's indexed version exists")
                .changed_record_count(),
            Self::CommitInspection => runtime
                .inspect_what_happened()
                .inspect_commit(execution, receipt.commit_id)?
                .expect("fixture's indexed commit exists")
                .changed_records
                .len(),
            Self::RecentBranchInspection => runtime
                .inspect_what_happened()
                .inspect_recent_commits(
                    execution,
                    &worth_relational::facade::inspection::RecentCommitInspectionRequest {
                        branch_id: Some(receipt.branch_id.clone()),
                        limit: 2,
                    },
                )?
                .commits
                .len(),
            Self::RecentInspection => runtime
                .inspect_what_happened()
                .inspect_recent_commits(
                    execution,
                    &worth_relational::facade::inspection::RecentCommitInspectionRequest {
                        branch_id: None,
                        limit: 2,
                    },
                )?
                .commits
                .len(),
        })
    }
}

#[test]
fn envelope_surfaces_keep_index_facts_readable_and_refuse_cold_body_damage() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in EnvelopeSurface::ALL {
        for damaged in [false, true] {
            let profile = match door {
                EnvelopeSurface::CompiledArtifact => RelationalRuntimeProfile::ChipSimulation,
                EnvelopeSurface::VersionSummary
                | EnvelopeSurface::CommitInspection
                | EnvelopeSurface::RecentInspection
                | EnvelopeSurface::RecentBranchInspection => {
                    RelationalRuntimeProfile::CertificationCore
                }
            };
            let original = source_with_profile(execution, 16, false, profile);
            let counter = original.source;
            counter.damaged.store(true, Ordering::Relaxed);
            let source = original.input;
            let position = match door {
                EnvelopeSurface::RecentInspection | EnvelopeSurface::RecentBranchInspection => {
                    *counter.commits.keys().rev().nth(1).unwrap()
                }
                _ => *counter.commits.keys().nth(3).unwrap(),
            };
            let medium = TestDurableMedium::new();
            let mut runtime = RelationalRuntimeApi::builder()
                .profile(profile)
                .schema_registry(worth_testkit_relational::durable_publication::schema())
                .build_durable(medium.attachment())
                .unwrap();
            let plan = runtime
                .durability()
                .recovery_plan_from_source(
                    execution,
                    source,
                    RecoveryVerificationMode::NormalRecoveryVerification,
                )
                .unwrap();
            runtime
                .durability_recovery()
                .recover(execution, plan)
                .unwrap();
            assert!(
                counter.journal_reads.lock().unwrap().is_empty(),
                "selected history never falls back to its retired journal"
            );
            let selected_before = counter.canonical_member_reads.lock().unwrap().len();
            let reads_before = counter.commit_reads.lock().unwrap().len();
            assert!(
                reads_before < 4,
                "reopen must not read the history population"
            );
            let member_reads = counter.reads.load(Ordering::Relaxed);
            let frontier = runtime.history().latest_patch_stream_position();
            let offered = medium.offered().len();
            counter.commits_damaged.store(damaged, Ordering::Relaxed);
            let receipt = runtime
                .history()
                .immutable_commit_receipt_at_patch_stream_position(position)
                .unwrap();
            assert_eq!(
                runtime
                    .history()
                    .immutable_commit_receipt_for_version(receipt.version_id),
                Some(receipt.clone())
            );
            assert_eq!(counter.commit_reads.lock().unwrap().len(), reads_before);
            let read = || door.read(execution, &runtime, &receipt);
            if damaged {
                let failure = read().unwrap_err();
                assert_eq!(
                    failure,
                    RelationalExecutionDenialCause::StorageUnavailable(
                        RelationalStorageUnavailable {
                            subject: RelationalStorageSubject::Envelope(position),
                            cause: Cause::Damaged,
                        }
                    ),
                    "{door:?}"
                );
                counter.commits_damaged.store(false, Ordering::Relaxed);
                assert_eq!(read().unwrap_err(), failure, "{door:?}");
            } else {
                let count = read().unwrap();
                assert!(count > 0, "{door:?}");
                assert_eq!(read().unwrap(), count, "{door:?}");
            }
            assert!(counter.journal_reads.lock().unwrap().is_empty());
            assert!(counter.canonical_member_reads.lock().unwrap().len() > selected_before);
            let reads = counter.commit_reads.lock().unwrap();
            assert!(!reads[reads_before..].is_empty(), "{door:?}");
            assert!(
                reads[reads_before..].iter().all(|read| *read == position),
                "{door:?}"
            );
            assert!(
                reads.len() - reads_before < 3,
                "warm reads and sticky damage do not reload: {door:?}"
            );
            assert_eq!(counter.reads.load(Ordering::Relaxed), member_reads);
            assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
            assert_eq!(medium.offered().len(), offered);
        }
    }
}
