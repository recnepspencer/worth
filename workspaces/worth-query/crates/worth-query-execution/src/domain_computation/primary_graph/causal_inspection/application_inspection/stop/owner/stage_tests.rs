//! Literal diagnostic contracts independently name each previously erased stage.
use super::super::super::{
    identity::InspectionIdentityStop as Identity, report::InspectionReportDenial as Report,
    rich::source::SourceMaterializationDenial as Source,
};
use super::*;
use crate::domain_computation::primary_graph::application_query::ProjectionComparisonDenial as Comparison;
use worth_runtime_bridge::facade::BridgeQueryObservationReportingStop as Bridge;
use InspectionAdmissionDenial as A;
fn captured<E>(cause: E) -> CaptureStop<E> {
    CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Domain(cause)))
}
fn denial(cause: A) -> WorthQueryApplicationInspectionDenial {
    WorthQueryApplicationInspectionDenial::new(InspectionCause::Admission(cause), None)
}
macro_rules! stage_court {
    ($name:ident, $input:expr, $expected:pat) => {
        #[test]
        fn $name() {
            let denied = denial($input);
            assert!(
                matches!(denied.owner_refusal(), Some($expected)),
                "{:?}",
                denied.owner_refusal()
            );
        }
    };
}
stage_court!(
    inspection_preserves_missing_graph,
    A::MissingGraph,
    Refusal::MissingGraph
);
stage_court!(
    inspection_preserves_missing_inspection,
    A::MissingInspection,
    Refusal::MissingInspection
);
stage_court!(
    inspection_preserves_facet_denied,
    A::FacetDenied,
    Refusal::FacetDenied
);
stage_court!(
    inspection_preserves_missing_original_projection,
    A::MissingOriginalProjection,
    Refusal::MissingOriginalProjection
);
stage_court!(
    inspection_preserves_diagnostic_contract_not_declared,
    A::DiagnosticContract(captured(DiagnosticContractDenial::NotDeclared)),
    Refusal::DiagnosticContractNotDeclared
);
stage_court!(
    inspection_preserves_diagnostic_contract_unsupported,
    A::DiagnosticContract(captured(DiagnosticContractDenial::UnsupportedShape)),
    Refusal::DiagnosticContractUnsupportedShape
);
stage_court!(
    inspection_preserves_comparison_parameter_stage,
    A::ComparisonParameters(captured(())),
    Refusal::ComparisonParametersUnavailable
);
stage_court!(
    inspection_preserves_source_unresolved_field,
    A::RichSource(captured(Source::UnresolvedField)),
    Refusal::SourceUnresolvedField
);
stage_court!(
    inspection_preserves_source_missing_value,
    A::RichSource(captured(Source::MissingOriginalValue)),
    Refusal::SourceMissingOriginalValue
);
stage_court!(
    inspection_preserves_comparison_interpretation,
    A::Comparison(captured(Comparison::DisclosureInterpretationChanged)),
    Refusal::ComparisonDisclosureInterpretationChanged
);
stage_court!(
    inspection_preserves_comparison_incomplete,
    A::Comparison(captured(Comparison::OriginalDisclosureIncomplete)),
    Refusal::ComparisonOriginalDisclosureIncomplete
);
stage_court!(
    inspection_preserves_comparison_nesting,
    A::Comparison(captured(Comparison::NestingExceedsDeclaredShape)),
    Refusal::ComparisonNestingExceedsDeclaredShape
);
stage_court!(
    inspection_preserves_plan_overflow,
    A::Plan(captured(InspectionPlanDenial::Overflow)),
    Refusal::PlanAccountingOverflow
);
stage_court!(
    inspection_preserves_foreign_report_comparison,
    A::Report(captured(Report::ForeignComparison)),
    Refusal::ReportForeignComparison
);
stage_court!(
    inspection_preserves_foreign_report_materialization,
    A::Report(captured(Report::ForeignMaterialization)),
    Refusal::ReportForeignMaterialization
);
stage_court!(
    inspection_preserves_report_identity_overflow,
    A::Report(captured(Report::Identity(Identity::Overflow))),
    Refusal::ReportIdentityOverflow
);
stage_court!(
    inspection_preserves_report_identity_session,
    A::Report(captured(Report::Identity(Identity::ForeignSession))),
    Refusal::ReportIdentityForeignSession
);
stage_court!(
    inspection_preserves_report_identity_product,
    A::Report(captured(Report::Identity(Identity::ProductRequired))),
    Refusal::ReportIdentityProductRequired
);
stage_court!(
    inspection_preserves_bridge_overflow,
    A::Report(captured(Report::Bridge(Bridge::AccountingOverflow))),
    Refusal::ReportBridgeAccountingOverflow
);
stage_court!(
    inspection_preserves_bridge_association,
    A::Report(captured(Report::Bridge(
        Bridge::InvalidReportingAssociation
    ))),
    Refusal::ReportBridgeInvalidReportingAssociation
);
#[test]
fn inspection_preserves_unavailable_owner_stage() {
    let denied = WorthQueryApplicationInspectionDenial::new(InspectionCause::Unavailable, None);
    assert!(matches!(denied.owner_refusal(), Some(Refusal::Unavailable)));
}
#[test]
fn inspection_preserves_original_advancement_owner() {
    use crate::domain_computation::request_execution::WorthQueryAdvancementDenial as Advancement;
    for cause in [
        Advancement::NestedOpening,
        Advancement::NestedStopped,
        Advancement::Panicked,
    ] {
        let denied =
            WorthQueryApplicationInspectionDenial::new(InspectionCause::Advancement(cause), None);
        let Some(Refusal::Advancement(actual)) = denied.owner_refusal() else {
            panic!("original advancement must survive");
        };
        let InspectionCause::Advancement(original) = &denied.cause else {
            unreachable!()
        };
        assert!(std::ptr::eq(actual, original));
    }
}
