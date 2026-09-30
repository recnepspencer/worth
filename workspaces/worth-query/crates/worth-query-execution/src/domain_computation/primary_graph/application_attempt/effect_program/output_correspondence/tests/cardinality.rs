//! A role's cardinality decides the shape of every read. A use whose
//! cardinality differs from the declaration cannot be written with a marker;
//! its erased form, as a readmitted or portable role, is refused on both
//! sides.

use super::*;

/// The companion role read as exactly-one.
fn companion_as_required() -> OutputRoleUse {
    stray::<OptionalOutputs, Entity>(
        "companion",
        WorthQueryApplicationOutputPosture::Preserve,
        Cardinality::ExactlyOne,
    )
}

/// The exactly-one subject role read as at-most-one.
fn subject_as_optional() -> OutputRoleUse {
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
        .bind(
            OutputRoleUse::fixed::<Subject>(),
            &existing_handle(subject, &program),
            &program,
        )
        .unwrap();
    let absent = absent.seal_with(|_| None);
    assert!(absent.entity::<Companion>().unwrap().is_none());
    assert_eq!(absent.entity::<Subject>().unwrap().entity_id(), subject);

    let mut present = prepared::<OptionalOutputs>();
    present
        .bind(
            OutputRoleUse::fixed::<Subject>(),
            &existing_handle(subject, &program),
            &program,
        )
        .unwrap();
    present
        .bind(
            OutputRoleUse::fixed::<Companion>(),
            &existing_handle(companion, &program),
            &program,
        )
        .unwrap();
    let present = present.seal_with(|_| None);
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
        .bind(
            OutputRoleUse::fixed::<Subject>(),
            &existing_handle(EntityId::new(PartitionId::main(), 52, 1), &program),
            &program,
        )
        .unwrap();
    candidate
        .bind(
            OutputRoleUse::fixed::<Companion>(),
            &existing_handle(EntityId::new(PartitionId::main(), 53, 1), &program),
            &program,
        )
        .unwrap();
    let committed = candidate.seal_with(|_| None);

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
                .bind(role, &existing, &program)
                .unwrap_err()
                .kind(),
            WorthQueryApplicationAttemptDenialKind::OutputRoleCardinalityMismatch
        );
    }
}

#[test]
fn checkpoint_readmission_keeps_absence_a_value() {
    let entity = EntityId::new(PartitionId::main(), 55, 1);
    let readmitted = WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
        TypeId::of::<Binding>(),
        TypeId::of::<OptionalOutputs>(),
        BTreeSet::from(["companion".to_owned()]),
        vec![WorthQueryCheckpointOutputRole {
            role: "preserved".into(),
            posture: WorthQueryApplicationOutputPosture::Preserve,
            entity_name: "entity".into(),
            entity,
        }],
        |_| Some(TypeId::of::<Entity>()),
    )
    .unwrap();

    assert!(readmitted.entity::<Companion>().unwrap().is_none());
    assert_eq!(readmitted.entity::<Subject>().unwrap().entity_id(), entity);
    assert_eq!(
        readmitted.bound_entity(&companion_as_required()).err(),
        Some(WorthQueryApplicationOutputProjectionDenial::CardinalityMismatch)
    );
    assert_eq!(
        readmitted.entity::<Preserved>().err(),
        Some(WorthQueryApplicationOutputProjectionDenial::ForeignContract)
    );
}
