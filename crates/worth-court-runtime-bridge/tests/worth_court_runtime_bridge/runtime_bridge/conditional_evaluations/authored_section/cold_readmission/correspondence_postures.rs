//! Actual definition phases retain or exclude the original installed read basis.
use super::*;

#[test]
fn definition_preparation_keeps_installed_correspondence_while_pending_refuses() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let (definitions, advances) = source(&execution);
    let medium = recorded_cut::RecordedCut::new(
        &definitions,
        &advances,
        &execution,
        "22222222-2222-4222-8222-222222222222",
    );
    let owner = Box::new(source_free_builder())
        .readmit_authored_owner(
            medium,
            SignalRecordedIdentityContribution::empty(branch().owner()),
            &mut Bindings { providers: 0 },
            &execution,
        )
        .unwrap();
    let acting = definitions[0].branch();
    let source = owner.signal_owner().source_port();
    let descriptor = source.observe(acting).unwrap();
    let handle = source.readmit_descriptor(&descriptor).unwrap();
    let original = owner
        .with_authored_branch(acting, |view| {
            view.with_conditional_definitions(|items| items.next().unwrap())
        })
        .unwrap();
    let successor = || BridgeOwnedConditionalInstallationRequest {
        contract: original.contract().clone(),
        location: original.location().clone(),
        dependencies: Vec::new(),
        providers: BridgeConditionalProviderSet::new()
            .authored_trigger(Requested)
            .authored_compute(VersionCompute),
    };
    let correspondence = owner.runtime_world_correspondence_port();
    let baseline = correspondence
        .admit_signal_baseline(handle.basis())
        .unwrap();
    correspondence.compare_current_exact(&baseline).unwrap();
    let prepared = owner
        .prepare_conditional_definition_publication(&execution, acting, &original, successor())
        .unwrap();
    assert!(matches!(owner.with_authored_branch(acting, |_| ()),
        Err(BridgeAuthoredBranchDenial::Preparing { branch, node })
            if branch == acting && node == original.signal_node()));
    correspondence.compare_current_exact(&baseline).unwrap();
    assert_eq!(source.observe(acting).unwrap(), descriptor);
    drop(prepared);
    correspondence.compare_current_exact(&baseline).unwrap();
    let task = owner
        .begin_conditional_definition_publication(&execution, &handle, &original, successor())
        .unwrap();
    assert!(matches!(owner.with_authored_branch(acting, |_| ()),
        Err(BridgeAuthoredBranchDenial::Pending { original, .. }) if original == task.original()));
    assert_eq!(
        correspondence.compare_current_exact(&baseline).unwrap_err(),
        RuntimeWorldCorrespondenceAdmissionDenial::AuthoredBranchUnavailable {
            signal_graph_instance_id: original.signal_graph_instance_id(),
        }
    );
    assert_eq!(source.observe(acting).unwrap(), descriptor);
    // The immutable read-cut refuses every I/O call: this court has only
    // prepared originals and does not claim a completed owner effect.
    drop(task);
}
