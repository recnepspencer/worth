//! The real Store adapter observes only the installed composing issuer's original.
use super::{medium_fixture as media, owner_fixture as fixture};
mod home_retirement;
mod publication_membership;
use std::cell::Cell;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    CompositeAttemptAssociation, DurableCompositePublicationAttemptIdentity,
    DurableProductBranchIdentity, DurableProductBranchIncarnation, DurableWorldOwnerIdentity,
    WorldOwnerSlotIdentity,
};
use worth_proof::{
    CompositeSignalPublicationAdmissionDenial, InstalledCompositeSignalPublicationOwner,
};
use worth_signal::facade::durability::*;

pub(super) fn association(ordinal: u64) -> CompositeAttemptAssociation {
    let world = DurableWorldOwnerIdentity::from_recorded(
        media::owner().home(),
        WorldOwnerSlotIdentity::from_recorded(4),
    );
    CompositeAttemptAssociation::from_recorded(
        DurableCompositePublicationAttemptIdentity::from_recorded(world, ordinal),
        DurableProductBranchIdentity::from_recorded(world, "product".into()).unwrap(),
        DurableProductBranchIncarnation::from_recorded(world, 1),
    )
    .unwrap()
}

#[test]
fn composite_grant_publishes_original_association_for_retained_and_changed_definitions() {
    for change_definition in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let funding = media::serial_request();
        let execution = ExecutionRequest::serial(&funding);
        let (serving, medium) = media::open(directory.path(), true, &execution);
        let mut reference = fixture::seed(medium.as_ref(), &execution);
        let recipe = fixture::recipe(&mut reference.memory);
        let owner = fixture::readmit(medium.clone(), &execution);
        let descriptor = reference.heads.values().next().unwrap();
        let before = owner.source_port().readmit_descriptor(descriptor).unwrap();
        let original_writer = owner.mutation_port();
        let issuer = InstalledCompositeSignalPublicationOwner::fresh();
        let composite = owner
            .bind_composite_owner(issuer.authority(), &execution)
            .unwrap();
        assert!(matches!(
            original_writer.begin_advance(execution, &before, &mut (), |_| {
                panic!("attached standalone writer must refuse before its body")
            }),
            Err(SignalDurableAdvanceDenial::CompositePublicationRequired)
        ));
        let original = association(19);
        let binding = composite
            .mutation_port()
            .prepare_composite_binding(&execution, before.basis(), original.clone())
            .unwrap();
        let grant = issuer.admit_original(binding.commitment());
        let calls = Cell::new(0);
        let task = composite
            .mutation_port()
            .begin_composite_advance(
                execution,
                binding,
                grant,
                &mut (),
                &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
                |tx| {
                    calls.set(calls.get() + 1);
                    if change_definition {
                        recipe
                            .keyed("composite-authored-node")
                            .node_in_transaction(tx);
                    }
                    Ok(())
                },
            )
            .unwrap();
        let output = fixture::finish(task, &execution);
        assert_eq!(calls.get(), 1);
        assert_eq!(output.artifact().association(), Some(&original));
        assert_eq!(
            output.branch().descriptor().definition() != descriptor.definition(),
            change_definition
        );
        let persisted = owner
            .read_original_advance(output.artifact().identity(), &execution)
            .unwrap();
        assert_eq!(persisted.digest(), output.artifact().digest());
        assert_eq!(persisted.association(), Some(&original));
        assert!(matches!(
            original_writer.resume_original(persisted.identity()),
            Err(SignalDurableAdvanceDenial::CompositePublicationRequired)
        ));
        drop((persisted, output, before, composite, original_writer, owner));
        media::close(serving, medium, &execution);
    }
}

