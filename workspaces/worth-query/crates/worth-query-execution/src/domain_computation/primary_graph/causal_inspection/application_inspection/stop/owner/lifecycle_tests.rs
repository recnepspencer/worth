//! Projection-only courts use literal owner causes, never fabricated execution proof.
use super::*;
use InspectionAdmissionDenial as Admission;
fn denial(cause: Admission) -> WorthQueryApplicationInspectionDenial {
    WorthQueryApplicationInspectionDenial::new(InspectionCause::Admission(cause), None)
}
fn captured<E>(cause: E) -> CaptureStop<E> {
    CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Domain(cause)))
}
#[test]
fn inspection_preserves_every_read_cause_and_its_original_owner() {
    use worth_relational::facade::{
        indexes::DerivedIndexId, transactions::RelationalExecutionDenialCause as Native,
    };
    use WorthQueryManagedGraphReadDenial as Read;
    for expected in [
        Read::RequestStop(Native::PolicyMemoryExhausted {
            requested: 17,
            admitted: 5,
            ancestor: 2,
        }),
        Read::SnapshotUnavailable,
        Read::IndexNotInstalled(DerivedIndexId(37)),
        Read::MutationSession,
        Read::ForeignBasis,
        Read::ForeignGraph,
        Read::ForeignReadProof,
        Read::ForeignReadIntent,
        Read::TerminalReleaseMismatch,
    ] {
        let denied = denial(Admission::Read(expected));
        let Some(Refusal::Read(actual)) = denied.owner_refusal() else {
            panic!("exact read cause must survive");
        };
        assert_eq!(*actual, expected);
        let InspectionCause::Admission(Admission::Read(original)) = &denied.cause else {
            unreachable!()
        };
        assert!(std::ptr::eq(actual, original));
    }
}
#[test]
fn inspection_preserves_terminal_stage_and_original_cause() {
    use WorthQueryInspectionReadTerminalDenial as Terminal;
    use WorthQueryManagedGraphReadDenial as Read;
    for expected in [
        Terminal::Request(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::Cancelled,
        )),
        Terminal::Read(Read::ForeignReadIntent),
    ] {
        let denied = denial(Admission::Terminal(expected));
        let Some(Refusal::Terminal(actual)) = denied.owner_refusal() else {
            panic!("exact terminal stage must survive");
        };
        match actual {
            Terminal::Request(cause) => assert_eq!(
                *cause,
                worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::Cancelled
                )
            ),
            Terminal::Read(cause) => assert_eq!(*cause, Read::ForeignReadIntent),
        }
        let InspectionCause::Admission(Admission::Terminal(original)) = &denied.cause else {
            unreachable!()
        };
        assert!(std::ptr::eq(actual, original));
    }
}
#[test]
fn inspection_preserves_every_diagnostic_admission_cause() {
    use DiagnosticComparisonDenial as Diagnostic;
    for expected in [
        Diagnostic::NotDeclared,
        Diagnostic::UnsupportedShape,
        Diagnostic::ForeignBinding,
        Diagnostic::FieldDenied,
        Diagnostic::RelationDenied,
        Diagnostic::InternalInfluenceDenied,
    ] {
        let denied = denial(Admission::DiagnosticAdmission(captured(expected)));
        let Some(Refusal::DiagnosticAdmission(actual)) = denied.owner_refusal() else {
            panic!("exact diagnostic admission cause must survive");
        };
        let InspectionCause::Admission(Admission::DiagnosticAdmission(CaptureStop::Kernel(
            ScopeStop::Failure(MapKernelFailure::Domain(original)),
        ))) = &denied.cause
        else {
            unreachable!()
        };
        assert_eq!(actual, original);
        assert!(std::ptr::eq(actual, original));
    }
}
