//! Actual empty originals and current Diagnostic masks keep distinct rich outcomes.
use super::super::rich::{
    WorthQueryApplicationEvidenceFamily as Family, WorthQueryApplicationMaterializationRequest,
    WorthQueryMaterializedFamilyOutcome as Outcome,
};
use super::*;

fn detail() -> WorthQueryApplicationMaterializationRequest {
    WorthQueryApplicationMaterializationRequest::family(Family::QueryInspection)
}
#[test]
fn rich_retained_empty_is_provided_and_bodyless_is_not_retained() {
    let world = comparison_world();
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let readers = world.application.application_query_basis_observer();
    let baseline = readers.observe().active();
    for retain in [false, true] {
        let (original, rows) =
            read::<DiagnosticAccountBinding>(&world, &external, &request, "no-match", retain);
        assert!(rows.is_empty());
        let rich = world
            .application
            .inspect_application_observation::<DiagnosticAccountBinding>(
                &external, &request, &original,
            )
            .why_observed()
            .materialize(detail())
            .execute_with_diagnostic(capability(&world, &external, &request))
            .unwrap();
        if retain {
            assert_eq!(
                rich.materialization()
                    .family_outcome(Family::QueryInspection),
                Outcome::Provided
            );
            assert_eq!(
                rich.materialization()
                    .original_query()
                    .unwrap()
                    .rows()
                    .len(),
                0
            );
        } else {
            assert_eq!(
                rich.materialization()
                    .family_outcome(Family::QueryInspection),
                Outcome::OriginalProjectionNotRetained
            );
            assert!(rich.materialization().original_query().is_none());
        }
        rich.close();
        original.close();
    }
    assert_eq!(readers.observe().active(), baseline);
}
#[test]
fn rich_current_diagnostic_field_denial_precedes_original_availability() {
    let world = comparison_world();
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let readers = world.application.application_query_basis_observer();
    let baseline = readers.observe().active();
    for retain in [false, true] {
        let (original, rows) =
            read::<DeniedDiagnosticFieldBinding>(&world, &external, &request, "no-match", retain);
        assert!(rows.is_empty());
        let denied = world.application.inspect_application_observation::<DeniedDiagnosticFieldBinding>(
            &external, &request, &original).why_observed().materialize(detail())
            .execute_with_diagnostic(capability(&world, &external, &request))
            .err().expect("real current Diagnostic field policy refuses both retained and absent empty originals");
        assert!(matches!(
            &denied.cause,
            stop::InspectionCause::Admission(
                admission::InspectionAdmissionDenial::DiagnosticAdmission(CaptureStop::Kernel(
                    ScopeStop::Failure(MapKernelFailure::Domain(
                        DiagnosticComparisonDenial::FieldDenied
                    ))
                ))
            )
        ));
        assert!(denied.materialization_refusal().is_none());
        drop(denied);
        original.close();
    }
    assert_eq!(readers.observe().active(), baseline);
}
