//! Signal owner courts cross the actual Store port, without claiming Home integration.
use super::{medium_fixture as media, owner_fixture as fixture};
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::*;
use worth_signal::facade::durability::*;
use worth_signal::facade::*;

#[test]
fn several_branches_reopen_with_authored_definitions_and_values_recompute_on_first_demand() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let mut reference = fixture::seed(medium.as_ref(), &execution);
    media::close(serving, medium, &execution);
    let (serving, medium) = media::open(directory.path(), false, &execution);
    let mut owner = fixture::readmit(medium.clone(), &execution);
    let port = owner.source_port();
    assert_eq!(
        port.branches()
            .unwrap()
            .iter()
            .map(|(_, h)| h.clone())
            .collect::<Vec<_>>(),
        reference.memory.known_branches()
    );
    let contracts = owner.take_installation_contracts();
    assert_eq!(contracts.len(), 3);
    for (branch, rows) in &contracts {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].occurrence().node().branch(), *branch);
        assert!(matches!(
            rows[0].installed().semantic_condition(),
            SignalConditionalCondition::DeltaThreshold(_)
        ));
    }
    let calls = AtomicUsize::new(0);
    let evaluate = |_: &mut EvaluationContext<'_, ()>| -> Result<AspectVersion, SignalError> {
        calls.fetch_add(1, Ordering::Relaxed);
        Ok(AspectVersion::from_updates([(Aspect::new(0), 71)]))
    };
    for (branch, expected) in &reference.heads {
        assert_eq!(&port.observe(*branch).unwrap(), expected);
        let basis = port.readmit_descriptor(expected).unwrap();
        let (graph, runtime) = port.capture_authored_images(&basis, &execution).unwrap();
        assert_eq!(graph.as_bytes(), reference.images[branch].0);
        assert_eq!(runtime.as_bytes(), reference.images[branch].1);
        let memory_handle = reference
            .memory
            .branch_handle(SignalBranchId(branch.ordinal()))
            .unwrap();
        reference
            .memory
            .switch_branch(memory_handle.clone())
            .unwrap();
        let memory_basis = reference
            .memory
            .observe_signal_branch_basis(memory_handle)
            .unwrap();
        let mut memory_value = None;
        reference
            .memory
            .advance_signal_branch(execution, &mut (), &memory_basis, |tx| {
                memory_value = Some(tx.target(reference.node).read(&evaluate)?);
                Ok(())
            })
            .unwrap();
        let before = calls.load(Ordering::Relaxed);
        let mut actual = None;
        let task = owner
            .mutation_port()
            .begin_advance(execution, &basis, &mut (), |tx| {
                actual = Some(tx.target(reference.node).read(&evaluate)?);
                Ok(())
            })
            .unwrap();
        assert_eq!(actual, memory_value);
        assert_eq!(calls.load(Ordering::Relaxed), before + 1);
        let original = task.original();
        assert!(
            matches!(owner.mutation_port().begin_advance(execution, &basis, &mut (), |_| {
            panic!("the named original reservation must refuse a second callback")
        }), Err(SignalDurableAdvanceDenial::BranchReserved { original: held }) if held == original)
        );
        drop(owner.mutation_port().resume_original(original).unwrap());
        let output = fixture::finish(task, &execution);
        assert!(matches!(owner.mutation_port().resume_original(original),
            Err(SignalDurableAdvanceDenial::UnknownTask { original: delivered }) if delivered == original));
        assert_eq!(
            output.artifact().descriptor().generation().ordinal(),
            expected.generation().ordinal() + 1
        );
        assert_eq!(
            output.artifact().descriptor().definition(),
            expected.definition(),
            "a derived-only demand reuses original source"
        );
        assert_eq!(
            owner
                .read_original_advance(original, &execution)
                .unwrap()
                .as_bytes(),
            output.artifact().as_bytes()
        );
        let (basis, _, _) = output.into_parts();
        let after = calls.load(Ordering::Relaxed);
        let task = owner
            .mutation_port()
            .begin_advance(execution, &basis, &mut (), |tx| {
                assert_eq!(Some(tx.target(reference.node).read(&evaluate)?), actual);
                Ok(())
            })
            .unwrap();
        drop(fixture::finish(task, &execution));
        assert_eq!(
            calls.load(Ordering::Relaxed),
            after,
            "a serving owner's derived cache remains reusable"
        );
    }
    let source = medium.capture_recovery_source(&execution).unwrap();
    assert_eq!(
        source
            .frontier(SignalDurableJournal::Definitions, &execution)
            .unwrap(),
        4
    );
    drop((source, contracts, port, owner));
    media::close(serving, medium, &execution);
    let (serving, medium) = media::open_with_identity(
        directory.path(),
        false,
        3,
        "22222222-2222-4222-8222-222222222222",
        &execution,
    );
    let owner = fixture::readmit(medium.clone(), &execution);
    let before = calls.load(Ordering::Relaxed);
    for (branch, _) in owner.source_port().branches().unwrap() {
        let port = owner.source_port();
        let descriptor = port.observe(branch).unwrap();
        let handle = port.readmit_descriptor(&descriptor).unwrap();
        let task = owner
            .mutation_port()
            .begin_advance(execution, &handle, &mut (), |tx| {
                assert_eq!(
                    tx.target(reference.node).read(&evaluate)?,
                    AspectVersion::from_updates([(Aspect::new(0), 71)])
                );
                Ok(())
            })
            .unwrap();
        drop(fixture::finish(task, &execution));
    }
    assert_eq!(
        calls.load(Ordering::Relaxed),
        before + 3,
        "another build starts every branch's derived state cold"
    );
    drop(owner);
    media::close(serving, medium, &execution);
}

