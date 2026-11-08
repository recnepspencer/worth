//! Actual fork originals publish Created frames and reconstruct authored siblings.
use super::{medium_fixture as media, owner_fixture as fixture};
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_execution::ExecutionRequest;
use worth_signal::facade::durability::*;
use worth_signal::facade::{branch::*, *};

#[test]
fn durable_fork_resumes_the_same_original_and_reopens_inherited_authored_state_cold() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let mut reference = fixture::seed(medium.as_ref(), &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let port = owner.source_port();
    let parent = reference.heads.values().next().unwrap();
    let source = port.readmit_descriptor(parent).unwrap();
    let memory_source = reference
        .memory
        .branch_handle(SignalBranchId(parent.branch().ordinal()))
        .unwrap();
    let memory_basis = reference
        .memory
        .observe_signal_branch_basis(memory_source)
        .unwrap();
    let expected = reference
        .memory
        .fork_signal_branch("durable-child", &memory_basis)
        .unwrap();
    let reservation = owner
        .mutation_port()
        .reserve_fork(
            &execution,
            &source,
            validate_signal_branch_name("durable-child").unwrap(),
        )
        .unwrap();
    let target = reservation.target();
    assert_eq!(target.ordinal(), expected.created_branch().id.0);
    let task = owner
        .mutation_port()
        .begin_fork(
            execution,
            reservation,
            Box::new(SignalOnlyForkPublication),
            &SignalOwnerCancellationSource::new().token(),
        )
        .unwrap();
    let original = task.original();
    assert!(
        matches!(owner.mutation_port().begin_advance(execution, &source, &mut (), |_| {
        panic!("source remains reserved until its child is durable")
    }), Err(SignalDurableAdvanceDenial::BranchReserved { original: held }) if held == original)
    );
    assert_eq!(port.branches().unwrap().len(), 3);
    drop(task);
    let output = fixture::finish(
        owner.mutation_port().resume_original(original).unwrap(),
        &execution,
    );
    assert_eq!(output.artifact().identity(), original);
    assert_eq!(output.evidence().position(), 5);
    let SignalDurableOwnerEffect::Forked(fork) = output.effect() else {
        panic!("real fork effect required")
    };
    assert_eq!(fork.result().created_branch(), expected.created_branch());
    let definition = owner
        .read_definition_source(output.branch().descriptor(), &execution)
        .unwrap();
    assert_eq!(definition.predecessor(), Some(parent.definition()));
    assert_eq!(definition.conditional_occurrences().count(), 1);
    assert!(
        matches!(output.artifact().change(), SignalAuthoredBranchChange::Created { name, parent: Some(recorded) }
        if name == "durable-child" && recorded == parent)
    );
    let descriptor = output.branch().descriptor().clone();
    let original_point = output.evidence().source_point();
    assert_eq!(output.branch().source_point(), &original_point);
    let branches = port.branches().unwrap();
    assert_eq!(
        branches
            .iter()
            .map(|(_, handle)| handle.clone())
            .collect::<Vec<_>>(),
        reference.memory.known_branches()
    );
    drop((definition, output, source, port, owner));
    media::close(serving, medium, &execution);
    let (serving, medium) = media::open_with_identity(
        directory.path(),
        false,
        2,
        "22222222-2222-4222-8222-222222222222",
        &execution,
    );
    let owner = fixture::readmit(medium.clone(), &execution);
    assert_eq!(owner.source_port().observe(target).unwrap(), descriptor);
    let actual = owner.source_port().readmit_descriptor(&descriptor).unwrap();
    assert_eq!(actual.source_point(), &original_point);
    let expected_handle = reference
        .memory
        .branch_handle(expected.created_branch().id)
        .unwrap();
    reference
        .memory
        .switch_branch(expected_handle.clone())
        .unwrap();
    let (image, runtime) = owner
        .source_port()
        .capture_authored_images(&actual, &execution)
        .unwrap();
    assert_eq!(
        image.as_bytes(),
        reference
            .memory
            .graph()
            .capture_authored_image(&execution, target)
            .unwrap()
            .as_bytes()
    );
    assert_eq!(
        runtime.as_bytes(),
        reference
            .memory
            .config()
            .capture_authored_image(
                reference.memory.graph(),
                &fixture::vocabulary(),
                target,
                &execution
            )
            .unwrap()
            .as_bytes()
    );
    let calls = AtomicUsize::new(0);
    let evaluate = |_: &mut EvaluationContext<'_, ()>| -> Result<AspectVersion, SignalError> {
        calls.fetch_add(1, Ordering::Relaxed);
        Ok(AspectVersion::from_updates([(Aspect::new(0), 71)]))
    };
    let memory_basis = reference
        .memory
        .observe_signal_branch_basis(expected_handle)
        .unwrap();
    let mut expected_value = None;
    reference
        .memory
        .advance_signal_branch(execution, &mut (), &memory_basis, |tx| {
            expected_value = Some(tx.target(reference.node).read(&evaluate)?);
            Ok(())
        })
        .unwrap();
    let before = calls.load(Ordering::Relaxed);
    let task = owner
        .mutation_port()
        .begin_advance(execution, &actual, &mut (), |tx| {
            assert_eq!(
                Some(tx.target(reference.node).read(&evaluate)?),
                expected_value
            );
            Ok(())
        })
        .unwrap();
    assert_eq!(calls.load(Ordering::Relaxed), before + 1);
    drop(fixture::finish(task, &execution));
    drop((actual, owner));
    media::close(serving, medium, &execution);
}

