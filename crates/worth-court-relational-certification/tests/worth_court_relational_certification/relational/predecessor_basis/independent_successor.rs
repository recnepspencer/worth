//! Current independent custody is the exact canonical route, never a claimed owner.
use super::*;
use worth_relational::facade::{identity::*, mvcc::*, transactions::*};

fn birth(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
) -> (
    RelationalRequiredOutputPublication,
    RelationalRequiredOutputPublication,
) {
    let owned = world.birth(execution);
    let mut meaning = owned.meaning().clone();
    meaning.family = "independent".into();
    let independent = world
        .publish(
            execution,
            fixture::create("independent-output"),
            RequiredOutputPublication::Birth {
                admission: world.owner.admit_required_output("independent", [3; 32]),
                absence: None,
                meaning,
                roles: vec![RequiredOutputRoleTarget {
                    role: "output".into(),
                    entity_name: "node".into(),
                    kind: KindId(1),
                    posture: RequiredOutputRolePosture::Create,
                    target: EntityReference::Created(fixture::target("independent-output")),
                }],
            },
        )
        .unwrap();
    (independent, owned)
}
fn run(execution: &ExecutionRequest<'_, '_>) {
    let (world, other_owner) = World::with_other_publication_owner(execution);
    let (prior, owned) = birth(&world, execution);
    let old = world.basis();
    world.change_source(execution, "after-independent-publication");
    let basis = world.basis();
    let read = |producer, meaning, basis| {
        world.runtime.readmit_current_independent_output_successor(
            execution,
            &world.installed,
            basis,
            producer,
            meaning,
        )
    };
    let checkpoint = || {
        world
            .runtime
            .durability_authority()
            .native_checkpoint(execution, execution.allocation_policy())
            .unwrap()
    };
    let before = checkpoint();
    let selected = read("independent", prior.meaning(), &basis)
        .unwrap()
        .unwrap();
    assert!(selected.same_origin(&prior));
    assert_eq!(
        selected.members().collect::<Vec<_>>(),
        prior.members().collect::<Vec<_>>()
    );
    assert_eq!(
        selected.authority().publication_coordinate(),
        prior.authority().publication_coordinate()
    );
    for error in [
        world
            .runtime
            .readmit_current_independent_output_successor(
                execution,
                &other_owner,
                &basis,
                "independent",
                prior.meaning(),
            )
            .unwrap_err(),
        read("independent", owned.meaning(), &basis).unwrap_err(),
        read("source-owned", prior.meaning(), &basis).unwrap_err(),
    ] {
        let TransactionCommitError::Publication { error, .. } = error else {
            panic!("{error:?}")
        };
        assert_eq!(error.authored_retirement_refusal(), Some(worth_relational::facade::publication::AuthoredRetirementRefusal::ForeignCoupledOwner));
        assert_eq!(
            error.context.affected_records,
            [RecordRef::Entity(prior.meaning().source_scope)]
        );
        assert_eq!(checkpoint(), before, "refused selection cannot publish");
    }
    assert!(matches!(
        read("independent", prior.meaning(), &old),
        Err(TransactionCommitError::Publication { .. })
    ));
    assert_eq!(
        checkpoint(),
        before,
        "successful selection and stale-basis refusal are read-only"
    );
}
#[test]
fn current_independent_succession_requires_its_actual_owner_and_current_basis() {
    let host = request();
    run(&ExecutionRequest::serial(&host));
}
#[test]
fn worker_only_current_independent_succession_requires_its_actual_owner_and_current_basis() {
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
