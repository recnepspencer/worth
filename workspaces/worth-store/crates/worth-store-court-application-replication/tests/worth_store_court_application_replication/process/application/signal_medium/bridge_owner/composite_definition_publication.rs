//! The concrete composing issuer controls the same conditional publication original.
use super::definition_publication::successor;
use super::*;
use worth_proof::InstalledCompositeSignalPublicationOwner;
use worth_runtime_bridge::facade::*;

#[test]
fn composite_conditional_publication_rejects_foreign_grants_and_reopens_the_same_original_cold() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let reference = source::reference(&execution);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    source::seed(medium.as_ref(), &reference, &execution);
    let owner = readmit(medium.clone(), &execution);
    let branch = reference.definitions[0].branch();
    let descriptor = owner.signal_owner().source_port().observe(branch).unwrap();
    let head = owner
        .signal_owner()
        .source_port()
        .readmit_descriptor(&descriptor)
        .unwrap();
    let predecessor = owner
        .with_authored_branch(branch, |view| {
            view.with_conditional_definitions(|items| items.next().unwrap())
        })
        .unwrap();
    let issuer = InstalledCompositeSignalPublicationOwner::fresh();
    let foreign = InstalledCompositeSignalPublicationOwner::fresh();
    let composite = owner
        .signal_owner()
        .bind_composite_owner(issuer.authority(), &execution)
        .unwrap();
    let association = super::super::composite_owner::association(173);
    let calls = AtomicUsize::new(0);
    let binding = composite
        .mutation_port()
        .prepare_composite_binding(&execution, head.basis(), association.clone())
        .unwrap();
    let grant = foreign.admit_original(binding.commitment());
    let refused = owner.begin_composite_conditional_definition_publication(
        &execution,
        binding,
        grant,
        &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
        &predecessor,
        successor(&predecessor),
        |_| {
            calls.fetch_add(1, Ordering::Relaxed);
            Ok(())
        },
    );
    assert!(
        matches!(refused, Err(BridgeDurableDefinitionPublicationDenial::Signal(cause)) if matches!(cause.as_ref(), SignalDurableAdvanceDenial::CompositeAdmission(_)))
    );
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert_eq!(
        owner.signal_owner().source_port().observe(branch).unwrap(),
        descriptor
    );
    assert!(owner
        .with_authored_branch(branch, |view| Arc::ptr_eq(
            &view.with_conditional_definitions(|items| items.next().unwrap()),
            &predecessor
        ))
        .unwrap());
    assert!(
        matches!(owner.begin_conditional_definition_publication(&execution, &head, &predecessor, successor(&predecessor)), Err(BridgeDurableDefinitionPublicationDenial::Signal(cause)) if matches!(cause.as_ref(), SignalDurableAdvanceDenial::CompositePublicationRequired))
    );
    let binding = composite
        .mutation_port()
        .prepare_composite_binding(&execution, head.basis(), association.clone())
        .unwrap();
    let commitment = binding.commitment();
    let task = owner
        .begin_composite_conditional_definition_publication(
            &execution,
            binding,
            issuer.admit_original(commitment),
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
            &predecessor,
            successor(&predecessor),
            |_| {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(())
            },
        )
        .unwrap();
    let original = task.original();
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(
        matches!(owner.with_authored_branch(branch, |_| ()), Err(BridgeAuthoredBranchDenial::Pending { original: held, .. }) if held == original)
    );
    drop(task);
    let output = owner_fixture::finish(
        composite
            .mutation_port()
            .resume_composite_original(original, issuer.admit_original(commitment))
            .unwrap(),
        &execution,
    );
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert_eq!(output.artifact().identity(), original);
    assert_eq!(output.artifact().association(), Some(&association));
    assert_ne!(
        output.branch().descriptor().definition(),
        descriptor.definition()
    );
    let point = output.evidence().source_point();
    let admitted = owner
        .signal_owner()
        .readmit_source_point(&point, &execution)
        .unwrap();
    assert_eq!(admitted.original().association(), Some(&association));
    let current = owner
        .with_authored_branch(branch, |view| {
            view.with_conditional_definitions(|items| items.next().unwrap())
        })
        .unwrap();
    assert!(matches!(
        current.contract().condition(),
        BridgeConditionalCondition::Always
    ));
    let image = owner
        .signal_owner()
        .capture_authored_home_image(&execution)
        .unwrap();
    drop((
        admitted,
        current,
        output,
        predecessor,
        head,
        composite,
        owner,
    ));
    media::close(serving, medium, &execution);
    let (serving, medium) =
        super::super::owner_home_image::reopen(directory.path(), image, &execution);
    let owner = readmit(medium.clone(), &execution);
    let admitted = owner
        .signal_owner()
        .readmit_source_point(&point, &execution)
        .unwrap();
    assert_eq!(admitted.original().identity(), original);
    assert_eq!(admitted.original().association(), Some(&association));
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(owner
        .with_authored_branch(branch, |view| matches!(
            view.with_conditional_definitions(|items| items.next().unwrap())
                .contract()
                .condition(),
            BridgeConditionalCondition::Always
        ))
        .unwrap());
    for other in reference.definitions.iter().skip(1) {
        assert!(owner
            .with_authored_branch(other.branch(), |view| view.with_conditional_definitions(
                |mut items| Iterator::all(&mut items, |lowering| matches!(
                    lowering.contract().condition(),
                    BridgeConditionalCondition::OnDemand
                ))
            ))
            .unwrap());
    }
    drop((admitted, owner));
    media::close(serving, medium, &execution);
}

