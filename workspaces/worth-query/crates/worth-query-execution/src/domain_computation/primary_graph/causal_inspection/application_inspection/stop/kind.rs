//! Explicit public coarse classification after resource extraction.
use super::*;
use WorthQueryApplicationInspectionStopKind as Kind;
pub(super) fn admission(cause: &InspectionAdmissionDenial) -> Kind {
    match cause {
        InspectionAdmissionDenial::MissingOriginalProjection => Kind::Unavailable,
        InspectionAdmissionDenial::Request(_) | InspectionAdmissionDenial::RequestControl(_) => Kind::Request,
        InspectionAdmissionDenial::DiagnosticContract(stop) => match captured_domain::captured(stop) {
            Some(crate::domain_computation::authorization::application_disclosure::DiagnosticContractDenial::NotDeclared
                | crate::domain_computation::authorization::application_disclosure::DiagnosticContractDenial::UnsupportedShape) => Kind::Unavailable,
            Some(crate::domain_computation::authorization::application_disclosure::DiagnosticContractDenial::Contract(_)
                | crate::domain_computation::authorization::application_disclosure::DiagnosticContractDenial::Storage(_)) | None => Kind::Admission,
        },
        InspectionAdmissionDenial::ForeignIssuedAccess | InspectionAdmissionDenial::MissingGraph | InspectionAdmissionDenial::MissingInspection | InspectionAdmissionDenial::Authorization(_) | InspectionAdmissionDenial::CurrentAuthorization(_) | InspectionAdmissionDenial::Capability(_) | InspectionAdmissionDenial::Scope(_) | InspectionAdmissionDenial::Security(_) | InspectionAdmissionDenial::Governance(_) | InspectionAdmissionDenial::FacetDenied | InspectionAdmissionDenial::DiagnosticAdmission(_) | InspectionAdmissionDenial::ComparisonParameters(_) | InspectionAdmissionDenial::Connection(_) | InspectionAdmissionDenial::RichSource(_) | InspectionAdmissionDenial::Rich(_) | InspectionAdmissionDenial::Comparison(_) | InspectionAdmissionDenial::DiagnosticPreparation(_) | InspectionAdmissionDenial::Target(_) | InspectionAdmissionDenial::Storage(_) | InspectionAdmissionDenial::Plan(_) | InspectionAdmissionDenial::Session(_) | InspectionAdmissionDenial::Read(_) | InspectionAdmissionDenial::Terminal(_) | InspectionAdmissionDenial::Report(_) => Kind::Admission,
    }
}
