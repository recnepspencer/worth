//! Drive the exact cold-lineage failure classes through the installed Runtime.
use crate::lineage::data::LineageGraphRequest;

#[derive(Debug)]
pub(super) struct ColdLineagePageSource {
    pub(super) members: std::collections::BTreeMap<
        crate::durability::medium::RelationalDurableMemberLocator,
        worth_execution::ExecutionImmutableBytes,
    >,
    pub(super) failures: std::sync::Mutex<
        std::collections::BTreeMap<
            crate::durability::medium::RelationalDurableMemberLocator,
            crate::storage::data::RelationalStorageUnavailableCause,
        >,
    >,
    pub(super) reads: std::sync::atomic::AtomicUsize,
}
impl crate::durability::medium::RelationalDurableColdSource for ColdLineagePageSource {
    fn retain_members_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        _: &[crate::durability::medium::RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn crate::durability::medium::RelationalDurableColdSource>,
    ) -> Result<
        crate::durability::medium::RelationalDurableMemberCustody,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        crate::durability::medium::RelationalDurableMemberCustody::try_new_from_source_for_request(
            issuer,
            request,
            || Ok(()),
        )
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::durability::medium::RelationalDurableReadPayload,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        request.check_live().map_err(|cause| {
            crate::durability::medium::RelationalDurableOwnedReadRefusal::Request(cause.into())
        })?;
        Ok(Box::new(()))
    }
    fn read_member_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        _: &(dyn std::any::Any + Send + Sync),
        locator: crate::durability::medium::RelationalDurableMemberLocator,
        _: u64,
    ) -> Result<
        worth_execution::ExecutionImmutableBytes,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        request.check_live().map_err(|cause| {
            crate::durability::medium::RelationalDurableOwnedReadRefusal::Request(cause.into())
        })?;
        self.reads
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if let Some(cause) = self.failures.lock().unwrap().get(&locator).copied() {
            return Err(
                crate::durability::medium::RelationalDurableOwnedReadRefusal::Storage(cause),
            );
        }
        self.members.get(&locator).cloned().ok_or(
            crate::durability::medium::RelationalDurableOwnedReadRefusal::Storage(
                crate::storage::data::RelationalStorageUnavailableCause::MissingMember,
            ),
        )
    }
    fn retain_members(
        &self,
        _: &[crate::durability::medium::RelationalDurableMemberLocator],
    ) -> Result<
        crate::durability::medium::RelationalDurableMemberCustody,
        crate::storage::data::RelationalStorageUnavailableCause,
    > {
        Ok(crate::durability::medium::RelationalDurableMemberCustody::new(()))
    }
    fn read_member(
        &self,
        locator: crate::durability::medium::RelationalDurableMemberLocator,
        _: u64,
    ) -> Result<Vec<u8>, crate::storage::data::RelationalStorageUnavailableCause> {
        self.reads
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if let Some(cause) = self.failures.lock().unwrap().get(&locator).copied() {
            return Err(cause);
        }
        self.members
            .get(&locator)
            .map(|bytes| bytes.to_vec())
            .ok_or(crate::storage::data::RelationalStorageUnavailableCause::MissingMember)
    }
    fn read_commit(
        &self,
        _: crate::publication::patch::data::PatchStreamPosition,
        _: u64,
    ) -> Result<Vec<u8>, crate::storage::data::RelationalStorageUnavailableCause> {
        Err(crate::storage::data::RelationalStorageUnavailableCause::MissingMember)
    }
}

