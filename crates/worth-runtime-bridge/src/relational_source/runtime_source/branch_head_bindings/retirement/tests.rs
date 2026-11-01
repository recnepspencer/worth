//! Original owner gates and genuinely issued heads across the scalar wrap edge.
use super::*;
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;

fn assert_head_gate_busy(
    task: &mut RelationalBridgeBranchHeadRetirement,
    source: &RuntimeBridgeRelationalSource,
    request: &ExecutionRequest<'_, '_>,
) {
    let gate = source.branch_head_bindings.entries.lock().unwrap();
    let head = &gate[&task.lease.branch_identity];
    let original = head.binding_id;
    assert_eq!(task.advance(request, source), Progress::Busy);
    assert_eq!(gate[&task.lease.branch_identity].binding_id, original);
    drop(gate);
}

pub(in crate::relational_source) fn assert_wrapped_successor_survives_abandonment(
    source: &RuntimeBridgeRelationalSource,
    foreign: &RuntimeBridgeRelationalSource,
    basis: &AdmittedRelationalBranchBasis,
    request: &ExecutionRequest<'_, '_>,
) {
    let original = source.bind_branch_head_basis_for_bridge(basis).unwrap();
    assert_eq!(original.binding_id, 0);
    let branch = original.branch_identity.clone();
    let mut abandoned = original.into_retirement();
    assert_head_gate_busy(&mut abandoned, source, request);
    assert_eq!(
        abandoned.advance(request, foreign),
        Progress::Refused(Denial::ForeignSource)
    );
    source
        .branch_head_bindings
        .next_binding_id
        .store(u64::MAX, Ordering::Relaxed);
    let maximum = source.bind_branch_head_basis_for_bridge(basis).unwrap();
    assert_eq!(maximum.binding_id, u64::MAX);
    let successor = source.bind_branch_head_basis_for_bridge(basis).unwrap();
    assert_eq!(successor.binding_id, 0);
    assert_ne!(
        successor.snapshot_identity(),
        abandoned.lease.snapshot_identity()
    );
    drop(abandoned); // Deliberately exercise unbudgeted abandonment after refusals.
    let (_, selected) = source.branch_head_bindings.resolve(&branch).unwrap();
    assert_eq!(&selected, successor.snapshot_identity());
    let mut earlier = maximum.into_retirement();
    for _ in 0..16 {
        match earlier.advance(request, source) {
            Progress::Advanced => {}
            Progress::Complete => break,
            progress => panic!("unexpected superseded retirement progress: {progress:?}"),
        }
    }
    let earlier = earlier.into_retired().unwrap();
    let (_, selected) = source.branch_head_bindings.resolve(&branch).unwrap();
    assert_eq!(&selected, successor.snapshot_identity());
    earlier.reclaim();
    let mut task = successor.into_retirement();
    for _ in 0..16 {
        match task.advance(request, source) {
            Progress::Advanced => {}
            Progress::Complete => {
                task.into_retired().unwrap().reclaim();
                return;
            }
            progress => panic!("unexpected original retirement progress: {progress:?}"),
        }
    }
    panic!("successor retirement did not complete");
}
