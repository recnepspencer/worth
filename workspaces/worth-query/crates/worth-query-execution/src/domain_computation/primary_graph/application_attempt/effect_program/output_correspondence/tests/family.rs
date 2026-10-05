use super::*;
use worth_query_declaration::facade::application_operation::{
    WorthQueryCreateOutput, WorthQueryPreserveOutput, WorthQueryRetireOutput,
};

#[test]
fn role_family_accepts_variable_semantic_members_and_enforces_its_contract() {
    let program = Arc::new(());
    let mut candidate = prepared::<FaceOutputs>();
    let negative = created_handle("face-negative", 20, &program);
    let positive = created_handle("face-positive", 21, &program);
    let negative_role =
        OutputRoleUse::member::<Face, WorthQueryCreateOutput>("negative.inherited.wall").unwrap();
    let positive_role =
        OutputRoleUse::member::<Face, WorthQueryCreateOutput>("positive.cap").unwrap();
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
    // Erased uses no marker can write: a fixed role outside the contract, one
    // named exactly the family prefix, which no member can be, and a member
    // with a posture the family does not admit.
    let create = |name: &str| {
        stray::<FaceOutputs, Entity>(
            name,
            WorthQueryApplicationOutputPosture::Create,
            Cardinality::ExactlyOne,
        )
    };
    for (role, key, kind) in [
        (create("edge.0"), "edge", 22),
        (create("face."), "empty-member", 24),
    ] {
        assert_eq!(
            candidate
                .bind(role, &created_handle(key, kind, &program), &program)
                .unwrap_err()
                .kind(),
            WorthQueryApplicationAttemptDenialKind::UndeclaredOutputRole
        );
    }
    let wrong_posture = stray::<FaceOutputs, Entity>(
        "face.retained",
        WorthQueryApplicationOutputPosture::Preserve,
        Cardinality::ExactlyOne,
    );
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
    let outputs = committed.outputs_of::<FaceOutputs>().unwrap();
    let family = outputs.family_entries::<Face>().unwrap();
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
        outputs
            .member::<Face, WorthQueryCreateOutput>("positive.cap")
            .unwrap()
            .entity_id(),
        EntityId::new(PartitionId::main(), 31, 1)
    );
    assert!(matches!(
        outputs.member::<Face, WorthQueryCreateOutput>(""),
        Err(WorthQueryApplicationOutputProjectionDenial::InvalidMemberSuffix(_))
    ));
    // Another contract's family cannot be named on this view; its erased
    // read is still refused.
    assert!(matches!(
        committed.family_members::<Mixed>(),
        Err(WorthQueryApplicationOutputProjectionDenial::ForeignContract)
    ));
}

#[test]
fn family_range_is_deterministic_and_exposes_mixed_postures_only_within_prefix() {
    let program = Arc::new(());
    let preserve = OutputRoleUse::member::<Mixed, WorthQueryPreserveOutput>("preserve").unwrap();
    let create = OutputRoleUse::member::<Mixed, WorthQueryCreateOutput>("create").unwrap();
    let retire = OutputRoleUse::member::<Mixed, WorthQueryRetireOutput>("retire").unwrap();
    let mut candidate = prepared::<MixedOutputs>();
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
            OutputRoleUse::fixed::<Unrelated>(),
            &existing_handle(EntityId::new(PartitionId::main(), 43, 1), &program),
            &program,
        )
        .unwrap();
    let committed = candidate.seal_with(|_| Some(EntityId::new(PartitionId::main(), 41, 1)));

    let outputs = committed.outputs_of::<MixedOutputs>().unwrap();
    let entries = outputs.family_entries::<Mixed>().unwrap();
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
