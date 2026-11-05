use crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::retirement::tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::output_correspondence::WorthQueryCheckpointOutputRole;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as WorthQueryApplicationOutputPosture;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
use worth_relational::facade::transactions::RecordRef;

#[test]
fn checkpoint_retirement_requires_this_commits_changed_entity_and_deleted_snapshot() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let (owned, _, facts) = world
        .application
        .with_host_advancement(|phase| {
            reads(
                &phase,
                &phase.borrowed_request_for_test(),
                &world,
                "account-2",
            )
        })
        .expect("the fixture read owns its installed host call");
    // This isolates owner checkpoint selector admission. Typed write/delete agreement
    // is protected by retire_role_requires_the_matching_delete_effect.
    let retirement = restored_role(owned, WorthQueryApplicationOutputPosture::Retire);
    let preservation = restored_role(owned, WorthQueryApplicationOutputPosture::Preserve);
    let changed = [RecordRef::Entity(owned)];
    world
        .application
        .with_host_advancement(|phase| {
            let borrowed = phase.borrowed_request_for_test();
            let execution = &borrowed;
            assert!(
                sealed_rebase(
                    &phase,
                    execution,
                    &world,
                    facts.clone(),
                    &retirement,
                    &changed
                )
                .is_empty(),
                "an exact changed record cannot manufacture retirement of a live entity"
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let borrowed = phase.borrowed_request_for_test();
            delete(&borrowed, &world, owned)
        })
        .expect("the fixture read owns its installed host call");
    let exact = world
        .application
        .with_host_advancement(|phase| {
            let borrowed = phase.borrowed_request_for_test();
            let execution = &borrowed;
            sealed_rebase(
                &phase,
                execution,
                &world,
                facts.clone(),
                &retirement,
                &changed,
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(exact.len(), facts.len());
    world
        .application
        .with_host_advancement(|phase| {
            assert!(selection(
                &phase,
                &phase.borrowed_request_for_test(),
                &world,
                &exact
            ))
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let borrowed = phase.borrowed_request_for_test();
            let execution = &borrowed;
            for (correspondence, changed) in [
                (&retirement, Vec::new()),
                (
                    &retirement,
                    vec![RecordRef::Entity(
                        reads(
                            &phase,
                            &phase.borrowed_request_for_test(),
                            &world,
                            "account-1",
                        )
                        .0,
                    )],
                ),
                (&preservation, vec![RecordRef::Entity(owned)]),
            ] {
                let unqualified = sealed_rebase(
                    &phase,
                    execution,
                    &world,
                    facts.clone(),
                    correspondence,
                    &changed,
                );
                assert!(!unqualified
                    .iter()
                    .any(|fact| matches!(fact, Fact::RetiredOutputEntity { .. })));
                assert!(
                    !selection(
                        &phase,
                        &phase.borrowed_request_for_test(),
                        &world,
                        &unqualified
                    ),
                    "a previous retirement and another commit's changes cannot rebase this commit"
                );
            }
        })
        .expect("the fixture read owns its installed host call")
}

pub fn restored_role(
    entity: EntityId,
    posture: WorthQueryApplicationOutputPosture,
) -> WorthQueryApplicationOutputCorrespondence {
    struct CheckpointBinding;
    struct CheckpointOutputs;
    WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
        TypeId::of_for_test::<CheckpointBinding>(),
        TypeId::of_for_test::<CheckpointOutputs>(),
        std::iter::empty(),
        &[WorthQueryCheckpointOutputRole::new_for_test(
            "output".into(),
            posture,
            "Account".into(),
            entity,
        )],
        |_| Some(TypeId::of_for_test::<Account>()),
        &worth_execution::ExecutionRequest::serial(&worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        )),
    )
    .unwrap()
}

pub fn sealed_rebase(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    facts: Vec<Fact>,
    correspondence: &WorthQueryApplicationOutputCorrespondence,
    changed: &[RecordRef],
) -> worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::FactPayload{
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    (*world
        .application
        .runtime_for_test())
        .primary_graph_for_test()
        .unwrap()
        .integration_handle_for_test()
        .with_runtime(execution, |runtime| {
            exact(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::rebase_output(execution, runtime, selected.application_basis_for_test().snapshot_handle(), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::PreparedSourceFactRebase::admit_for_test(
                    facts,
                    [].into(),
                    StorageControl::new_for_test(
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                        None,
                    ),
                )
                .unwrap(), correspondence, changed, true, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(execution.allocation_policy(), None)))
        })
}