#[test]
fn refused_composite_conditional_callback_releases_projection_and_preserves_the_predecessor() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let reference = source::reference(&execution);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    source::seed(medium.as_ref(), &reference, &execution);
    let owner = readmit(medium.clone(), &execution);
    let branch = reference.definitions[0].branch();
    let descriptor = owner.signal_owner().source_port().observe(branch).unwrap();
    let head = owner
        .signal_owner()
        .source_port()
        .readmit_descriptor(&descriptor)
        .unwrap();
    let predecessor = owner
        .with_authored_branch(branch, |view| {
            view.with_conditional_definitions(|items| items.next().unwrap())
        })
        .unwrap();
    let issuer = InstalledCompositeSignalPublicationOwner::fresh();
    let composite = owner
        .signal_owner()
        .bind_composite_owner(issuer.authority(), &execution)
        .unwrap();
    let association = super::super::composite_owner::association(181);
    let binding = composite
        .mutation_port()
        .prepare_composite_binding(&execution, head.basis(), association.clone())
        .unwrap();
    let grant = issuer.admit_original(binding.commitment());
    let calls = AtomicUsize::new(0);
    let refused = owner.begin_composite_conditional_definition_publication(
        &execution,
        binding,
        grant,
        &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
        &predecessor,
        successor(&predecessor),
        |_| {
            calls.fetch_add(1, Ordering::Relaxed);
            Err(SignalError::internal(
                "refuse this original isolated callback",
            ))
        },
    );
    assert!(
        matches!(refused, Err(BridgeDurableDefinitionPublicationDenial::Signal(cause)) if matches!(cause.as_ref(), SignalDurableAdvanceDenial::Mutation(_)))
    );
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert_eq!(
        owner.signal_owner().source_port().observe(branch).unwrap(),
        descriptor
    );
    assert!(owner
        .with_authored_branch(branch, |view| Arc::ptr_eq(
            &view.with_conditional_definitions(|items| items.next().unwrap()),
            &predecessor
        ))
        .unwrap());
    let binding = composite
        .mutation_port()
        .prepare_composite_binding(&execution, head.basis(), association)
        .unwrap();
    let grant = issuer.admit_original(binding.commitment());
    let task = owner
        .begin_composite_conditional_definition_publication(
            &execution,
            binding,
            grant,
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
            &predecessor,
            successor(&predecessor),
            |_| {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(())
            },
        )
        .unwrap();
    drop(owner_fixture::finish(task, &execution));
    assert_eq!(
        calls.load(Ordering::Relaxed),
        2,
        "only the explicitly new original executes after refusal"
    );
    drop((predecessor, head, composite, owner));
    media::close(serving, medium, &execution);
}
