//! Real committed output read with all remaining invalidation capacity held.
use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::{WorthQueryProducerApplicability, WorthQueryProducerOutputFamily},
    tests::application_attempt::{
        authenticated_principal, idempotency,
        preimage_evidence::{retained_status_program, RetentionMutationBreadth, RetentionOutputs},
        resolved_account,
    },
    WorthQueryApplicationCommitOutcome, WorthQueryCurrentOutputDenialKind,
    WorthQueryOperationProjectionDenial,
};
use std::{any::TypeId, collections::BTreeMap};
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

// This operation isolates the retained-capacity boundary. Its work budget is
// the installed owner's standard marking ceiling, independent of retained bytes.
worth_query_operation!(pub(in crate::domain_computation::primary_graph) ReadRetainedAccount for IdentityExecutionSchema, input TouchAccountInputBinding);
worth_query_operation_reads!(ReadRetainedAccount => [Account]);
worth_query_operation_requires!(ReadRetainedAccount => [ViewAccount]);

pub(super) fn declare(
    schema: worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder<IdentityExecutionSchema>,
) -> worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder<
    IdentityExecutionSchema,
> {
    let operation = ReadRetainedAccount::reference();
    let work =
        crate::domain_computation::execution_runtime::product_world::test_product_world_resources()
            .invalidation_resources()
            .installation()
            .maximum_marking_work;
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, usize::try_from(work).unwrap())
        .operation_read_entity(operation, Account::reference())
        .operation_requires_ability(operation, ViewAccount::reference())
}

pub(in crate::domain_computation::primary_graph) struct RetainedFamily;
impl WorthQueryProducerOutputFamily<IdentityExecutionSchema> for RetainedFamily {
    type Source = TestAccountSourceBinding;
    type Entity = Account;
    const IDENTITY: &'static str = "test.retained-account-capacity";
    const SUPPORTED: &'static [WorthQueryProducerApplicability] = &[];
    fn profile_kind(_: &AccountSummaryResult) -> &'static str {
        "retained"
    }
}

pub(in crate::domain_computation::primary_graph) fn project_at_full_ledger() -> (
    WorthQueryCurrentOutputDenialKind,
    WorthQueryOperationProjectionDenial,
) {
    let product_resources =
        crate::domain_computation::execution_runtime::product_world::test_product_world_resources();
    let resources = product_resources.invalidation_resources();
    let world = installed_authorization_world_with_product_resources(product_resources);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let program = retained_status_program(
        &world,
        &principal,
        &account,
        &request,
        "open",
        RetentionMutationBreadth::Narrow,
    )
    .with_output_demand_observation();
    let identity = idempotency(233, 233)
        .bind_source(Some(&[17; 32]))
        .bind_source_partition(&[13; 32])
        .bind_producer_dependency(&[19; 32]);
    assert!(matches!(
        world.application.compare_and_commit_application(
            program,
            identity,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    // A concrete binding/role registry is the reader boundary's installation.
    // The correspondence and sealed witness came from the real World effect.
    handle
        .output_lineage
        .lock()
        .unwrap()
        .install_output_families(BTreeMap::from([(
            RetainedFamily::IDENTITY.to_owned(),
            vec![(
                TypeId::of::<RetentionOutputs>(),
                "retained-account".to_owned(),
            )],
        )]));
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ReadRetainedAccount::reference())
        .unwrap();
    let admitted = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            TypedMutationPreconditions::new(),
            &request,
        )
        .unwrap();
    let maximum = resources.installation().maximum_retained_bytes;
    let held = resources
        .reserve_retained_capacity(maximum - resources.retained_capacity_bytes())
        .unwrap();
    let mut seen = None;
    let projected = world.invariant.project_admitted_operation(
        &admitted,
        |reader, root| {
            let denied = reader
                .current_output::<RetainedFamily, Account>(root)
                .err()
                .expect("the full ledger cannot retain the consumed edge");
            seen = Some(denied.kind());
        },
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert_eq!(resources.retained_capacity_bytes(), maximum);
    drop(held);
    (
        seen.expect("the handler ran"),
        projected
            .err()
            .expect("the projection returns its capacity stop"),
    )
}
