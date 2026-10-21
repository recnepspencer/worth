use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind;
use worth_relational::facade::indexes::DerivedIndexId;

use super::*;

use crate::domain_computation::primary_graph::application_attempt::IndexedSelectionReobserveDenial;
use crate::domain_computation::primary_graph::application_entry::mutation::HandlerResult;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::OptionalOutputInput;
use crate::domain_computation::primary_graph::tests::fixture::OptionalOutputMutationBinding;
use crate::domain_computation::primary_graph::tests::fixture::OptionalOutputOperation;
use crate::domain_computation::primary_graph::tests::fixture::OptionalOutputPlan;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;

fn committed_indexed_facts(world: &AuthorizationWorld) -> Vec<WorthQueryApplicationObservedFact> {
    let request = live_scope();
    let principal = world.application.with_host_advancement(|phase| { let phase_custody = &phase;    crate::domain_computation::primary_graph::tests::application_attempt::authenticated_principal(&(phase_custody).test_host_request(), world, &request) }).expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let phase_custody = &phase;

            crate::domain_computation::primary_graph::tests::application_attempt::resolved_account(
                &(phase_custody).test_host_request(),
                world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(OptionalOutputOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            let phase_custody = &phase;

            world
                .selected_product(&(phase_custody).test_host_request())
                .authorize_operation(
                    phase_custody,
                    &principal,
                    &account,
                    &operation,
                    TypedMutationPreconditions::new(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let input = OptionalOutputInput {
        status: "open".to_owned(),
        plan: OptionalOutputPlan::RequiredAfterIndexedAbsence,
    };
    let request_id = "postcommit-indexed-denial".to_owned();
    let identities = ApplicationMutationIdentities::<
        IdentityExecutionSchema,
        OptionalOutputMutationBinding,
    >::encode(&request_id, &input)
    .unwrap();
    // Candidate preparation and publication are separate host calls.
    let completed = world
        .application
        .with_application_advancement(&request, |phase| {
            world
                .application
                .execute_mutation_handler::<OptionalOutputMutationBinding>(
                    &phase,
                    &identities,
                    principal.principal_identity(),
                    admission,
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let HandlerResult::Completed(completed) = completed else {
        panic!("the installed handler must read the indexed absence");
    };
    let (candidate, _) = completed.into_parts();
    let key = WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities);
    let WorthQueryApplicationCommitOutcome::Committed(receipt) =
        world.application.compare_and_commit_application(
            candidate,
            key,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the installed indexed decision must commit");
    };
    let facts = world
        .application
        .primary_provider
        .graph
        .output_lineage
        .lock()
        .unwrap()
        .checkpoint_facts_for_receipt(&receipt)
        .unwrap()
        .0;
    facts.iter().cloned().collect()
}

#[test]
fn indexed_postcommit_rebase_preserves_the_native_denial_and_fact_ordinal() {
    let world = installed_authorization_world(true);
    {
        let mut facts = committed_indexed_facts(&world);
        let mut ordinal = facts
            .iter()
            .position(|fact| {
                matches!(
                    fact,
                    WorthQueryApplicationObservedFact::IndexedEntitySelection { .. }
                )
            })
            .expect("the real handler must retain its indexed fact");
        // Repeat the actual retained observation so successful reobservation
        // precedes the faulted observation at a nonzero fact ordinal.
        let unchanged = facts[ordinal].clone();
        facts.insert(ordinal, unchanged);
        ordinal += 1;
        assert!(
            ordinal > 0,
            "the real indexed fact occupies a nonzero test ordinal"
        );
        let WorthQueryApplicationObservedFact::IndexedEntitySelection { index_id, .. } =
            &mut facts[ordinal]
        else {
            unreachable!();
        };
        *index_id = DerivedIndexId(u64::MAX);
        let outcome = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();

                rebase_result_at_current(
                    &host_custody.phase_for_application(&world.application),
                    &host_custody.borrowed_request_for_test(),
                    &world,
                    facts,
                )
            })
            .expect("the fixture read owns its installed host call");
        assert!(
            matches!(
                &outcome,
                RebasedSourceFacts::VerificationRequired {
                    reason: RebaseVerificationReason::IndexedSelectionFactDenied(
                        actual_ordinal,
                        IndexedSelectionReobserveDenial::Lookup(
                            BoundedEntityFieldLookupDenialKind::IndexNotInstalled
                        )
                    ),
                    ..
                } if *actual_ordinal == ordinal
            ),
            "the native denial and its actual nonzero ordinal must survive rebase: {outcome:?}"
        );
    }
}

#[test]
fn sparse_indexed_rebase_keeps_complete_dependencies_without_work_estimates() {
    let world = installed_authorization_world(true);
    {
        let committed = committed_indexed_facts(&world);
        let selected_value =
            StringApplicationValueBinding::encode(&"pending-membership".to_owned()).unwrap();
        let retained = committed
            .iter()
            .find(|fact| {
                matches!(
                    fact,
                    WorthQueryApplicationObservedFact::IndexedEntitySelection { value, .. }
                    if *value == selected_value
                )
            })
            .unwrap();
        let WorthQueryApplicationObservedFact::IndexedEntitySelection {
            index_id,
            entity_kind,
            locator,
            value,
            ..
        } = retained
        else {
            unreachable!()
        };
        let selected = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();

                world.selected_product(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                )
            })
            .expect("the fixture read owns its installed host call");
        let graph = world.application.runtime.primary_graph().unwrap();
        let observed = world.application.with_host_advancement(|phase| {   let borrowed_execution = phase.execution_request_for(&world.application.product_runtime).expect("fixture borrows its runtime request"); let execution = &borrowed_execution; graph.integration_handle().with_runtime(&phase.borrowed_request_for_test(), |runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        let observed = crate::domain_computation::primary_graph::application_attempt::observe_indexed_entity_selection(
            runtime,execution,  snapshot, *index_id, *entity_kind, locator.clone(), value.clone(), 100_001,
        ).expect("the installed index supplies a genuine complete sparse observation");
        assert!(matches!(&observed,
            WorthQueryApplicationObservedFact::IndexedEntitySelection { candidates, .. }
            if candidates.len() == 1
        ));
        for invalid_limit in [false, true] {
            let mut faulted = observed.clone();
            let WorthQueryApplicationObservedFact::IndexedEntitySelection { index_id, candidate_limit, .. } = &mut faulted else { unreachable!() };
            if invalid_limit { *candidate_limit = 0; } else { *index_id = DerivedIndexId(u64::MAX); }

            assert_eq!(faulted.source_currentness_in(runtime,execution,  snapshot),
                Ok(Err(WorthQuerySourceCurrentnessFailure::Unavailable)));

        }
        let original = vec![observed.clone(), observed.clone()];
        let outcome = rebase(execution, runtime, snapshot, PreparedSourceFactRebase::admit(
                original.clone(),
                [].into(),
                StorageControl::new(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None),
            ).unwrap(), &BTreeSet::new(), true);
        let RebasedSourceFacts::Exact(facts) = outcome else {
            panic!("native reobservation keeps the complete original facts: {outcome:?}");
        };
        assert_eq!(facts.iter().cloned().collect::<Vec<_>>(), original.as_slice(), "retain the original limits and dependencies");
        for fact in facts.iter() {
            let movement = fact.source_currentness_in(runtime,execution,  snapshot).unwrap().unwrap();
            assert_eq!(movement.movement(), Movement::Unmoved);
        }
        observed
    }) }).expect("the fixture read owns its installed host call");
        let (second_account, _) = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();

                account_note(
                    &host_custody.phase_for_application(&world.application),
                    &world,
                    "account-2",
                )
            })
            .expect("the fixture read owns its installed host call");
        world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();

                set_note(
                    &host_custody.borrowed_request_for_test(),
                    &world,
                    second_account,
                    locator.clone(),
                    "pending-membership",
                )
            })
            .expect("the fixture read owns its installed host call");
        let selected = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();

                world.selected_product(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                )
            })
            .expect("the fixture read owns its installed host call");
        world
            .application
            .with_host_advancement(|phase| {
                let borrowed_execution = phase
                    .execution_request_for(&world.application.product_runtime)
                    .expect("fixture borrows its runtime request");
                let execution = &borrowed_execution;
                graph
                    .integration_handle()
                    .with_runtime(execution, |runtime| {
                        let snapshot = selected.application_basis().snapshot_handle();

                        let current = observed.source_currentness_in(runtime, execution, snapshot)
                            .expect("the real lookup remains available")
                            .expect("the installed source is comparable");
                        assert_eq!(current.movement(), Movement::Moved,
                            "the independently authored second matching account changes membership");
                        let normalized = rebase(execution, runtime, snapshot,
                            PreparedSourceFactRebase::admit(vec![observed.clone()], [].into(),
                                StorageControl::new(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None)).unwrap(),
                            &BTreeSet::new(), true);
                        let RebasedSourceFacts::Exact(facts) = normalized else {
                            panic!("postcondition membership is normalized by the native lookup: {normalized:?}");
                        };
                        let WorthQueryApplicationObservedFact::IndexedEntitySelection { candidates, .. } = &facts[0] else {
                            panic!("normalization keeps the actual indexed fact");
                        };
                        let WorthQueryApplicationObservedFact::IndexedEntitySelection { candidates: original, .. } = &observed else { unreachable!() };
                        let mut expected = original.clone();
                        expected.push(second_account);
                        expected.sort_unstable();
                        expected.dedup();
                        assert_eq!(candidates, &expected);

                    })
            })
            .expect("the fixture read owns its installed host call");
    }
}
