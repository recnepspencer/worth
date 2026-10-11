//! Actual installed decision projections; admission precedes decoder allocation.
#[path = "predecode_admission/prepared_selection.rs"]
mod prepared_selection;
#[path = "predecode_admission/projection.rs"]
mod projection;
#[path = "predecode_admission/request.rs"]
mod request;
#[path = "predecode_admission/tracked_facts.rs"]
mod tracked_facts;

use super::super::fixture::{
    installed_authorization_world, installed_authorization_world_with_label, live_scope,
    AccountLabel, AccountStatus,
};
use super::{admitted_operation, authenticated_principal, resolved_account};
use crate::domain_computation::primary_graph::WorthQueryInvariantDecisionPlanDenialKind;
use std::cell::Cell;
use worth_foundational::facade::{AspectValue, InternedString};

// The compile-time marker alone must not broaden the installed decision plan.
worth_query_declaration::worth_query_operation_reads!(
    super::super::fixture::ProgramRequiredOperation => [AccountLabel]
);

#[test]
fn installed_handler_reader_admits_actual_raw_scalar_and_uses_declared_decoder() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let completed = projection::with_reader(&world, &request, |reader| {
        let scope = reader.scope().clone();
        reader.field_with_predecode_admission(&scope, AccountStatus::reference(), |raw, check| {
            check().unwrap();
            let AspectValue::String(InternedString::Raw(text)) = raw else {
                panic!("actual native string carrier expected")
            };
            assert_eq!(text, "open");
            assert_eq!(raw.semantic_byte_width(), 5);
            assert!(raw.owned_allocation_capacity_bytes() >= text.len());
            Ok::<_, ()>(())
        })
    });
    assert_eq!(completed.unwrap().unwrap(), Some("open".to_owned()));
}

#[test]
fn actual_admission_denial_is_preserved_without_a_decoded_value() {
    #[derive(Debug, PartialEq)]
    struct OwnerDenied(usize);
    let world = installed_authorization_world(true);
    let request = live_scope();
    let result = projection::with_reader(&world, &request, |reader| {
        let scope = reader.scope().clone();
        reader.field_with_predecode_admission(&scope, AccountStatus::reference(), |raw, _| {
            Err(OwnerDenied(raw.semantic_byte_width()))
        })
    });
    assert_eq!(result.unwrap(), Err(OwnerDenied(5)));
}

#[test]
fn large_carrier_denial_allocates_only_projection_metadata_before_decoder() {
    // Serial focused lane measures this real call, excluding fixture/label seed.
    // Any eager scalar clone or String binding decode would allocate >=128KiB.
    let payload = "q".repeat(128 * 1024);
    let world = installed_authorization_world_with_label(&payload);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let admission = admitted_operation(&world, &principal, &account, &request);
    let mut allocated = 0;
    let completed = world
        .invariant
        .project_admitted_operation(
            &admission,
            |reader, scope| {
                let region = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
                let result = reader.decision_field_with_predecode_admission(
                    scope,
                    AccountLabel::reference(),
                    |raw, _| {
                        let AspectValue::String(InternedString::Raw(actual)) = raw else {
                            panic!("actual label carrier expected")
                        };
                        assert_eq!(actual.len(), payload.len());
                        Err::<(), _>("owner-work-limit")
                    },
                    &|| Ok(()),
                );
                allocated = region.change().bytes_allocated;
                result
            },
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert_eq!(
        completed.output().as_ref().unwrap(),
        &Err("owner-work-limit")
    );
    assert!(
        allocated < payload.len(),
        "an eager carrier copy allocated {allocated} bytes"
    );
    assert_eq!(completed.work().field_reads(), 1);
}

#[test]
fn ordinary_decision_field_decodes_without_a_carrier_copy() {
    let filter = concat!(module_path!(), "::isolated_ordinary_field_decode")
        .split_once("::")
        .unwrap()
        .1;
    let output = crate::process_deadline::output(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", filter, "--test-threads=1", "--nocapture"])
            .env("WORTH_QUERY_BORROWED_FIELD_DECODE_PROBE", "1"),
    )
    .unwrap();
    assert!(
        output.status.success(),
        "ordinary field allocation probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
}

#[test]
fn isolated_ordinary_field_decode() {
    if std::env::var_os("WORTH_QUERY_BORROWED_FIELD_DECODE_PROBE").is_none() {
        return;
    }
    let small = installed_authorization_world_with_label("small");
    let (_, fixed_allocation) = ordinary_field_allocation(&small);
    let payload = "q".repeat(128 * 1024);
    let large = installed_authorization_world_with_label(&payload);
    let (actual, allocated) = ordinary_field_allocation(&large);
    assert_eq!(actual, payload);
    assert!(
        allocated <= fixed_allocation + payload.len(),
        "only the owned decoded String grows with the carrier: \
         allocated {allocated}, fixed allocation {fixed_allocation}"
    );
}

fn ordinary_field_allocation(world: &super::super::fixture::AuthorizationWorld) -> (String, usize) {
    let request = live_scope();
    let principal = authenticated_principal(world, &request);
    let account = resolved_account(world, "open", &request);
    let admission = admitted_operation(world, &principal, &account, &request);
    let mut allocated = 0;
    let completed = world
        .invariant
        .project_admitted_operation(
            &admission,
            |reader, scope| {
                // Measure only this ordinary field call, not installation, authorization,
                // completion or retained read-set capture. This is not a peak-byte proof.
                let region = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
                let actual = reader
                    .decision_field(scope, AccountLabel::reference())
                    .unwrap()
                    .unwrap();
                allocated = region.change().bytes_allocated;
                actual
            },
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert_eq!(completed.work().field_reads(), 1);
    let (actual, projection, _) = completed.into_parts();
    let _reads = world
        .application
        .begin_projected_application_read_attempt(
            admission,
            projection,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .complete_projected_dependencies(
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    (actual, allocated)
}

#[test]
fn undeclared_and_foreign_reads_deny_before_raw_callback() {
    let world = installed_authorization_world(true);
    let other = installed_authorization_world(true);
    let request = live_scope();
    let foreign = other
        .invariant
        .project(
            |reader| {
                reader
                    .resolve_entity(AccountStatus::reference(), "open".into())
                    .unwrap()
            },
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts()
        .0;
    let called = Cell::new(false);
    let result = projection::with_reader(&world, &request, |reader| {
        reader.field_with_predecode_admission(&foreign, AccountStatus::reference(), |_, _| {
            called.set(true);
            Ok::<_, ()>(())
        })
    });
    let denial = result
        .unwrap_err()
        .downcast::<crate::domain_computation::primary_graph::WorthQueryInvariantDecisionPlanDenial>()
        .unwrap();
    assert_eq!(
        denial.kind(),
        WorthQueryInvariantDecisionPlanDenialKind::ForeignIdentity
    );
    assert!(!called.get());

    let result = projection::with_reader(&world, &request, |reader| {
        let scope = reader.scope().clone();
        // ProgramRequiredOperation declares only AccountStatus, not AccountLabel.
        reader.field_with_predecode_admission(&scope, AccountLabel::reference(), |_, _| {
            called.set(true);
            Ok::<_, ()>(())
        })
    });
    let denial = result
        .unwrap_err()
        .downcast::<crate::domain_computation::primary_graph::WorthQueryInvariantDecisionPlanDenial>()
        .unwrap();
    assert_eq!(
        denial.kind(),
        WorthQueryInvariantDecisionPlanDenialKind::UndeclaredDecisionTarget
    );
    assert!(!called.get());
}