#[test]
fn foreign_issuer_and_changed_commitment_refuse_before_the_signal_body() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let reference = fixture::seed(medium.as_ref(), &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let (&branch, descriptor) = reference.heads.first_key_value().unwrap();
    let before = owner.source_port().readmit_descriptor(descriptor).unwrap();
    let issuer = InstalledCompositeSignalPublicationOwner::fresh();
    let foreign = InstalledCompositeSignalPublicationOwner::fresh();
    let composite = owner
        .bind_composite_owner(issuer.authority(), &execution)
        .unwrap();
    for foreign_owner in [true, false] {
        let binding = composite
            .mutation_port()
            .prepare_composite_binding(&execution, before.basis(), association(23))
            .unwrap();
        let grant = if foreign_owner {
            foreign.admit_original(binding.commitment())
        } else {
            issuer.admit_original([0; 32])
        };
        let expected = if foreign_owner {
            CompositeSignalPublicationAdmissionDenial::ForeignOwner
        } else {
            CompositeSignalPublicationAdmissionDenial::BindingMismatch
        };
        let Err(SignalDurableAdvanceDenial::CompositeAdmission(actual)) =
            composite.mutation_port().begin_composite_advance(
                execution,
                binding,
                grant,
                &mut (),
                &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
                |_| panic!("a foreign or changed binding cannot execute its body"),
            )
        else {
            panic!("the actual installed affinity and binding must refuse");
        };
        assert_eq!(actual, expected);
        assert_eq!(&owner.source_port().observe(branch).unwrap(), descriptor);
    }
    assert!(matches!(
        owner.bind_composite_owner(foreign.authority(), &execution),
        Err(SignalDurableAdvanceDenial::CompositeOwnerAlreadyAttached)
    ));
    drop((before, composite, owner));
    media::close(serving, medium, &execution);
}

#[test]
fn composite_attachment_names_every_pending_original_and_preserves_standalone_completion() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let reference = fixture::seed(medium.as_ref(), &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let mut tasks = Vec::new();
    for descriptor in reference.heads.values().take(2) {
        let basis = owner.source_port().readmit_descriptor(descriptor).unwrap();
        tasks.push(
            owner
                .mutation_port()
                .begin_advance(execution, &basis, &mut (), |_| Ok(()))
                .unwrap(),
        );
    }
    assert_eq!(tasks.len(), 2);
    let mut expected = tasks.iter().map(|task| task.original()).collect::<Vec<_>>();
    expected.sort();
    let issuer = InstalledCompositeSignalPublicationOwner::fresh();
    let Err(SignalDurableAdvanceDenial::CompositeAttachmentPending { originals }) =
        owner.bind_composite_owner(issuer.authority(), &execution)
    else {
        panic!("every original must remain owned before attachment");
    };
    assert_eq!(originals.elements(), expected.as_slice());
    drop(originals);
    for task in tasks {
        drop(fixture::finish(task, &execution));
    }
    drop(
        owner
            .bind_composite_owner(issuer.authority(), &execution)
            .unwrap(),
    );
    drop(owner);
    media::close(serving, medium, &execution);
}

struct ResumptionPause(worth_store::physical_runtime::production::PhysicalMutationPauseGate);
impl Drop for ResumptionPause {
    fn drop(&mut self) {
        self.0.release();
    }
}