pub(super) fn with_cold_lineage_reopened(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    body: impl FnOnce(
        &crate::runtime::LineageSubsystem,
        &crate::lineage::access::LineageAccess<'_>,
        &crate::facade::history::HistoryAccess<'_>,
        &ColdLineagePageSource,
        (
            crate::durability::medium::RelationalDurableMemberLocator,
            u64,
            u64,
        ),
        (
            crate::durability::medium::RelationalDurableMemberLocator,
            u64,
            u64,
        ),
    ),
) {
    use crate::durability::data::{
        RecoveryVerificationMode, RelationalCheckpointArtifactPolicy, RelationalCheckpointMember,
    };
    use crate::durability::published_medium::{
        RelationalDurableCheckpointSource, RelationalDurableRecoverySource,
    };

    let producer = crate::tests::support::runtime_with_test_schema();
    let entity = crate::tests::support::create_entity(execution, &producer, "lineage-graph-seed");
    crate::tests::support::update_entity_and_release_snapshot(
        execution,
        &producer,
        entity,
        "lineage-graph-updated",
    );
    let _branch = crate::tests::support::create_branch_from_main(&producer, "peer");
    let native = producer
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let source = std::sync::Arc::new(ColdLineagePageSource {
        members: native
            .members()
            .iter()
            .map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.clone()),
                RelationalCheckpointMember::Carried { .. } => unreachable!(),
            })
            .collect(),
        failures: Default::default(),
        reads: Default::default(),
    });
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            native.covered_position().unwrap(),
            native.covered_branch_transitions().unwrap(),
            native,
        )),
        Vec::new(),
        Vec::new(),
        source.clone(),
    );
    let mut runtime = crate::tests::support::runtime_with_test_schema();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(runtime.lineage.node_count(), 0);
    assert_eq!(runtime.lineage.event_count(), 0);
    let dir = runtime
        .durability
        .checkpoint_selection()
        .snapshot()
        .unwrap();
    let node_page = &dir.manifest.lineage[0];
    let event_page = &dir.manifest.lineage_events[0];
    body(
        &runtime.lineage,
        &runtime.lineage_access(),
        &runtime.history(),
        &source,
        (node_page.member, node_page.first, node_page.last),
        (
            event_page.member,
            event_page.first_node,
            event_page.last_node,
        ),
    );
}

// Fixture adaptation (C2.4): restores the cfg(test)-only `Graph` door from
// ea2ffee250:cold_lineage_event_matrix and 81d41f5575:cold_lineage_matrix
// across all six failure classes (sticky and transient).
#[test]
fn graph_cold_lineage_doors_preserve_every_failure_class() {
    use crate::execution::RelationalExecutionDenialCause as Execution;
    use crate::identity::data::LineageId;
    use crate::storage::data::{
        RelationalStorageSubject as Subject, RelationalStorageUnavailable as Unavailable,
        RelationalStorageUnavailableCause as Cause,
    };
    use std::sync::atomic::Ordering;
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let failures = [
        (Cause::Damaged, true),
        (Cause::IntegrityMismatch, true),
        (Cause::MissingMember, true),
        (
            Cause::LimitExceeded {
                length: 9,
                limit: 8,
            },
            false,
        ),
        (Cause::MemoryExhausted, false),
        (Cause::SourceUnavailable, false),
    ];
    let req = || LineageGraphRequest {
        branch_id: crate::history::data::BranchId("main".into()),
        traversal_basis:
            crate::lineage::data::LineageGraphTraversalBasis::FullBranchGraphMaterialization,
    };
    for (event_fault, (cause, sticky)) in [false, true]
        .into_iter()
        .flat_map(|ef| failures.into_iter().map(move |f| (ef, f)))
    {
        with_cold_lineage_reopened(
            execution,
            |lineage,
             access,
             history,
             source,
             (n_mem, n_first, n_last),
             (e_mem, e_first, e_last)| {
                let frontier = history.latest_patch_stream_position();
                let (member, subject) = if event_fault {
                    (
                        e_mem,
                        Subject::Lineage {
                            first: LineageId(e_first),
                            last: LineageId(e_last),
                        },
                    )
                } else {
                    (
                        n_mem,
                        Subject::Lineage {
                            first: LineageId(n_first),
                            last: LineageId(n_last),
                        },
                    )
                };
                source.failures.lock().unwrap().insert(member, cause);
                let expected = Execution::StorageUnavailable(Unavailable { subject, cause });
                assert_eq!(access.graph(execution, req()), Err(expected));
                source.failures.lock().unwrap().clear();
                let reads = source.reads.load(Ordering::Relaxed);
                if sticky {
                    assert_eq!(access.graph(execution, req()), Err(expected));
                    assert_eq!(source.reads.load(Ordering::Relaxed), reads);
                } else {
                    let graph = access.graph(execution, req()).unwrap();
                    assert!(!graph.nodes().is_empty() && !graph.events().is_empty());
                    assert!(source.reads.load(Ordering::Relaxed) > reads);
                    assert!(lineage.node(graph.nodes()[0].lineage_id()).is_some());
                }
                assert_eq!(history.latest_patch_stream_position(), frontier);
            },
        );
    }
}

