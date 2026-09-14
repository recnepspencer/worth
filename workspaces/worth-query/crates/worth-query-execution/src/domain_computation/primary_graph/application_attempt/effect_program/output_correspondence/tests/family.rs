use super::*;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPostureSet;

#[test]
fn role_family_accepts_variable_semantic_members_and_enforces_its_contract() {
    let program = Arc::new(());
    let mut candidate = WorthQueryApplicationOutputCorrespondenceCandidate::default();
    candidate.prepare_test_family::<Binding>(
        "face.",
        ApplicationMutationOutputPostureSet::CREATE,
        "entity",
        2,
    );
    let negative = created_handle("face-negative", 20, &program);
    let positive = created_handle("face-positive", 21, &program);
    let negative_role = WorthQueryApplicationOutputRole::<Binding, Entity, Create>::from_static(
        "face.negative.inherited.wall",
    );
    let positive_role = WorthQueryApplicationOutputRole::<Binding, Entity, Create>::from_static(
        "face.positive.cap",
    );
    candidate.bind(negative_role, &negative, &program).unwrap();
    assert_eq!(
        candidate.validate_effects(&[]).unwrap_err().kind(),
        WorthQueryApplicationAttemptDenialKind::MissingOutputRole
    );
    candidate.bind(positive_role, &positive, &program).unwrap();
    candidate
        .validate_effects(&[
            create_effect("face-negative", 20),
            create_effect("face-positive", 21),
        ])
        .unwrap();
    let undeclared =
        WorthQueryApplicationOutputRole::<Binding, Entity, Create>::from_static("edge.0");
    let extra = created_handle("edge", 22, &program);
    assert_eq!(
        candidate
            .bind(undeclared, &extra, &program)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationAttemptDenialKind::UndeclaredOutputRole
    );
    let empty_member = created_handle("empty-member", 24, &program);
    assert_eq!(
        candidate
            .bind(
                WorthQueryApplicationOutputRole::<Binding, Entity, Create>::from_static("face."),
                &empty_member,
                &program,
            )
            .unwrap_err()
            .kind(),
        WorthQueryApplicationAttemptDenialKind::UndeclaredOutputRole
    );
    let wrong_posture =
        WorthQueryApplicationOutputRole::<Binding, Entity, Preserve>::from_static("face.retained");
    let existing = existing_handle(EntityId::new(PartitionId::main(), 23, 0), &program);
    assert_eq!(
        candidate
            .bind(wrong_posture, &existing, &program)
            .unwrap_err()
            .kind(),
        WorthQueryApplicationAttemptDenialKind::OutputRoleActionMismatch
    );
    let committed = candidate.seal_with(|created| {
        Some(match created.client_key.as_raw_str() {
            Some("face-negative") => EntityId::new(PartitionId::main(), 30, 1),
            Some("face-positive") => EntityId::new(PartitionId::main(), 31, 1),
            key => panic!("unexpected created output: {key:?}"),
        })
    });
    let family = family_entries::<Entity>(&committed, "face.").unwrap();
    assert_eq!(
        family
            .iter()
            .map(|(role, posture, _)| (*role, *posture))
            .collect::<Vec<_>>(),
        vec![
            (
                "face.negative.inherited.wall",
                WorthQueryApplicationOutputPosture::Create
            ),
            (
                "face.positive.cap",
                WorthQueryApplicationOutputPosture::Create
            ),
        ]
    );
    assert_eq!(
        family_entries::<WrongEntity>(&committed, "face.").unwrap_err(),
        WorthQueryApplicationOutputProjectionDenial::EntityMismatch
    );
}

#[test]
fn family_range_is_deterministic_and_exposes_mixed_postures_only_within_prefix() {
    let program = Arc::new(());
    let preserve = WorthQueryApplicationOutputRole::<Binding, Entity, Preserve>::from_static(
        "family.preserve",
    );
    let create =
        WorthQueryApplicationOutputRole::<Binding, Entity, Create>::from_static("family.create");
    let retire =
        WorthQueryApplicationOutputRole::<Binding, Entity, Retire>::from_static("family.retire");
    let mut candidate = WorthQueryApplicationOutputCorrespondenceCandidate::default();
    candidate.prepare_test_family::<Binding>(
        "family.",
        ApplicationMutationOutputPostureSet::ALL,
        "entity",
        0,
    );
    candidate.prepare_test_role(
        WorthQueryApplicationOutputRole::<Binding, Entity, Preserve>::from_static("unrelated"),
        "entity",
    );
    candidate
        .bind(
            preserve,
            &existing_handle(EntityId::new(PartitionId::main(), 40, 1), &program),
            &program,
        )
        .unwrap();
    candidate
        .bind(
            create,
            &created_handle("family-create", 41, &program),
            &program,
        )
        .unwrap();
    candidate
        .bind(
            retire,
            &existing_handle(EntityId::new(PartitionId::main(), 42, 1), &program),
            &program,
        )
        .unwrap();
    candidate
        .bind(
            WorthQueryApplicationOutputRole::<Binding, Entity, Preserve>::from_static("unrelated"),
            &existing_handle(EntityId::new(PartitionId::main(), 43, 1), &program),
            &program,
        )
        .unwrap();
    let committed = candidate.seal_with(|_| Some(EntityId::new(PartitionId::main(), 41, 1)));

    let entries = family_entries::<Entity>(&committed, "family.").unwrap();
    assert_eq!(
        entries
            .iter()
            .map(|(role, posture, _)| (*role, *posture))
            .collect::<Vec<_>>(),
        vec![
            ("family.create", WorthQueryApplicationOutputPosture::Create),
            (
                "family.preserve",
                WorthQueryApplicationOutputPosture::Preserve,
            ),
            ("family.retire", WorthQueryApplicationOutputPosture::Retire),
        ]
    );
    assert_eq!(
        entries
            .iter()
            .filter(|(_, posture, _)| *posture != WorthQueryApplicationOutputPosture::Retire)
            .count(),
        2
    );
}

fn family_entries<'family, EntityType: 'static>(
    correspondence: &'family WorthQueryApplicationOutputCorrespondence,
    prefix: &'family str,
) -> Result<
    Vec<(&'family str, WorthQueryApplicationOutputPosture, EntityId)>,
    WorthQueryApplicationOutputProjectionDenial,
> {
    correspondence
        .binding_family_entries::<Binding, EntityType>(prefix)?
        .collect()
}