#[test]
fn disposed_composite_handles_resume_the_exact_original_before_drive_and_during_physical_pending() {
    use worth_proof::CompositeSignalPublicationAdmissionDenial as Admission;
    for after_pending in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let funding = media::serial_request();
        let execution = ExecutionRequest::serial(&funding);
        let (serving, medium) = media::open(directory.path(), true, &execution);
        let reference = fixture::seed(medium.as_ref(), &execution);
        let owner = fixture::readmit(medium.clone(), &execution);
        let issuer = InstalledCompositeSignalPublicationOwner::fresh();
        let composite = owner
            .bind_composite_owner(issuer.authority(), &execution)
            .unwrap();
        let writer = composite.mutation_port();
        let before = owner
            .source_port()
            .readmit_descriptor(reference.heads.values().next().unwrap())
            .unwrap();
        let binding = writer
            .prepare_composite_binding(&execution, before.basis(), association(31))
            .unwrap();
        let commitment = binding.commitment();
        let calls = Cell::new(0);
        let task = writer
            .begin_composite_advance(
                execution,
                binding,
                issuer.admit_original(commitment),
                &mut (),
                &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
                |_| {
                    calls.set(calls.get() + 1);
                    Ok(())
                },
            )
            .unwrap();
        let original = task.original();
        let pause = after_pending.then(|| {
            ResumptionPause(serving.pause_physical_mutation_at(
            worth_store::physical_runtime::production::PhysicalMutationCheckpoint::BeforeWalAppend,
        ))
        });
        let task = if let Some(pause) = &pause {
            let SignalDurableAdvanceProgress::Pending(task) = task
                .drive(&execution)
                .unwrap_or_else(|cause| panic!("{:?}", cause.denial()))
            else {
                panic!("the actual physical gate precedes terminal publication");
            };
            assert!(pause.0.await_arrival());
            task
        } else {
            task
        };
        drop(task);
        assert!(matches!(
            writer.resume_original(original),
            Err(SignalDurableAdvanceDenial::CompositePublicationRequired)
        ));
        let foreign = InstalledCompositeSignalPublicationOwner::fresh();
        assert!(matches!(
            writer.resume_composite_original(original, foreign.admit_original(commitment)),
            Err(SignalDurableAdvanceDenial::CompositeAdmission(
                Admission::ForeignOwner
            ))
        ));
        let changed = writer
            .prepare_composite_binding(&execution, before.basis(), association(32))
            .unwrap();
        assert!(matches!(
            writer.resume_composite_original(original, issuer.admit_original(changed.commitment())),
            Err(SignalDurableAdvanceDenial::CompositeAdmission(
                Admission::BindingMismatch
            ))
        ));
        let other_before = owner
            .source_port()
            .readmit_descriptor(reference.heads.values().nth(1).unwrap())
            .unwrap();
        let other_binding = writer
            .prepare_composite_binding(&execution, other_before.basis(), association(33))
            .unwrap();
        let other_grant = issuer.admit_original(other_binding.commitment());
        let other = writer
            .begin_composite_advance(
                execution,
                other_binding,
                other_grant,
                &mut (),
                &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
                |_| Ok(()),
            )
            .unwrap();
        let other = if after_pending {
            let SignalDurableAdvanceProgress::Pending(task) = other
                .drive(&execution)
                .unwrap_or_else(|cause| panic!("{:?}", cause.denial()))
            else {
                panic!("the first actual publication retains the physical turn");
            };
            task
        } else {
            other
        };
        let resumed = writer
            .resume_composite_original(original, issuer.admit_original(commitment))
            .unwrap();
        assert_eq!(resumed.original(), original);
        drop(pause);
        let output = fixture::finish(resumed, &execution);
        assert_eq!(calls.get(), 1);
        assert_eq!(output.artifact().identity(), original);
        assert_eq!(output.artifact().association(), Some(&association(31)));
        assert_eq!(
            output.branch().descriptor().generation().ordinal(),
            before.descriptor().generation().ordinal() + 1
        );
        assert_eq!(
            owner
                .read_original_advance(original, &execution)
                .unwrap()
                .digest(),
            output.artifact().digest()
        );
        assert!(
            matches!(writer.resume_composite_original(original, issuer.admit_original(commitment)),
            Err(SignalDurableAdvanceDenial::UnknownTask { original: missing }) if missing == original)
        );
        // Finishing a different branch proves the original physical turn released.
        drop(fixture::finish(other, &execution));
        // A new original on the same branch proves its branch reservation released.
        let next_binding = writer
            .prepare_composite_binding(&execution, output.branch().basis(), association(34))
            .unwrap();
        let next_grant = issuer.admit_original(next_binding.commitment());
        drop(fixture::finish(
            writer
                .begin_composite_advance(
                    execution,
                    next_binding,
                    next_grant,
                    &mut (),
                    &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
                    |_| Ok(()),
                )
                .unwrap(),
            &execution,
        ));
        drop((output, before, other_before, writer, composite, owner));
        media::close(serving, medium, &execution);
    }
}
