//! A recorded selector compares complete identity with actual Native custody.
use super::*;
use worth_relational::facade::{identity::*, mvcc::*};
fn run(execution: &ExecutionRequest<'_, '_>) {
    let world = World::new(execution);
    let selected = world.birth(execution);
    let foreign = world.add_unrelated_publication(execution);
    let own = selected.members().next().unwrap().3;
    let other = foreign.members().next().unwrap().3;
    let before = world.basis();
    let role = RecordedRequiredOutputRole {
        role: "output",
        entity_name: "node",
        kind: KindId(1),
        posture: RequiredOutputRolePosture::Create,
        entity: Some(own),
    };
    let matches = |role| {
        selected
            .matches_recorded_description(
                execution,
                "source-owned",
                &[1; 32],
                selected.meaning(),
                std::iter::once(role),
            )
            .unwrap()
    };
    assert!(matches(role));
    assert!(!matches(RecordedRequiredOutputRole {
        entity: None,
        ..role
    }));
    assert!(
        !matches(RecordedRequiredOutputRole {
            entity: Some(other),
            ..role
        }),
        "a live same-kind record owned by another publication is not this member"
    );
    assert!(
        !matches(RecordedRequiredOutputRole {
            entity: Some(EntityId::new(
                own.partition_id,
                own.local_slot_value(),
                own.generation.as_u32() + 1
            )),
            ..role
        }),
        "another generation of the selected slot is not this member"
    );
    assert!(!matches(RecordedRequiredOutputRole {
        role: "foreign-role",
        ..role
    }));
    assert!(!matches(RecordedRequiredOutputRole {
        entity_name: "foreign-entity",
        ..role
    }));
    assert!(!matches(RecordedRequiredOutputRole {
        posture: RequiredOutputRolePosture::Preserve,
        ..role
    }));
    assert_eq!(world.basis().descriptor(), before.descriptor());
}
#[test]
fn a_recorded_selector_requires_the_genuine_publications_complete_identity() {
    let host = request();
    run(&ExecutionRequest::serial(&host));
}
#[test]
fn worker_only_a_recorded_selector_requires_the_genuine_publications_complete_identity() {
    use worth_execution::{ExecutionResourceLease, LeaseRequest};
    use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::new(1).unwrap(),
        ),
        deadline: None,
        cancellation: worth_execution::CancellationToken::new(),
    })
    .unwrap();
    let execution = ExecutionRequest::leased(&lease);
    assert!(!execution.has_accounting());
    run(&execution);
}
