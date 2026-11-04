//! A role's cardinality decides the shape of every read. A use whose
//! cardinality differs from the declaration cannot be written with a marker;
//! its erased form, as a readmitted or portable role, is refused on both
//! sides.

use crate::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::tests::*;

/// The companion role read as exactly-one.
pub fn companion_as_required() -> OutputRoleUse {
    stray::<OptionalOutputs, Entity>(
        "companion",
        WorthQueryApplicationOutputPosture::Preserve,
        Cardinality::ExactlyOne,
    )
}

/// The exactly-one subject role read as at-most-one.
pub fn subject_as_optional() -> OutputRoleUse {
    stray::<OptionalOutputs, Entity>(
        "preserved",
        WorthQueryApplicationOutputPosture::Preserve,
        Cardinality::AtMostOne,
    )
}

#[test]
fn an_unbound_at_most_one_role_reads_as_none_and_a_bound_one_as_some() {
    let program = Arc::new(());
    let subject = EntityId::new(PartitionId::main(), 50, 1);
    let companion = EntityId::new(PartitionId::main(), 51, 1);

    let mut absent = prepared::<OptionalOutputs>();
    absent
        .bind_for_test(
            OutputRoleUse::fixed_for_test::<Subject>(),
            &existing_handle(subject, &program),
            &program,
        )
        .unwrap();
    let absent = absent.seal_with_for_test(|_| None);
    let absent = absent.outputs_of::<OptionalOutputs>().unwrap();
    assert!(absent.entity::<Companion>().unwrap().is_none());
    assert_eq!(absent.entity::<Subject>().unwrap().entity_id(), subject);

    let mut present = prepared::<OptionalOutputs>();
    present
        .bind_for_test(
            OutputRoleUse::fixed_for_test::<Subject>(),
            &existing_handle(subject, &program),
            &program,
        )
        .unwrap();
    present
        .bind_for_test(
            OutputRoleUse::fixed_for_test::<Companion>(),
            &existing_handle(companion, &program),
            &program,
        )
        .unwrap();
    let present = present.seal_with_for_test(|_| None);
    let present = present.outputs_of::<OptionalOutputs>().unwrap();
    assert_eq!(
        present
            .entity::<Companion>()
            .unwrap()
            .map(|output| output.entity_id()),
        Some(companion)
    );
}

#[test]
fn a_read_whose_cardinality_differs_from_the_declaration_is_refused() {
    let program = Arc::new(());
    let mut candidate = prepared::<OptionalOutputs>();
    candidate
        .bind_for_test(
            OutputRoleUse::fixed_for_test::<Subject>(),
            &existing_handle(EntityId::new(PartitionId::main(), 52, 1), &program),
            &program,
        )
        .unwrap();
    candidate
        .bind_for_test(
            OutputRoleUse::fixed_for_test::<Companion>(),
            &existing_handle(EntityId::new(PartitionId::main(), 53, 1), &program),
            &program,
        )
        .unwrap();
    let committed = candidate.seal_with_for_test(|_| None);

    for role in [companion_as_required(), subject_as_optional()] {
        assert_eq!(
            committed.bound_entity(&role).err(),
            Some(WorthQueryApplicationOutputProjectionDenial::CardinalityMismatch)
        );
    }
}

#[test]
fn a_write_whose_cardinality_differs_from_the_declaration_is_refused() {
    let program = Arc::new(());
    let existing = existing_handle(EntityId::new(PartitionId::main(), 54, 1), &program);
    let mut candidate = prepared::<OptionalOutputs>();
    let optional_member = stray::<OptionalOutputs, Entity>(
        "member.one",
        WorthQueryApplicationOutputPosture::Preserve,
        Cardinality::AtMostOne,
    );

    for role in [
        companion_as_required(),
        subject_as_optional(),
        optional_member,
    ] {
        assert_eq!(
            candidate
                .bind_for_test(role, &existing, &program)
                .unwrap_err()
                .kind(),
            WorthQueryApplicationAttemptDenialKind::OutputRoleCardinalityMismatch
        );
    }
}

#[test]
fn checkpoint_readmission_keeps_absence_a_value() {
    let host = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let entity = EntityId::new(PartitionId::main(), 55, 1);
    let readmitted = WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
        TypeId::of_for_test::<Binding>(),
        TypeId::of_for_test::<OptionalOutputs>(),
        ["companion"].into_iter(),
        &[WorthQueryCheckpointOutputRole {
            role: "preserved".into(),
            posture: WorthQueryApplicationOutputPosture::Preserve,
            entity_name: "entity".into(),
            entity,
        }],
        |_| Some(TypeId::of_for_test::<Entity>()),
        &execution,
    )
    .unwrap();

    let outputs = readmitted.outputs_of_for_test::<OptionalOutputs>().unwrap();
    assert!(outputs.entity::<Companion>().unwrap().is_none());
    assert_eq!(outputs.entity::<Subject>().unwrap().entity_id(), entity);
    assert_eq!(
        readmitted.bound_entity_for_test(&companion_as_required()).err(),
        Some(WorthQueryApplicationOutputProjectionDenial::CardinalityMismatch)
    );
    // Reading it as another contract is refused once, at the typed view;
    // no read of that contract's roles is reached.
    assert_eq!(
        readmitted.outputs_of_for_test::<Outputs>().err(),
        Some(WorthQueryApplicationOutputProjectionDenial::ForeignContract)
    );
}
