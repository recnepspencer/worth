//! Genuine Native outputs, policies and terminals qualify the paired issuer.
use super::super::{CausalInspectionReason, WorthQueryApplicationQueryObservation};
use super::*;
use crate::domain_computation::authorization::application_disclosure::DiagnosticComparisonDenial;
use crate::domain_computation::primary_graph::admitted_capture::CaptureStop;
use crate::domain_computation::primary_graph::tests::fixture::{
    diagnostic_comparison::{
        DeniedDiagnosticFieldBinding, DeniedDiagnosticInternalBinding,
        DeniedDiagnosticRelationBinding, DiagnosticAccountBinding,
        GovernedDiagnosticAccountBinding,
    },
    installed_inspection_capability_world, live_scope, AccountLabel, AccountNote, ActivitySequence,
    AuthorizationWorld, IdentityExecutionSchema, TestAccountSourceBinding,
};
use std::{num::NonZeroUsize, time::Duration};
use worth_execution::{ExecutionAllocationPolicy, MapKernelFailure, ScopeStop};
use worth_foundational::facade::AspectValue;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
#[path = "comparison_controls.rs"]
mod controls;
#[path = "comparison_reads.rs"]
pub(super) mod reads;
use reads::{
    account, activity, capability, clear_optional_field, compare, read, write, DiagnosticBinding,
};

fn comparison_world() -> AuthorizationWorld {
    let world = installed_inspection_capability_world();
    world
        .authorization_time
        .hold(std::time::UNIX_EPOCH + Duration::from_secs(100));
    world
}

#[test]
fn paired_originals_report_real_changed_and_equal_rewrite_with_one_original_payer() {
    let world = comparison_world();
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let registry = world.application.application_query_basis_observer();
    let baseline_readers = registry.observe().active();
    let (before, rows) =
        read::<GovernedDiagnosticAccountBinding>(&world, &external, &request, "open", true);
    assert_eq!(rows[0].label, "primary");
    assert_eq!(rows[0].note.as_deref(), Some("reviewed"));
    assert_eq!(
        rows[0].activities,
        vec![
            ("activity-primary".to_owned(), 11),
            ("activity-secondary".to_owned(), 22)
        ]
    );
    let label = AccountLabel::reference();
    let subject = account(&world, &request, "account-1");
    write(
        &world,
        (label.entity(), label.aspect(), label.field()),
        subject,
        AspectValue::String("changed".into()),
    );
    let (after, rows) =
        read::<GovernedDiagnosticAccountBinding>(&world, &external, &request, "open", true);
    assert_eq!(rows[0].label, "changed");
    // A second ordinary Native authored write has equal disclosed cell meaning.
    write(
        &world,
        (label.entity(), label.aspect(), label.field()),
        subject,
        AspectValue::String("changed".into()),
    );
    let (equal, rows) =
        read::<GovernedDiagnosticAccountBinding>(&world, &external, &request, "open", true);
    assert_eq!(rows[0].label, "changed");
    let authority = worth_execution::ExecutionAuthority::serial();
    let lease = authority
        .request_lease(worth_execution::LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    NonZeroUsize::MIN,
                    Some(32 * 1024 * 1024),
                    Some(100_000_000),
                ),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();
    let baseline_memory = lease.charged_memory_bytes();
    for (left, right, expected) in [
        (&before, &after, CausalInspectionReason::ChangedResult),
        (&after, &equal, CausalInspectionReason::UnchangedResult),
    ] {
        let metadata = capability(&world, &external, &request);
        let diagnostic = capability(&world, &external, &request);
        let result = world
            .application
            .inspect_application_observation::<GovernedDiagnosticAccountBinding>(
                &external, &request, right,
            )
            .why_changed(left)
            .reference_only()
            .with_allocation_policy(ExecutionAllocationPolicy::Execution(&lease))
            .execute_governed(metadata, diagnostic)
            .unwrap();
        assert_eq!(result.reason(), expected);
        assert!(std::ptr::eq(result.before(), left));
        assert!(std::ptr::eq(result.after(), right));
        assert!(!result.before_observation_identity().is_empty());
        assert!(!result.after_observation_identity().is_empty());
        assert_ne!(
            result.before_report_identity(),
            result.after_report_identity()
        );
        assert_ne!(result.report_identity(), result.before_report_identity());
        assert_eq!(registry.observe().active(), baseline_readers);
        assert!(lease.charged_memory_bytes() > baseline_memory);
        result.close();
        assert_eq!(lease.charged_memory_bytes(), baseline_memory);
    }
    assert_eq!(registry.observe().active(), baseline_readers);
}

#[test]
fn paired_native_nested_order_and_optional_presence_are_disclosed_output_changes() {
    let world = comparison_world();
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let registry = world.application.application_query_basis_observer();
    let baseline = registry.observe().active();
    let (before, rows) =
        read::<DiagnosticAccountBinding>(&world, &external, &request, "open", true);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].label, "primary");
    assert_eq!(rows[0].note.as_deref(), Some("reviewed"));
    let note = AccountNote::reference();
    clear_optional_field(
        &world,
        (note.entity(), note.aspect(), note.field()),
        account(&world, &request, "account-1"),
    );
    let (after, rows) = read::<DiagnosticAccountBinding>(&world, &external, &request, "open", true);
    assert_eq!(rows[0].note, None);
    let result =
        compare::<DiagnosticAccountBinding>(&world, &external, &request, &before, &after).unwrap();
    assert_eq!(result.reason(), CausalInspectionReason::ChangedResult);
    result.close();
    let (before, rows) =
        read::<DiagnosticAccountBinding>(&world, &external, &request, "open", true);
    assert_eq!(
        rows[0].activities,
        vec![
            ("activity-primary".to_owned(), 11),
            ("activity-secondary".to_owned(), 22)
        ]
    );
    let sequence = ActivitySequence::reference();
    write(
        &world,
        (sequence.entity(), sequence.aspect(), sequence.field()),
        activity(&world, &request, "activity-primary"),
        AspectValue::UInt64(33),
    );
    let (after, rows) = read::<DiagnosticAccountBinding>(&world, &external, &request, "open", true);
    assert_eq!(
        rows[0].activities,
        vec![
            ("activity-secondary".to_owned(), 22),
            ("activity-primary".to_owned(), 33)
        ]
    );
    let result =
        compare::<DiagnosticAccountBinding>(&world, &external, &request, &before, &after).unwrap();
    assert_eq!(result.reason(), CausalInspectionReason::ChangedResult);
    result.close();
    assert_eq!(registry.observe().active(), baseline);
}

#[path = "rich_query.rs"]
mod rich_query;
