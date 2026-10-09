//! Genuine Native computations that change their own source inputs at commit.
use std::sync::Arc;
use worth_execution::ExecutionAllocationPolicy;
#[cfg(test)]
mod comparison_access;
mod index_loss;
use crate::domain_computation::primary_graph::{
    application_attempt::{ComputationRead, OutputRoleUse},
    output_lineage::RecordedOutput,
    tests::{
        application_attempt::{
            authenticated_principal, idempotency,
            preimage_evidence::{
                retained_status_program, RetainedAccount, RetentionMutationBreadth,
                RetentionOutputs,
            },
            resolved_account,
        },
        fixture::{
            live_scope,
            own_write_computation::{GeneratedAccount, OwnWriteComputation, OwnWriteOutputs},
            AccountLabel, AccountStatus, AuthorizationWorld,
        },
    },
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt,
};
pub(in crate::domain_computation::primary_graph) use index_loss::evict_index_with_unrelated_write;
use worth_foundational::facade::PartitionIdentity;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

pub(in crate::domain_computation::primary_graph) fn with_committed_own_write(
    test: impl FnOnce(
        &AuthorizationWorld,
        WorthQueryApplicationCommitReceipt,
        Arc<std::sync::OnceLock<RecordedOutput>>,
        crate::domain_computation::execution_runtime::WorthQueryInvalidationResources,
    ),
) {
    commit_own_write(false, test);
}
pub(in crate::domain_computation::primary_graph) fn with_generated_own_write(
    test: impl FnOnce(
        &AuthorizationWorld,
        WorthQueryApplicationCommitReceipt,
        Arc<std::sync::OnceLock<RecordedOutput>>,
        crate::domain_computation::execution_runtime::WorthQueryInvalidationResources,
    ),
) {
    commit_own_write(true, test);
}
fn commit_own_write(
    generated: bool,
    test: impl FnOnce(
        &AuthorizationWorld,
        WorthQueryApplicationCommitReceipt,
        Arc<std::sync::OnceLock<RecordedOutput>>,
        crate::domain_computation::execution_runtime::WorthQueryInvalidationResources,
    ),
) {
    let product_resources =
        crate::domain_computation::execution_runtime::product_world::test_product_world_resources();
    let resources = product_resources.invalidation_resources();
    let world = crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world_with_product_resources(product_resources);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let seed = retained_status_program(
        &world,
        &principal,
        &account,
        &request,
        "1",
        RetentionMutationBreadth::Narrow,
    );
    assert!(matches!(
        world.application.compare_and_commit_application(
            seed,
            idempotency(231, 231),
            ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "1", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(OwnWriteComputation::reference())
        .unwrap();
    let mut admitted = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            TypedMutationPreconditions::new(),
            &request,
        )
        .unwrap();
    admitted.bind_source_partition([7; 32]);
    let (computed, projection, _) = world
        .invariant
        .project_admitted_operation(
            &admitted,
            |reader, projected| {
                reader
                    .require_decision_field(projected, AccountLabel::reference())
                    .unwrap();
                reader.begin_computation_reads();
                let x = reader.attributed(
                    ComputationRead::Partition(PartitionIdentity::new(7)),
                    |reader| {
                        reader
                            .decision_field(projected, AccountStatus::reference())
                            .unwrap()
                            .unwrap()
                    },
                );
                assert_eq!(x, "1");
                format!("computed-from-{x}")
            },
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(
            admitted,
            projection,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
        .begin_effect_program();
    let source = effects.existing_entity(&account).unwrap();
    let output = if generated {
        effects.prepare_output_contract_for_test::<OwnWriteOutputs>();
        let output = effects
            .create_entity(
                crate::domain_computation::primary_graph::tests::fixture::Account::reference(),
                crate::domain_computation::primary_graph::WorthQueryApplicationEntityKey::new(
                    "computed-own-write".to_owned(),
                )
                .unwrap(),
            )
            .unwrap();
        effects
            .bind_output(OutputRoleUse::fixed::<GeneratedAccount>(), &output)
            .unwrap();
        effects.initialize_field(&output, crate::domain_computation::primary_graph::tests::fixture::AccountIdentity::reference(), "computed-own-write".to_owned()).unwrap();
        effects.initialize_field(&output, crate::domain_computation::primary_graph::tests::fixture::AccountMembershipTag::reference(), "open".to_owned()).unwrap();
        effects
            .initialize_field(&output, AccountStatus::reference(), "generated".to_owned())
            .unwrap();
        output
    } else {
        effects.prepare_output_contract_for_test::<RetentionOutputs>();
        effects
            .bind_output(OutputRoleUse::fixed::<RetainedAccount>(), &source)
            .unwrap();
        effects.existing_entity(&account).unwrap()
    };
    if generated {
        effects
            .initialize_field(&output, AccountLabel::reference(), computed)
            .unwrap();
    } else {
        effects
            .write_field(&output, AccountLabel::reference(), computed)
            .unwrap();
    }
    effects
        .write_field(&source, AccountStatus::reference(), "2".to_owned())
        .unwrap();
    let mut program = effects.finish().unwrap().with_output_demand_observation();
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let maximum = resources.installation().maximum_retained_bytes;
    let (key, dependency) = if generated {
        program.producer_idempotency_identities::<OwnWriteOutputs>(
            &world.application,
            [11; 32],
            None,
            &mut owner.edit_admission(),
        )
    } else {
        program.producer_idempotency_identities::<RetentionOutputs>(
            &world.application,
            [11; 32],
            None,
            &mut owner.edit_admission(),
        )
    }
    .unwrap();
    let identity =
        crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding::new(
            key, [232; 32],
        )
        .bind_source(Some(&[11; 32]))
        .bind_source_partition(&[7; 32])
        .bind_producer_dependency(&dependency);
    let outcome = world.application.compare_and_commit_application(
        program,
        identity,
        ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
        panic!("the legal effect commits: {outcome:?}");
    };
    assert!(resources.retained_capacity_bytes() <= maximum);
    if generated {
        // The tested boundary starts at an already performed record, not at
        // producer selection. Give that real computation its test edition and
        // prepared-input metadata; Native facts, own-effect evidence and output
        // witness remain the actual commit's. These identities are not under test.
        let mut lineage = handle.output_lineage.lock().unwrap();
        let cell = lineage
            .by_source
            .values_mut()
            .flat_map(|branches| branches.values_mut())
            .flat_map(|history| history.values_mut())
            .flat_map(|rows| rows.iter_mut())
            .rfind(|cell| cell.get().unwrap().source_partition_identity == Some([7; 32]))
            .unwrap();
        let recorded = Arc::get_mut(cell)
            .expect("performed cell has only lineage custody")
            .get_mut()
            .unwrap();
        let count = recorded.observed_source_facts().unwrap().len();
        let boundary = crate::domain_computation::primary_graph::application_attempt::CompletedHandlerFactBoundary::completed_for_test(count);
        let context = crate::domain_computation::primary_graph::application_attempt::PreparedDecisionReuseContext::new(
            crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerInputReuseContract::canonical_bitwise(
                crate::domain_computation::primary_graph::application_contribution::WorthQueryDecisionContextDependencies::KEY), Some([11;32]), None, None).unwrap();
        recorded.completed_decision_reuse = boundary.seal_decision_reuse(
            context,
            crate::domain_computation::primary_graph::DecisionContextUse::default().key_for_test(),
        );
        recorded.completed_handler_facts = Some(boundary);
        recorded.prepared_input_reuse_key = Some(super::PreparedInputReuseKey::new(
            crate::domain_computation::primary_graph::application_query::WorthQueryObservedSourceSelection::selecting_for_test(u64::try_from(count).unwrap()),
            [11;32], crate::domain_computation::primary_graph::application_contribution::InstalledProducerEdition::for_test([12;32])));
    }
    let cell = handle
        .output_lineage
        .lock()
        .unwrap()
        .by_source
        .values()
        .flat_map(|branches| branches.values())
        .flat_map(|history| history.values())
        .flat_map(|rows| rows.iter())
        .rfind(|cell| cell.get().unwrap().source_partition_identity == Some([7; 32]))
        .cloned()
        .expect("the committed output has a retained cell");
    let completed = world
        .application
        .primary_provider
        .observe_completed_application(receipt.commit_reference())
        .unwrap();
    assert!(
        !completed
            .commit_evidence()
            .source_facts_superseded_by_own_effect()
            .is_empty(),
        "the actual commit carries the computation's own-write staleness"
    );
    test(&world, receipt, cell, resources);
}
