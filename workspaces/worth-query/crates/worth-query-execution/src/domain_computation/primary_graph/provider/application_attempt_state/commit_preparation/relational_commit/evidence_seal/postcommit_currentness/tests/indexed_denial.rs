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

#[test]
fn indexed_postcommit_rebase_preserves_the_native_denial_and_fact_ordinal() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = crate::domain_computation::primary_graph::tests::application_attempt::authenticated_principal(&world, &request);
    let account =
        crate::domain_computation::primary_graph::tests::application_attempt::resolved_account(
            &world, "open", &request,
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
        )
        .unwrap();
    let HandlerResult::Completed(completed) = completed else {
        panic!("the installed handler must read the indexed absence");
    };
    let (candidate, _) = completed.into_parts();
    let key = WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities);
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = world
        .application
        .compare_and_commit_application(candidate, key)
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
    let mut facts = facts.to_vec();
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
