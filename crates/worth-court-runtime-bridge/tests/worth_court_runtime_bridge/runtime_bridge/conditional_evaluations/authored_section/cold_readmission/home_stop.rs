//! The actual multi-branch owner rolls back an uncommitted close and fences a committed close.
use super::*;
#[test]
fn authored_home_stop_rolls_back_and_commits_the_same_multi_branch_owner() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let (definitions, advances) = source(&execution);
    let medium = recorded_cut::RecordedCut::new(
        &definitions,
        &advances,
        &execution,
        "22222222-2222-4222-8222-222222222222",
    );
    let mut bindings = Bindings { providers: 0 };
    let owner = Box::new(source_free_builder())
        .readmit_authored_owner(
            medium,
            SignalRecordedIdentityContribution::empty(branch().owner()),
            &mut bindings,
            &execution,
        )
        .unwrap();
    let source = owner.signal_owner().source_port();
    let branch = source.branches().unwrap()[0].0;
    let original = source.observe(branch).unwrap();
    let prepared = owner.signal_owner().prepare_home_stop(&execution).unwrap();
    assert!(matches!(
        source.observe(branch),
        Err(SignalDurableSourceDenial::Admission(
            worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::OwnerUnavailable(_)
        ))
    ));
    drop(prepared);
    assert_eq!(source.observe(branch).unwrap(), original);
    owner
        .signal_owner()
        .prepare_home_stop(&execution)
        .unwrap()
        .commit();
    assert!(matches!(
        source.observe(branch),
        Err(SignalDurableSourceDenial::Admission(
            worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::OwnerUnavailable(_)
        ))
    ));
    owner.signal_owner().commit_home_stop(&execution).unwrap();
}
