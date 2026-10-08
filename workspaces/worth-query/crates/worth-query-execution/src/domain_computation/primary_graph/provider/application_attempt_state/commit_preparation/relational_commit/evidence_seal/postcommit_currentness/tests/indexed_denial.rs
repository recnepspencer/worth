use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationIdentities,
    application_schema::TypedMutationPreconditions,
};
use worth_relational::facade::indexes::{BoundedEntityFieldLookupDenialKind, DerivedIndexId};

use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::IndexedSelectionReobserveDenial,
    application_entry::mutation::HandlerResult,
    tests::fixture::{
        IdentityExecutionSchema, OptionalOutputInput, OptionalOutputMutationBinding,
        OptionalOutputOperation, OptionalOutputPlan,
    },
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationIdempotencyBinding,
};

fn committed_indexed_facts(world: &AuthorizationWorld) -> Vec<WorthQueryApplicationObservedFact> {
    let request = live_scope();
    let principal = crate::domain_computation::primary_graph::tests::application_attempt::authenticated_principal(world, &request);
    let account =
        crate::domain_computation::primary_graph::tests::application_attempt::resolved_account(
            world, "open", &request,
        );
    let operation = world
        .application
        .installed_schema()
        .installed_operation(OptionalOutputOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            TypedMutationPreconditions::new(),
            &request,
        )
        .unwrap();
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
    let completed = world
        .application
        .execute_mutation_handler::<OptionalOutputMutationBinding>(
            &identities,
            principal.principal_identity(),
            admission,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
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
    facts.to_vec()
}

#[test]
fn indexed_postcommit_rebase_preserves_the_native_denial_and_fact_ordinal() {
    let world = installed_authorization_world(true);
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
    let outcome = rebase_result_at_current(&world, facts);
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

#[test]
fn sparse_indexed_rebase_spends_examined_rows_and_keeps_complete_dependencies() {
    let world = installed_authorization_world(true);
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
    let selected = world.selected_product();
    let graph = world.application.runtime.primary_graph().unwrap();
    let observed = graph.integration_handle().with_runtime(|runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        let observed = crate::domain_computation::primary_graph::application_attempt::observe_indexed_entity_selection(
            runtime, snapshot, *index_id, *entity_kind, locator.clone(), value.clone(), 100_001,
        ).expect("the installed index supplies a genuine complete sparse observation");
        assert!(matches!(&observed,
            WorthQueryApplicationObservedFact::IndexedEntitySelection { candidates, .. }
            if candidates.len() == 1
        ));
        let original = vec![observed.clone(), observed.clone()];
        let outcome = rebase(runtime, snapshot,
            PreparedSourceFactRebase::admit(
                original.clone(),
                StorageControl::new(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None),
            ).unwrap(),
            &BTreeSet::new(), true, 4, None);
        let RebasedSourceFacts::Exact(facts) = outcome else {
            panic!("two native one-row probes fit four work units: {outcome:?}");
        };
        assert_eq!(facts.as_ref(), original.as_slice(), "retain the original limits and dependencies");
        for fact in facts.iter() {
            assert_eq!(fact.source_currentness_in(runtime, snapshot, 2), Ok((true, 2)));
        }
        let denied = rebase(runtime, snapshot,
            PreparedSourceFactRebase::admit(
                original.clone(),
                StorageControl::new(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None),
            ).unwrap(),
            &BTreeSet::new(), true, 2, None);
        assert_eq!(denied, RebasedSourceFacts::VerificationRequired {
            reason: RebaseVerificationReason::IndexedSelectionFactDenied(
                1, IndexedSelectionReobserveDenial::WorkBudgetExceeded),
            facts: original.into(),
        }, "exhaustion must preserve the complete original fact set without certification");
        observed
    });
    let (second_account, _) = account_note(&world, "account-2");
    set_note(
        &world,
        second_account,
        locator.clone(),
        "pending-membership",
    );
    let selected = world.selected_product();
    graph.integration_handle().with_runtime(|runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        assert_eq!(
            observed.source_currentness_in(runtime, snapshot, 2),
            Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded)
        );
        let denied = rebase(
            runtime,
            snapshot,
            PreparedSourceFactRebase::admit(
                vec![observed.clone()],
                StorageControl::new(
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    None,
                ),
            )
            .unwrap(),
            &BTreeSet::new(),
            true,
            2,
            None,
        );
        assert_eq!(
            denied,
            RebasedSourceFacts::VerificationRequired {
                reason: RebaseVerificationReason::IndexedSelectionFactDenied(
                    0,
                    IndexedSelectionReobserveDenial::WorkBudgetExceeded
                ),
                facts: vec![observed].into(),
            },
            "a narrowed overflowing probe must not certify a partial posting"
        );
    });
}
