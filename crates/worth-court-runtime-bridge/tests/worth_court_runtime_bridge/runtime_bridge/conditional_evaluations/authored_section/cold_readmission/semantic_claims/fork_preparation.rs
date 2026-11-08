//! Preparing a real fork rebinds performed claims without publishing or reading sources.
use super::*;
use worth_signal::facade::branch::{validate_signal_branch_name, SignalOwnerCancellationSource};

#[test]
fn authored_fork_preparation_rebases_performed_semantic_claims_without_source_contacts() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let contacts = Arc::new(native_absence::Contacts::default());
    let readers = Arc::new(AtomicUsize::new(0));
    let (program, original) =
        native_absence::installed_builder(Arc::clone(&contacts), Arc::clone(&readers));
    let branch = branch();
    let node = original.signal_node();
    let occurrence = DurableSignalConditionalDefinitionOccurrence::from_recorded(
        DurableSignalNodeIdentity::from_recorded(branch, node.index(), node.generation()),
        DurableSignalConditionalOccurrence::from_recorded(branch.owner(), 1),
    )
    .unwrap();
    let definition = program
        .capture_authored_definition(
            &execution,
            SignalDefinitionTransitionIdentity::from_recorded(branch.owner(), 1),
            branch,
            &[occurrence],
        )
        .unwrap();
    let descriptor = SignalDurableBranchDescriptor::from_recorded(
        DurableSignalGeneration::from_recorded(
            DurableSignalCellIncarnation::from_recorded(branch, 1),
            0,
        ),
        None,
        DurableSignalAdmissionIdentity::from_recorded(branch.owner(), 1),
        definition.reference(),
    )
    .unwrap();
    let advance = SignalBranchAdvanceArtifact::capture(
        &execution,
        SignalBranchAdvanceIdentity::from_recorded(branch.owner(), 1),
        descriptor.clone(),
        None,
        SignalAuthoredBranchChange::Created {
            name: "semantic-branch".into(),
            parent: None,
        },
    )
    .unwrap();
    let medium = recorded_cut::RecordedCut::new(
        &[definition],
        &[advance],
        &execution,
        "22222222-2222-4222-8222-222222222222",
    );
    let (fresh, fresh_lowering) =
        native_absence::installed_builder(Arc::clone(&contacts), Arc::clone(&readers));
    let mut bindings = Bindings {
        candidate: fresh_lowering
            .semantic_dependencies()
            .next()
            .unwrap()
            .clone(),
        contacts: Arc::clone(&contacts),
        rebound: 0,
    };
    let owner = fresh
        .readmit_authored_owner(
            medium,
            SignalRecordedIdentityContribution::empty(branch.owner()),
            &mut bindings,
            &execution,
        )
        .unwrap();
    let port = owner.signal_owner().source_port();
    let handle = port.readmit_descriptor(&descriptor).unwrap();
    let reservation = owner
        .signal_owner()
        .mutation_port()
        .reserve_fork(
            &execution,
            &handle,
            validate_signal_branch_name("semantic-child").unwrap(),
        )
        .unwrap();
    let target = reservation.target();
    // This read-cut refuses every publish call. Preparation must build and validate
    // declarations, performed slots, providers and the fresh ledger before any I/O.
    let task = owner
        .begin_authored_branch_fork(
            execution,
            reservation,
            &SignalOwnerCancellationSource::new().token(),
        )
        .unwrap();
    assert!(matches!(owner.with_authored_branch(target, |_| ()),
        Err(BridgeAuthoredBranchDenial::ForkReserved { branch }) if branch == target));
    assert!(matches!(owner.with_authored_branch(branch, |_| ()),
        Err(BridgeAuthoredBranchDenial::Pending { original, .. }) if original == task.original()));
    assert_eq!(port.branches().unwrap().len(), 1);
    assert_eq!(port.observe(branch).unwrap(), descriptor);
    assert_eq!(contacts.reads.load(Ordering::SeqCst), 0);
    assert_eq!(contacts.predicates.load(Ordering::SeqCst), 0);
    assert_eq!(contacts.computes.load(Ordering::SeqCst), 0);
    assert_eq!(readers.load(Ordering::SeqCst), 0);
    drop((task, port, handle, owner));
}