#[test]
fn descriptors_refuse_foreign_owners_unknown_versions_and_old_live_incarnations() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let reference = fixture::seed(medium.as_ref(), &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let original = reference.heads.first_key_value().unwrap().1.clone();
    let old = owner.source_port().readmit_descriptor(&original).unwrap();
    drop(owner);
    media::close(serving, medium, &execution);
    let (serving, medium) = media::open(directory.path(), false, &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let port = owner.source_port();
    assert!(port.readmit_descriptor(&original).is_ok());
    assert!(matches!(
        port.capture_authored_images(&old, &execution),
        Err(SignalDurableSourceDenial::HomeIncarnation(
            worth_proof::HomeIncarnationDenial::StaleIncarnation
        ))
    ));
    let foreign = DurableSignalOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([9; 16]).unwrap(),
        SignalOwnerSlotIdentity::from_recorded(3),
    );
    let branch = DurableSignalBranchIdentity::from_recorded(foreign, original.branch().ordinal());
    let descriptor = SignalDurableBranchDescriptor::from_recorded(
        DurableSignalGeneration::from_recorded(
            DurableSignalCellIncarnation::from_recorded(branch, original.cell().ordinal()),
            original.generation().ordinal(),
        ),
        None,
        DurableSignalAdmissionIdentity::from_recorded(foreign, original.admission().ordinal()),
        SignalDefinitionTransitionReference::from_recorded(
            SignalDefinitionTransitionIdentity::from_recorded(
                foreign,
                original.definition().identity().ordinal(),
            ),
            original.definition().digest(),
        ),
    )
    .unwrap();
    assert!(matches!(
        port.readmit_descriptor(&descriptor),
        Err(SignalDurableSourceDenial::ForeignOwner)
    ));
    let mut bytes = original.encode_bytes(&execution).unwrap().bytes().to_vec();
    bytes[8..12].copy_from_slice(&999_u32.to_le_bytes());
    assert!(matches!(
        port.readmit_descriptor_bytes(&execution, &bytes),
        Err(SignalDurableSourceDenial::Frame(
            SignalDurableFrameDenial::UnsupportedVersion(_)
        ))
    ));
    drop((old, port, owner));
    media::close(serving, medium, &execution);
}
