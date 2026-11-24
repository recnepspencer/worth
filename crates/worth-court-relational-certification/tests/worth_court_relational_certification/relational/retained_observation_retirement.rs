//! Retained observations and basis retirement preserve exact issued root custody.
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    change_source::{
        RelationalReadCustodyHolders, RelationalRetainedObservationRetirement,
        RelationalRetainedObservationRetirementProgress,
    },
    runtime::RelationalRuntimeCloseDenial,
};
use worth_testkit_relational::durable_publication::{create_entity_outcome, memory_runtime};

#[test]
fn retained_observation_independent_alias_remains_readable_and_vetoes_close() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    use worth_relational::facade::{
        identity::{KindId, PartitionId},
        transactions::CreatedEntityRef,
    };
    let key = "original retained reader";
    let result = create_entity_outcome(&request, &runtime, key);
    let entity = result
        .created_entity(&CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: key.into(),
        })
        .unwrap();
    runtime
        .snapshots()
        .release_snapshot(&result.snapshot)
        .unwrap();
    drop(result);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let alias = basis.observation();
    let retained = runtime.retain_observation_snapshot(&basis).unwrap();
    drop(basis);
    let mut task = retained.into_retirement();
    finish(&mut task, &request);
    let retired = task.into_retired().unwrap();
    assert!(runtime
        .entity_record_at_observation(&request, &alias, entity)
        .unwrap()
        .is_some());
    let refused = runtime.prepare_close().unwrap_err();
    assert!(
        matches!(refused.reason(), RelationalRuntimeCloseDenial::ReadCustodyRetained(
            RelationalReadCustodyHolders::SnapshotsAndHistory {
                active_snapshots, detached_history,
            }
        ) if active_snapshots != 0 || detached_history)
    );
    let runtime = refused.into_runtime();
    assert!(runtime
        .entity_record_at_observation(&request, &alias, entity)
        .unwrap()
        .is_some());
    drop(alias);
    runtime.prepare_close().unwrap().finish();
    retired.reclaim();
}

#[test]
fn retained_observation_original_root_allocation_survives_retirement_until_reclaim() {
    let stop = CancellationSource::new();
    let host = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    let request = ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let root = basis.custody_witness();
    let retained = runtime.retain_observation_snapshot(&basis).unwrap();
    drop(basis);
    let mut task = retained.into_retirement();
    finish(&mut task, &request);
    stop.cancel();
    assert_eq!(
        task.advance(&request),
        RelationalRetainedObservationRetirementProgress::Complete
    );
    let retired = task.into_retired().unwrap();
    runtime.prepare_close().unwrap().finish();
    assert!(
        root.root_retained(),
        "the original root allocation remains in retired custody"
    );
    retired.reclaim();
    assert!(
        !root.root_retained(),
        "reclaim releases the same original root allocation"
    );
}

fn finish(task: &mut RelationalRetainedObservationRetirement, request: &ExecutionRequest<'_, '_>) {
    for _ in 0..12 {
        match task.advance(request) {
            RelationalRetainedObservationRetirementProgress::Advanced => {}
            RelationalRetainedObservationRetirementProgress::Complete => return,
            progress => panic!("unexpected retirement progress: {progress:?}"),
        }
    }
    panic!("original custody did not complete retirement");
}

#[test]
fn original_basis_retirement_preserves_independent_reads_and_retained_payload() {
    use worth_relational::facade::branch::RelationalBranchBasisRetirementDenial as Denial;
    use worth_relational::facade::branch::RelationalBranchBasisRetirementProgress as Progress;
    let stop = CancellationSource::new();
    let live = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let stopped = SerialRequest::from_memory(SerialMemoryBudget::new(None), stop.token(), None);
    stop.cancel();
    let request = ExecutionRequest::serial(&live);
    let runtime = memory_runtime();
    let foreign = memory_runtime();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let independent = basis.clone();
    let root = basis.custody_witness();
    let mut task = basis.into_retirement();
    assert!(matches!(
        task.advance(&request, &foreign),
        Err(Denial::ForeignOwner)
    ));
    assert!(matches!(
        task.advance(&ExecutionRequest::serial(&stopped), &runtime),
        Err(Denial::Execution(
            worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)
        ))
    ));
    assert_eq!(
        task.advance(&request, &runtime).unwrap(),
        Progress::Complete
    );
    let retired_share = task.into_retired().unwrap();
    let refused = runtime.prepare_close().unwrap_err();
    assert!(
        matches!(refused.reason(), RelationalRuntimeCloseDenial::ReadCustodyRetained(
            RelationalReadCustodyHolders::SnapshotsAndHistory {
                active_snapshots, detached_history,
            }
        ) if active_snapshots != 0 || detached_history)
    );
    let runtime = refused.into_runtime();
    assert_eq!(independent.identity(), &runtime.main_branch_identity());
    let mut last = independent.into_retirement();
    let mut complete = false;
    for _ in 0..8 {
        if last.advance(&request, &runtime).unwrap() == Progress::Complete {
            complete = true;
            break;
        }
    }
    assert!(
        complete,
        "the original Native basis must finish its actual lease retirement"
    );
    let retired_last = last.into_retired().unwrap();
    runtime.prepare_close().unwrap().finish();
    assert!(
        root.root_retained(),
        "the original root stays in retired payload custody"
    );
    drop(retired_share);
    drop(retired_last);
    assert!(
        !root.root_retained(),
        "explicit reclaim destroys the same original allocation"
    );
    foreign.prepare_close().unwrap().finish();
}
