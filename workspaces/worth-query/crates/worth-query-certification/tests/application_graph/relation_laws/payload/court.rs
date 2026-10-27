//! The authored handler publishes real relation payload; failed writers leave it unchanged.
use super::super::{installation::*, outcome::committed};
use super::*;
use crate::document_retention_model::operator_identity::request_scope;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome as Outcome,
    WorthQueryApplicationRequestMutationDenial as RequestDenial,
};
use worth_relational::facade::identity::RelationId;

#[test]
fn typed_relation_initialization_publishes_native_payload_and_refuses_other_programs_and_duplicates(
) {
    let world = install::<Payload>(3, &[]);
    committed(world.edit(1, &[(0, 1)]));
    let original = payloads(&world);
    assert_eq!(original.len(), 1);
    assert_eq!((original[0].0, original[0].1, original[0].3), (0, 1, 1));
    for (serial, expected) in [
        (
            2,
            WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
        ),
        (
            3,
            WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep,
        ),
        (
            4,
            WorthQueryApplicationAttemptDenialKind::DuplicateEffectKey,
        ),
    ] {
        let error = world
            .edit_in_scope(serial, &[(1, 2)], &request_scope())
            .unwrap_err();
        let RequestDenial::Handler(MutationHandlerExecutionDenial::Handler(cause)) = error else {
            panic!("actual typed writer denial required: {error:?}");
        };
        let cause = cause
            .downcast::<WorthQueryApplicationAttemptDenial>()
            .expect("original typed writer cause");
        assert_eq!(cause.kind(), expected);
        assert_eq!(payloads(&world), original);
    }
    assert!(matches!(
        world.edit(1, &[(0, 1)]),
        Outcome::AlreadyCommitted(_) | Outcome::PreviouslyCommitted(_)
    ));
    assert!(matches!(
        world.edit(1, &[(0, 2)]),
        Outcome::IdempotencyIntentDrift
    ));
    assert_eq!(
        payloads(&world),
        original,
        "command drift cannot initialize an existing relation"
    );
    committed(world.edit(5, &[(1, 2)]));
    let actual = payloads(&world);
    assert_eq!(actual.len(), 2);
    assert_eq!(actual[0], original[0]);
    assert_eq!((actual[1].0, actual[1].1, actual[1].3), (1, 2, 5));
    assert_ne!(
        actual[0].2, actual[1].2,
        "genuine physical relation occurrences"
    );
    world.close();
}

fn payloads(world: &World<Payload>) -> Vec<(u64, u64, RelationId, u64)> {
    let completed = world
        .application
        .runtime()
        .with_host_advancement(|phase| {
            world
                .projection
                .project(
                    &phase,
                    |reader| {
                        let mut values = Vec::new();
                        for source in 0..world.nodes {
                            let source_identity =
                                reader.resolve_entity(NodeKey::reference(), source).unwrap();
                            for edge in reader
                                .relations_from(Precedes::reference(), &source_identity)
                                .unwrap()
                            {
                                let target = reader.field(edge.to(), NodeKey::reference()).unwrap();
                                let weight = reader
                                    .relation_field(&edge, EdgeWeight::reference())
                                    .expect("actual native initialized payload");
                                values.push((source, target, edge.relation_id(), weight));
                            }
                        }
                        values.sort();
                        values
                    },
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let (values, snapshot, _) = completed.into_parts();
    drop(snapshot);
    values
}
