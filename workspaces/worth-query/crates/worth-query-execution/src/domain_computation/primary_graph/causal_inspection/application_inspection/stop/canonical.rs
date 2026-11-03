//! Canonical owner denials remain distinct from original request resources.
use super::super::{identity::InspectionIdentityStop, report::InspectionReportDenial};
use super::*;
use worth_foundational::facade::{
    CanonicalBasisConstructionDenial, CanonicalBasisPreparationStop, CanonicalDigestAdmissionStop,
    CanonicalDigestDerivationDenial, CanonicalSortImplementationDenial,
};
use worth_runtime_bridge::facade::BridgeQueryObservationReportingStop;

/// Non-resource canonical refusal borrowed from the actual Basis/Digest owner.
#[derive(Clone, Copy, Debug)]
pub enum WorthQueryApplicationInspectionCanonicalRefusal<'a> {
    Basis(
        WorthQueryApplicationInspectionStage,
        &'a CanonicalBasisConstructionDenial,
    ),
    Digest(
        WorthQueryApplicationInspectionStage,
        &'a CanonicalDigestDerivationDenial,
    ),
    UnsupportedSortImplementation(
        WorthQueryApplicationInspectionStage,
        &'a CanonicalSortImplementationDenial,
    ),
    AccountingOverflow(WorthQueryApplicationInspectionStage),
    AllocationUnavailable(WorthQueryApplicationInspectionStage),
}
type Refusal<'a> = WorthQueryApplicationInspectionCanonicalRefusal<'a>;

pub(super) fn cause(cause: &InspectionCause) -> Option<Refusal<'_>> {
    match cause {
        InspectionCause::Admission(cause) => match cause {
            InspectionAdmissionDenial::Report(stop) => report(captured_domain::captured(stop)?),
            InspectionAdmissionDenial::Request(_)
            | InspectionAdmissionDenial::RequestControl(_)
            | InspectionAdmissionDenial::ForeignIssuedAccess
            | InspectionAdmissionDenial::MissingGraph
            | InspectionAdmissionDenial::MissingInspection
            | InspectionAdmissionDenial::Authorization(_)
            | InspectionAdmissionDenial::CurrentAuthorization(_)
            | InspectionAdmissionDenial::Capability(_)
            | InspectionAdmissionDenial::Scope(_)
            | InspectionAdmissionDenial::Security(_)
            | InspectionAdmissionDenial::Governance(_)
            | InspectionAdmissionDenial::FacetDenied
            | InspectionAdmissionDenial::DiagnosticAdmission(_)
            | InspectionAdmissionDenial::ComparisonParameters(_)
            | InspectionAdmissionDenial::Connection(_)
            | InspectionAdmissionDenial::MissingOriginalProjection
            | InspectionAdmissionDenial::RichSource(_)
            | InspectionAdmissionDenial::Rich(_)
            | InspectionAdmissionDenial::Comparison(_)
            | InspectionAdmissionDenial::DiagnosticContract(_)
            | InspectionAdmissionDenial::DiagnosticPreparation(_)
            | InspectionAdmissionDenial::Target(_)
            | InspectionAdmissionDenial::Storage(_)
            | InspectionAdmissionDenial::Plan(_)
            | InspectionAdmissionDenial::Session(_)
            | InspectionAdmissionDenial::Read(_)
            | InspectionAdmissionDenial::Terminal(_) => None,
        },
        InspectionCause::Unavailable
        | InspectionCause::Advancement(_)
        | InspectionCause::Selection(_)
        | InspectionCause::Binding(_)
        | InspectionCause::Principal(_)
        | InspectionCause::Scope(_)
        | InspectionCause::Query(_) => None,
    }
}
fn report(cause: &InspectionReportDenial) -> Option<Refusal<'_>> {
    use WorthQueryApplicationInspectionStage as Stage;
    match cause {
        InspectionReportDenial::Identity(cause) => match cause {
            InspectionIdentityStop::Basis(cause) => basis(Stage::ReportIdentity, cause),
            InspectionIdentityStop::Digest(cause) => digest(Stage::ReportIdentity, cause),
            InspectionIdentityStop::Overflow
            | InspectionIdentityStop::ForeignSession
            | InspectionIdentityStop::ProductRequired
            | InspectionIdentityStop::Resource(_) => None,
        },
        InspectionReportDenial::Bridge(cause) => match cause {
            BridgeQueryObservationReportingStop::Basis(cause) => basis(Stage::ReportBridge, cause),
            BridgeQueryObservationReportingStop::Digest(cause) => {
                digest(Stage::ReportBridge, cause)
            }
            BridgeQueryObservationReportingStop::AccountingOverflow
            | BridgeQueryObservationReportingStop::InvalidReportingAssociation
            | BridgeQueryObservationReportingStop::Resource(_) => None,
        },
        InspectionReportDenial::Input(_)
        | InspectionReportDenial::ForeignComparison
        | InspectionReportDenial::ForeignMaterialization
        | InspectionReportDenial::Storage(_) => None,
    }
}

fn basis<S>(
    stage: WorthQueryApplicationInspectionStage,
    cause: &CanonicalBasisPreparationStop<S>,
) -> Option<Refusal<'_>> {
    match cause {
        CanonicalBasisPreparationStop::Construction(cause) => Some(Refusal::Basis(stage, cause)),
        CanonicalBasisPreparationStop::UnsupportedSortImplementation(cause) => {
            Some(Refusal::UnsupportedSortImplementation(stage, cause))
        }
        CanonicalBasisPreparationStop::AccountingOverflow => {
            Some(Refusal::AccountingOverflow(stage))
        }
        CanonicalBasisPreparationStop::Resource { .. } => None,
    }
}

fn digest<S>(
    stage: WorthQueryApplicationInspectionStage,
    cause: &CanonicalDigestAdmissionStop<S>,
) -> Option<Refusal<'_>> {
    match cause {
        CanonicalDigestAdmissionStop::Derivation(cause) => Some(Refusal::Digest(stage, cause)),
        CanonicalDigestAdmissionStop::AccountingOverflow => {
            Some(Refusal::AccountingOverflow(stage))
        }
        CanonicalDigestAdmissionStop::AllocationUnavailable => {
            Some(Refusal::AllocationUnavailable(stage))
        }
        CanonicalDigestAdmissionStop::Resource(_) => None,
    }
}