#[test]
fn composite_fork_refusals_release_destination_and_original_grant_resumes_the_same_creation() {
    use worth_proof::{
        CompositeSignalPublicationAdmissionDenial, InstalledCompositeSignalPublicationOwner,
    };
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let reference = fixture::seed(medium.as_ref(), &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let source = owner
        .source_port()
        .readmit_descriptor(reference.heads.values().next().unwrap())
        .unwrap();
    let issuer = InstalledCompositeSignalPublicationOwner::fresh();
    let foreign = InstalledCompositeSignalPublicationOwner::fresh();
    let composite = owner
        .bind_composite_owner(issuer.authority(), &execution)
        .unwrap();
    let association = super::composite_owner::association(79);
    let cancellation = SignalOwnerCancellationSource::new();
    enum Mode {
        Foreign,
        Cancelled,
        Admitted,
    }
    for mode in [Mode::Foreign, Mode::Cancelled, Mode::Admitted] {
        let reservation = composite
            .mutation_port()
            .reserve_fork(
                &execution,
                &source,
                validate_signal_branch_name("composite-child").unwrap(),
            )
            .unwrap();
        let binding = composite
            .mutation_port()
            .prepare_composite_fork_binding(&execution, reservation, association.clone())
            .unwrap();
        let commitment = binding.commitment();
        let grant = match mode {
            Mode::Foreign => foreign.admit_original(commitment),
            Mode::Cancelled | Mode::Admitted => issuer.admit_original(commitment),
        };
        if matches!(mode, Mode::Cancelled) {
            cancellation.cancel();
        }
        let token = if matches!(mode, Mode::Cancelled) {
            cancellation.token()
        } else {
            SignalOwnerCancellationSource::new().token()
        };
        let result = composite.mutation_port().begin_composite_fork(
            execution,
            binding,
            grant,
            Box::new(SignalOnlyForkPublication),
            &token,
        );
        match mode {
            Mode::Foreign => assert!(matches!(
                result,
                Err(SignalDurableAdvanceDenial::CompositeAdmission(
                    CompositeSignalPublicationAdmissionDenial::ForeignOwner
                ))
            )),
            Mode::Cancelled => assert!(matches!(
                result,
                Err(SignalDurableAdvanceDenial::Fork(
                    SignalBranchForkOperationDenial::CancelledNoMovement
                ))
            )),
            Mode::Admitted => {
                let task = result.unwrap();
                let original = task.original();
                drop(task);
                let resumed = composite
                    .mutation_port()
                    .resume_composite_original(original, issuer.admit_original(commitment))
                    .unwrap();
                let output = fixture::finish(resumed, &execution);
                assert_eq!(output.artifact().identity(), original);
                assert_eq!(output.artifact().association(), Some(&association));
                assert_eq!(output.evidence().position(), 5);
            }
        }
        assert_eq!(
            owner.source_port().branches().unwrap().len(),
            if matches!(mode, Mode::Admitted) { 4 } else { 3 }
        );
    }
    drop((source, composite, owner));
    media::close(serving, medium, &execution);
}
