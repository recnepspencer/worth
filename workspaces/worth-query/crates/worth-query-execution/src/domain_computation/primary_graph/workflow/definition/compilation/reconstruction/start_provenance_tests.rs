//! Lower-level canonical producer/native data courts, not a public workflow journey.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationRealizedEffect as Effect,
    tests::{
        application_attempt::{admitted_operation, authenticated_principal, resolved_account},
        fixture::*,
    },
    workflow::{
        actor::record::WorkflowAdmittedActor,
        instance::{start_provenance, visit_instance_start_facts, WorkflowLineageCarry},
    },
};
use worth_execution::{CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest};
use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::{runtime::RelationalRuntime, transactions::*};

mod corruption;
mod definition_fixture;
mod fixture;
use fixture::*;

#[test]
fn started_revision_actual_producer_is_immutable_but_current_revision_can_change() {
    let world = installed_authorization_world(true);
    let (support, original, successor) =
        installed_provenance_support(world.application.installed_schema());
    assert_ne!(original, successor);
    let original_payer = payer();
    let execution = ExecutionRequest::leased(&original_payer);
    let layout = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout
        .workflow()
        .clone();
    let actor = admitted_actor(&world);
    let instance = publish_instance(&world, &execution, &layout, original, "original", actor);
    let before = selected(&world, &execution);
    let handle = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle();
    handle.with_runtime(&execution, |runtime| {
        assert_eq!(
            start_provenance::read(
                &execution,
                runtime,
                before.1.snapshot_handle(),
                &layout,
                instance,
                &support
            )
            .unwrap(),
            original
        );
        assert_eq!(
            text_at(
                runtime,
                &execution,
                before.1.snapshot_handle(),
                instance,
                layout.instance.entity_kind,
                &layout.instance.started_program_revision,
                true
            ),
            original.to_string()
        );
    }); // No active deployment/activation is consulted for historical DATA.
    let replacement = handle.with_runtime(&execution, |runtime| {
        start_provenance::admit(&execution, runtime, &before.0, &successor).unwrap()
    });
    let candidate = handle.with_runtime(&execution, |runtime| {
        let mut transaction = runtime
            .begin_branch_transaction(
                &before.0,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .unwrap();
        transaction
            .push_batch(
                update(
                    instance,
                    &layout.instance.started_program_revision,
                    replacement,
                ),
                execution.allocation_policy(),
            )
            .unwrap();
        runtime.prepare_branch_transaction(&execution, transaction)
    });
    let Err(TransactionCommitError::Conflict { error, .. }) = candidate else {
        panic!("schema-valid provenance replacement must hit native custody");
    };
    assert!(matches!(
        error.class,
        ConflictClass::InvariantViolation { .. }
    ));
    assert!(
        error
            .detail
            .contains("worth-query.workflow.publication-immutability"),
        "{error:?}"
    );
    let after = selected(&world, &execution);
    assert_eq!(before.0.descriptor(), after.0.descriptor());
    // This is a native mutability counterpart, NOT a program adoption admission.
    publish_relational_mutation(&execution, &world, update(instance, &layout.instance.program_revision,
        crate::domain_computation::primary_graph::program_occurrence::program_revision_rendering(&successor)));
    let current = selected(&world, &execution);
    handle.with_runtime(&execution, |runtime| {
        assert_eq!(
            start_provenance::read(
                &execution,
                runtime,
                current.1.snapshot_handle(),
                &layout,
                instance,
                &support
            )
            .unwrap(),
            original
        );
        assert_eq!(
            start_provenance::read(
                &execution,
                runtime,
                before.1.snapshot_handle(),
                &layout,
                instance,
                &support
            )
            .unwrap(),
            original
        );
        assert_eq!(
            text_at(
                runtime,
                &execution,
                current.1.snapshot_handle(),
                instance,
                layout.instance.entity_kind,
                &layout.instance.program_revision,
                false
            ),
            successor.to_string()
        );
        assert_eq!(
            text_at(
                runtime,
                &execution,
                before.1.snapshot_handle(),
                instance,
                layout.instance.entity_kind,
                &layout.instance.program_revision,
                false
            ),
            original.to_string()
        );
    });
    let next = publish_instance(&world, &execution, &layout, successor, "successor", actor);
    let latest = selected(&world, &execution);
    handle.with_runtime(&execution, |runtime| {
        assert_eq!(
            start_provenance::read(
                &execution,
                runtime,
                latest.1.snapshot_handle(),
                &layout,
                next,
                &support
            )
            .unwrap(),
            successor
        );
        assert_eq!(
            start_provenance::read(
                &execution,
                runtime,
                latest.1.snapshot_handle(),
                &layout,
                instance,
                &support
            )
            .unwrap(),
            original
        );
    });
}

#[test]
fn started_revision_current_capture_restore_retains_exact_data_after_original_payer_drop() {
    let world = installed_authorization_world(true);
    let (support, original, _) = installed_provenance_support(world.application.installed_schema());
    let original_payer = payer();
    let execution = ExecutionRequest::leased(&original_payer);
    let layout = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout
        .workflow()
        .clone();
    let recovery_config = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle()
        .with_runtime(&execution, |runtime| runtime.config()); // before managed revision text exists
    let instance = publish_instance(
        &world,
        &execution,
        &layout,
        original,
        "capture",
        admitted_actor(&world),
    );
    let handle = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle();
    let checkpoint = handle.with_runtime(&execution, |runtime| {
        runtime
            .durability_authority()
            .native_checkpoint(execution.allocation_policy())
            .unwrap()
    });
    let bytes = checkpoint.bytes().to_vec();
    drop((handle, world, original_payer));
    let receiving_payer = payer();
    let receiving = ExecutionRequest::leased(&receiving_payer);
    let mut restored = RelationalRuntime::new(recovery_config);
    restored
        .durability_recovery()
        .restore_native_checkpoint(&receiving, &checkpoint)
        .unwrap();
    let snapshot = native_snapshot(&restored);
    assert_eq!(
        start_provenance::read(&receiving, &restored, &snapshot, &layout, instance, &support)
            .unwrap(),
        original
    );
    assert_eq!(
        text_at(
            &restored,
            &receiving,
            &snapshot,
            instance,
            layout.instance.entity_kind,
            &layout.instance.started_program_revision,
            true
        ),
        original.to_string()
    );
    assert_eq!(checkpoint.bytes(), bytes);
}
