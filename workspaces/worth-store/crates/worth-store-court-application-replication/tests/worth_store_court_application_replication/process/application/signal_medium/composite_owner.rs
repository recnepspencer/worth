//! The real Store adapter observes only the installed composing issuer's original.
use super::{medium_fixture as media, owner_fixture as fixture};
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

fn association(ordinal: u64) -> CompositeAttemptAssociation {
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
            .begin_composite_advance(execution, binding, grant, &mut (), |tx| {
                calls.set(calls.get() + 1);
                if change_definition {
                    recipe
                        .keyed("composite-authored-node")
                        .node_in_transaction(tx);
                }
                Ok(())
            })
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
        let Err(SignalDurableAdvanceDenial::CompositeAdmission(actual)) = composite
            .mutation_port()
            .begin_composite_advance(execution, binding, grant, &mut (), |_| {
                panic!("a foreign or changed binding cannot execute its body")
            })
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
