use super::*;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPostureSet;

const FACE: WorthQueryApplicationOutputRoleFamily<Binding, Entity> =
    WorthQueryApplicationOutputRoleFamily::for_entity::<Schema>(
        "face.",
        ApplicationMutationOutputPostureSet::CREATE,
        2,
    );
const FAMILY: WorthQueryApplicationOutputRoleFamily<Binding, Entity> =
    WorthQueryApplicationOutputRoleFamily::for_entity::<Schema>(
        "family.",
        ApplicationMutationOutputPostureSet::ALL,
        0,
    );
const UNRELATED: WorthQueryApplicationOutputRole<Binding, Entity, WorthQueryPreserveOutput> =
    WorthQueryApplicationOutputRole::for_entity::<Schema>("unrelated");

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
    let negative_role = FACE
        .member::<WorthQueryCreateOutput>("negative.inherited.wall")
        .unwrap();
    let positive_role = FACE
        .member::<WorthQueryCreateOutput>("positive.cap")
        .unwrap();
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
    // Stray declarations: a fixed token outside the binding's contract, and
    // one named exactly the family prefix, which no member can be.
    let undeclared =
        WorthQueryApplicationOutputRole::<Binding, Entity, WorthQueryCreateOutput>::for_entity::<
            Schema,
        >("edge.0");
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
                WorthQueryApplicationOutputRole::<Binding, Entity, WorthQueryCreateOutput>::for_entity::<
                    Schema,
                >("face."),
                &empty_member,
                &program,
            )
            .unwrap_err()
            .kind(),
        WorthQueryApplicationAttemptDenialKind::UndeclaredOutputRole
    );
    let wrong_posture = FACE.member::<WorthQueryPreserveOutput>("retained").unwrap();
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
    let family = committed.family_entries(FACE).unwrap();
    assert_eq!(
        family
            .iter()
            .map(|entry| (entry.role(), entry.posture()))
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
        committed
            .family_entries(
                WorthQueryApplicationOutputRoleFamily::<Binding, WrongEntity>::for_entity::<Schema>(
                    "face.",
                    ApplicationMutationOutputPostureSet::CREATE,
                    2,
                )
            )
            .unwrap_err(),
        WorthQueryApplicationOutputProjectionDenial::EntityMismatch
    );
}

#[test]
fn family_range_is_deterministic_and_exposes_mixed_postures_only_within_prefix() {
    let program = Arc::new(());
    let preserve = FAMILY
        .member::<WorthQueryPreserveOutput>("preserve")
        .unwrap();
    let create = FAMILY.member::<WorthQueryCreateOutput>("create").unwrap();
    let retire = FAMILY.member::<WorthQueryRetireOutput>("retire").unwrap();
    let mut candidate = WorthQueryApplicationOutputCorrespondenceCandidate::default();
    candidate.prepare_test_family::<Binding>(
        "family.",
        ApplicationMutationOutputPostureSet::ALL,
        "entity",
        0,
    );
    candidate.prepare_test_role(&UNRELATED, "entity");
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
            UNRELATED,
            &existing_handle(EntityId::new(PartitionId::main(), 43, 1), &program),
            &program,
        )
        .unwrap();
    let committed = candidate.seal_with(|_| Some(EntityId::new(PartitionId::main(), 41, 1)));

    let entries = committed.family_entries(FAMILY).unwrap();
    assert_eq!(
        entries
            .iter()
            .map(|entry| (entry.role(), entry.posture()))
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
            .filter(|entry| entry.posture() != WorthQueryApplicationOutputPosture::Retire)
            .count(),
        2
    );
}
