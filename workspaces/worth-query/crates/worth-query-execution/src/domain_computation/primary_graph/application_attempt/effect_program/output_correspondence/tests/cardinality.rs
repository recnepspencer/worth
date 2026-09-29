//! A role token's cardinality decides the shape of every read, and a token
//! whose cardinality differs from the declaration is refused on both sides.

use super::*;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPostureSet;

const COMPANION: WorthQueryApplicationOptionalOutputRole<Binding, Entity, Preserve> =
    WorthQueryApplicationOptionalOutputRole::from_static("companion");
const COMPANION_AS_REQUIRED: WorthQueryApplicationOutputRole<Binding, Entity, Preserve> =
    WorthQueryApplicationOutputRole::from_static("companion");
const PRESERVED_AS_OPTIONAL: WorthQueryApplicationOptionalOutputRole<Binding, Entity, Preserve> =
    WorthQueryApplicationOptionalOutputRole::from_static("preserved");

fn optional_candidate() -> WorthQueryApplicationOutputCorrespondenceCandidate {
    let mut candidate = WorthQueryApplicationOutputCorrespondenceCandidate::default();
    candidate.prepare_test_role(&PRESERVED, "entity");
    candidate.prepare_test_role(&COMPANION, "entity");
    candidate
}

#[test]
fn an_unbound_at_most_one_role_reads_as_none_and_a_bound_one_as_some() {
    let program = Arc::new(());
    let subject = EntityId::new(PartitionId::main(), 50, 1);
    let companion = EntityId::new(PartitionId::main(), 51, 1);

    let mut absent = optional_candidate();
    absent
        .bind(PRESERVED, &existing_handle(subject, &program), &program)
        .unwrap();
    let absent = absent.seal_with(|_| None);
    assert!(absent.entity(COMPANION).unwrap().is_none());
    assert_eq!(absent.entity(PRESERVED).unwrap().entity_id(), subject);

    let mut present = optional_candidate();
    present
        .bind(PRESERVED, &existing_handle(subject, &program), &program)
        .unwrap();
    present
        .bind(COMPANION, &existing_handle(companion, &program), &program)
        .unwrap();
    let present = present.seal_with(|_| None);
    assert_eq!(
        present
            .entity(COMPANION)
            .unwrap()
            .map(|output| output.entity_id()),
        Some(companion)
    );
}

#[test]
fn a_read_token_whose_cardinality_differs_from_the_declaration_is_refused() {
    let program = Arc::new(());
    let mut candidate = optional_candidate();
    candidate
        .bind(
            PRESERVED,
            &existing_handle(EntityId::new(PartitionId::main(), 52, 1), &program),
            &program,
        )
        .unwrap();
    candidate
        .bind(
            COMPANION,
            &existing_handle(EntityId::new(PartitionId::main(), 53, 1), &program),
            &program,
        )
        .unwrap();
    let committed = candidate.seal_with(|_| None);

    assert_eq!(
        committed.entity(COMPANION_AS_REQUIRED).err(),
        Some(WorthQueryApplicationOutputProjectionDenial::CardinalityMismatch)
    );
    assert_eq!(
        committed.entity(PRESERVED_AS_OPTIONAL).err(),
        Some(WorthQueryApplicationOutputProjectionDenial::CardinalityMismatch)
    );
}

#[test]
fn a_write_token_whose_cardinality_differs_from_the_declaration_is_refused() {
    let program = Arc::new(());
    let existing = existing_handle(EntityId::new(PartitionId::main(), 54, 1), &program);
    let mut candidate = optional_candidate();
    candidate.prepare_test_family::<Binding>(
        "member.",
        ApplicationMutationOutputPostureSet::PRESERVE,
        "entity",
        0,
    );
    let optional_member =
        WorthQueryApplicationOptionalOutputRole::<Binding, Entity, Preserve>::from_static(
            "member.one",
        );

    for denial in [
        candidate.bind(COMPANION_AS_REQUIRED, &existing, &program),
        candidate.bind(PRESERVED_AS_OPTIONAL, &existing, &program),
        candidate.bind(optional_member, &existing, &program),
    ] {
        assert_eq!(
            denial.unwrap_err().kind(),
            WorthQueryApplicationAttemptDenialKind::OutputRoleCardinalityMismatch
        );
    }
}

#[test]
fn checkpoint_readmission_keeps_absence_a_value() {
    let entity = EntityId::new(PartitionId::main(), 55, 1);
    let readmitted = WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
        TypeId::of::<Binding>(),
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

    assert!(readmitted.entity(COMPANION).unwrap().is_none());
    assert_eq!(readmitted.entity(PRESERVED).unwrap().entity_id(), entity);
    assert_eq!(
        readmitted.entity(COMPANION_AS_REQUIRED).err(),
        Some(WorthQueryApplicationOutputProjectionDenial::CardinalityMismatch)
    );
}
