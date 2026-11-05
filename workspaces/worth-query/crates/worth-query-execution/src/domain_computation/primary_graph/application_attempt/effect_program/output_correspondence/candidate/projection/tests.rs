//! Pure correspondence admission: these local IDs compare meaning, not authority.
use super::super::super::WorthQueryApplicationOutputProjectionDenial as Denial;
use super::*;
use worth_relational::facade::{identity::PartitionId, mvcc::RequiredOutputRolePosture as Native};
fn candidate() -> WorthQueryApplicationOutputCorrespondenceCandidate {
    let mut candidate = WorthQueryApplicationOutputCorrespondenceCandidate::default();
    candidate.roles.insert(
        "output".into(),
        CandidateOutputBinding {
            posture: WorthQueryApplicationOutputPosture::Preserve,
            entity_name: "Node".into(),
            entity_type: TypeId::of::<()>(),
            entity: EntityReference::Existing(EntityId::new(PartitionId::main(), 1, 1)),
        },
    );
    candidate
}
#[test]
fn equal_counts_do_not_admit_a_different_canonical_role_or_identity() {
    let mut candidate = candidate();
    let before = candidate.clone();
    assert_eq!(
        candidate.admit_primary_publication(
            [(
                "other",
                "Node",
                Native::Preserve,
                EntityId::new(PartitionId::main(), 1, 1)
            )]
            .into_iter()
        ),
        Err((Denial::MissingRole, "other".into()))
    );
    assert_eq!(candidate, before);
    assert_eq!(
        candidate.admit_primary_publication(
            [(
                "output",
                "Node",
                Native::Preserve,
                EntityId::new(PartitionId::main(), 2, 1)
            )]
            .into_iter()
        ),
        Err((Denial::EntityMismatch, "output".into()))
    );
    assert_eq!(candidate, before);
    candidate
        .admit_primary_publication(
            [(
                "output",
                "Node",
                Native::Preserve,
                EntityId::new(PartitionId::main(), 1, 1),
            )]
            .into_iter(),
        )
        .unwrap();
    assert!(candidate.closure_roles.is_empty());
}
#[test]
fn an_unmatched_live_role_refuses_before_any_correspondence_is_partitioned() {
    let mut candidate = candidate();
    let before = candidate.clone();
    assert_eq!(
        candidate.admit_primary_publication(std::iter::empty()),
        Err((Denial::ActionMismatch, "output".into()))
    );
    assert_eq!(candidate, before);
}