// Fixture adaptation (C2.4): restores the cfg(test)-only `Divergence` door
// from ea2ffee250:cold_lineage_event_matrix and 81d41f5575:cold_lineage_matrix
// across all six failure classes (sticky and transient).
#[test]
fn divergence_cold_lineage_doors_preserve_every_failure_class() {
    use crate::execution::RelationalExecutionDenialCause as Execution;
    use crate::identity::data::LineageId;
    use crate::lineage::data::LineageDivergenceRequest;
    use crate::storage::data::{
        RelationalStorageSubject as Subject, RelationalStorageUnavailable as Unavailable,
        RelationalStorageUnavailableCause as Cause,
    };
    use std::sync::atomic::Ordering;
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let failures = [
        (Cause::Damaged, true),
        (Cause::IntegrityMismatch, true),
        (Cause::MissingMember, true),
        (
            Cause::LimitExceeded {
                length: 9,
                limit: 8,
            },
            false,
        ),
        (Cause::MemoryExhausted, false),
        (Cause::SourceUnavailable, false),
    ];
    let req = || LineageDivergenceRequest {
        left_branch: crate::history::data::BranchId("main".into()),
        right_branch: crate::history::data::BranchId("peer".into()),
        traversal_basis:
            crate::lineage::data::LineageDivergenceTraversalBasis::FullBranchGraphComparison,
    };
    for (event_fault, (cause, sticky)) in [false, true]
        .into_iter()
        .flat_map(|ef| failures.into_iter().map(move |f| (ef, f)))
    {
        with_cold_lineage_reopened(
            execution,
            |lineage,
             access,
             history,
             source,
             (n_mem, n_first, n_last),
             (e_mem, e_first, e_last)| {
                let frontier = history.latest_patch_stream_position();
                let (member, subject) = if event_fault {
                    (
                        e_mem,
                        Subject::Lineage {
                            first: LineageId(e_first),
                            last: LineageId(e_last),
                        },
                    )
                } else {
                    (
                        n_mem,
                        Subject::Lineage {
                            first: LineageId(n_first),
                            last: LineageId(n_last),
                        },
                    )
                };
                source.failures.lock().unwrap().insert(member, cause);
                let expected = Execution::StorageUnavailable(Unavailable { subject, cause });
                assert_eq!(
                    access.divergence_between_branches(execution, req()),
                    Err(expected)
                );
                source.failures.lock().unwrap().clear();
                let reads = source.reads.load(Ordering::Relaxed);
                if sticky {
                    assert_eq!(
                        access.divergence_between_branches(execution, req()),
                        Err(expected)
                    );
                    assert_eq!(source.reads.load(Ordering::Relaxed), reads);
                } else {
                    let summary = access
                        .divergence_between_branches(execution, req())
                        .unwrap();
                    assert!(!summary.shared_lineage_ids().is_empty());
                    assert!(source.reads.load(Ordering::Relaxed) > reads);
                    assert!(lineage.node(summary.shared_lineage_ids()[0]).is_some());
                }
                assert_eq!(history.latest_patch_stream_position(), frontier);
            },
        );
    }
}
